//! Analysis and tracking of network outage periods.
//!
//! This module provides types and functions for analyzing periods of failed network checks:
//! - [`Outage`] - Represents a period of consecutive failed checks
//! - [`Severity`] - Classifies outage impact (complete, partial, none)
//!
//! # Outage Analysis
//!
//! An outage is defined as a period containing one or more failed network checks. The module helps:
//! - Track start/end times of outages
//! - Calculate outage severity/impact
//! - Generate outage reports and statistics

use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Write;

use tracing::error;
use tracing::warn;

use crate::records::Check;

use super::{fmt_timestamp, key_value_write, CheckGroup};

/// Error returned when trying to create a [`Severity`] from an invalid value.
///
/// Returned when attempting to create a [`Severity`] from a value greater than 1.0
/// since severity is represented as a percentage between 0.0 and 1.0.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct FromRawSeverityError(f64);

/// Classification of outage impact severity.
///
/// Represents how severely network connectivity was impacted during an outage period:
/// - Complete (only failed checks)
/// - Partial (some failed checks)
/// - None (no failed checks)
///
/// # Examples
///
/// ```rust
/// use netpulse::analyze::outage::Severity;
///
/// let complete = Severity::try_from(1.0).unwrap();
/// let partial = Severity::try_from(0.5).unwrap();
/// let none = Severity::try_from(0.0).unwrap();
///
/// assert!(complete > partial);
/// assert!(partial > none);
/// ```
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Severity {
    /// All checks failed (100% failure rate)
    Complete,
    /// Some checks failed (partial failure rate between 0-100%)
    Partial(f64),
    /// No checks failed (0% failure rate)
    None,
}

impl TryFrom<f64> for Severity {
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value > 1.0 {
            return Err(FromRawSeverityError(value));
        }
        Ok(if value == 1.0 {
            Severity::Complete
        } else if value == 0.0 {
            Severity::None
        } else {
            Severity::Partial(value)
        })
    }

    type Error = FromRawSeverityError;
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Complete, Self::Complete) => Some(std::cmp::Ordering::Equal),
            (Self::Complete, _) => Some(std::cmp::Ordering::Greater),
            (Self::Partial(p1), Self::Partial(p2)) => p1.partial_cmp(p2),
            (Self::Partial(_), Self::Complete) => Some(std::cmp::Ordering::Less),
            (Self::Partial(_), Self::None) => Some(std::cmp::Ordering::Greater),
            (Self::None, Self::None) => Some(std::cmp::Ordering::Equal),
            (Self::None, _) => Some(std::cmp::Ordering::Less),
        }
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Complete => write!(f, "Complete")?,
            Self::None => write!(f, "No Outage")?,
            Self::Partial(p) => write!(f, "Partial ({:.02} %)", p * 100.0)?,
        }
        Ok(())
    }
}

/// Represents a period of consecutive failed network checks.
///
/// An outage is defined by:
/// - One or more consecutive checks that failed
/// - An outage with no checks is technically allowed, but serves no purpose
///
/// From that, we can extrapolate:
/// - A start time (first failed check)
/// - An end time (last failed check)
/// - A severity classification
///
/// # Examples
///
/// ```rust,no_run
/// use netpulse::records::Check;
/// use netpulse::analyze::outage::Outage;
///
/// # let checks = vec![];
/// let outage = Outage::new(&checks);
///
/// println!("Outage report:\n{}", outage.short_report().unwrap());
/// ```
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct Outage<'check> {
    /// All checks that occurred during this outage period
    all: Vec<&'check Check>,
}

