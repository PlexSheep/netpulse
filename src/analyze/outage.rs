use std::fmt::Display;
use std::fmt::Write;

use tracing::error;

use crate::records::{display_group, Check};

use super::{fmt_timestamp, key_value_write, CheckGroup};

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub struct FromRawSeverityError(f64);

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub enum Severity {
    Total,
    Partial(f64),
    None,
}

impl TryFrom<f64> for Severity {
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value > 1.0 {
            return Err(FromRawSeverityError(value));
        }
        Ok(if value == 1.0 {
            Severity::Total
        } else if value == 0.0 {
            Severity::None
        } else {
            Severity::Partial(value)
        })
    }

    type Error = FromRawSeverityError;
}

/// Represents a period of consecutive failed checks.
///
/// An outage is defined by:
/// - A start check that failed
/// - An optional end check (None if outage is ongoing)
/// - All failed checks during the outage period
///
/// This struct helps track and analyze network connectivity issues
/// over time.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Outage<'check> {
    /// All checks that failed during this outage period
    all: Vec<&'check Check>,
}

impl<'check> Outage<'check> {
    /// Creates a new outage from its constituent checks.
    ///
    /// # Arguments
    ///
    /// * `all_checks` - Slice of all failed checks in this period
    pub fn new(all_checks: &[&'check Check]) -> Self {
        {
            let mut f = String::new();
            display_group(all_checks, &mut f).expect("could not dump checks");
        }
        let mut all = all_checks.to_vec();
        all.sort();
        Self { all }
    }

    /// Returns a reference to all [Checks](Check) of this [`Outage`].
    pub fn all(&self) -> &[&Check] {
        &self.all
    }

    /// Returns the last [Check] of the [Outage], or `None` if it is empty.
    pub fn last(&self) -> Option<&Check> {
        self.all.last().copied()
    }

    /// Returns the first [Check] of the [Outage], or `None` if it is empty.
    pub fn first(&self) -> Option<&Check> {
        self.all.first().copied()
    }

    /// Display information about that [Outage] in a short format
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
        write!(&mut buf, ", Total {}", self.len())?;
        Ok(buf)
    }

    /// Returns the length of this [`Outage`].
    pub fn len(&self) -> usize {
        self.all.len()
    }

    /// Returns true if this [`Outage`] is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn severity(&self) -> Severity {
        let all = self.all();
        let percentage: f64 =
            all.len() as f64 / all.iter().filter(|a| !a.is_success()).count() as f64;
        Severity::try_from(percentage).expect("calculated more than 100% success")
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
        writeln!(buf, "\nFirst\n{}", self.last().unwrap())?;
        writeln!(buf, "\nLast\n{}", self.last().unwrap())?;
        write!(f, "{buf}")?;
        Ok(())
    }
}