impl<'check> Outage<'check> {
    /// Creates a new outage from a slice of checks.
    ///
    /// # Arguments
    ///
    /// * `all_checks` - Slice of all checks in this period (both failed and successful)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use netpulse::records::Check;
    /// use netpulse::analyze::outage::Outage;
    ///
    /// # let checks = vec![];
    /// let outage = Outage::new(&checks);
    /// ```
    pub fn new(all_checks: &[&'check Check]) -> Self {
        if all_checks.is_empty() {
            warn!("created an empty outage");
        }
        let mut all = all_checks.to_vec();
        all.sort();
        Self { all }
    }

    /// Returns a reference to all [Checks](Check) of this [`Outage`].
    pub fn all(&self) -> &[&Check] {
        &self.all
    }

    /// Returns the last [Check] of the [Outage], or [`None`] if it is empty.
    pub fn last(&self) -> Option<&Check> {
        self.all.last().copied()
    }

    /// Returns the first [Check] of the [Outage], or [`None`] if it is empty.
    pub fn first(&self) -> Option<&Check> {
        self.all.first().copied()
    }

    /// Generates a concise single-line report of the outage.
    ///
    /// The report includes:
    /// - Start time
    /// - End time  
    /// - Total number of checks
    /// - Severity classification
    ///
    /// # Errors
    ///
    /// Returns [`std::fmt::Error`] if string formatting fails.
    pub fn short_report(&self) -> Result<String, std::fmt::Error> {
        if self.is_empty() {
            error!("Outage does not contain any checks");
        }
        let mut buf: String = String::new();
        write!(
            &mut buf,
            "From {}",
            fmt_timestamp(self.first().unwrap().timestamp_parsed()),
        )?;
        write!(
            &mut buf,
            " To {}",
            fmt_timestamp(self.last().unwrap().timestamp_parsed()),
        )?;
        write!(&mut buf, ", Total {:>6}", self.len())?;
        write!(&mut buf, ", {}", self.severity())?;
        Ok(buf)
    }

    /// Returns the total number of checks in this outage period.
    pub fn len(&self) -> usize {
        self.all.len()
    }

    /// Returns true if this [`Outage`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Calculates the severity of this outage.
    ///
    /// Severity is based on the percentage of failed checks:
    /// - 100% = Complete outage
    /// - 0% = No outage
    /// - Other = Partial outage
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use netpulse::records::Check;
    /// use netpulse::analyze::outage::Outage;
    ///
    /// # let checks = vec![];
    /// let outage = Outage::new(&checks);
    /// println!("Severity: {}", outage.severity());
    /// ```
    pub fn severity(&self) -> Severity {
        let all = self.all();
        let percentage: f64 =
            all.iter().filter(|a| !a.is_success()).count() as f64 / all.len() as f64;
        Severity::try_from(percentage).expect("calculated more than 100% success")
    }

    /// Compares two outages by severity then by duration.
    ///
    /// Orders outages first by severity (complete > partial > none),
    /// then by number of checks for equal severities.
    pub fn cmp_severity(&self, other: &Self) -> Ordering {
        match self
            .severity()
            .partial_cmp(&other.severity())
            .unwrap_or(Ordering::Equal)
        {
            Ordering::Equal => self.len().cmp(&other.len()),
            other => other,
        }
    }
}

impl<'check> From<&'check [Check]> for Outage<'check> {
    fn from(value: &'check [Check]) -> Self {
        if value.is_empty() {
            panic!("tried to make an outage from an empty check group");
        }
        let a: Vec<&Check> = value.iter().collect();
        Outage::new(&a)
    }
}

impl<'check> From<&'check Vec<&Check>> for Outage<'check> {
    fn from(value: &'check Vec<&Check>) -> Self {
        Outage::new(value)
    }
}

impl<'check> From<CheckGroup<'check>> for Outage<'check> {
    fn from(value: CheckGroup<'check>) -> Self {
        if value.is_empty() {
            panic!("tried to make an outage from an empty check group");
        }
        Outage::new(&value)
    }
}

impl Display for Outage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            error!("Outage does not contain any checks");
        }
        let mut buf: String = String::new();
        key_value_write(
            &mut buf,
            "From",
            fmt_timestamp(self.first().unwrap().timestamp_parsed()),
        )?;
        key_value_write(
            &mut buf,
            "To",
            fmt_timestamp(self.last().unwrap().timestamp_parsed()),
        )?;
        key_value_write(&mut buf, "Total", self.len())?;
        key_value_write(&mut buf, "Severity", self.severity())?;
        writeln!(buf, "\nFirst\n{}", self.last().unwrap())?;
        writeln!(buf, "\nLast\n{}", self.last().unwrap())?;
        write!(f, "{buf}")?;
        Ok(())
    }
}
