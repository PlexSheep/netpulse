//! Module providing analysis and reporting of network check results.
//!
//! # Analysis Features
//!
//! This module analyzes data from the [Store] to provide:
//! - Outage detection and tracking
//! - Success/failure statistics per check type
//! - Latency analysis
//! - Report generation
//!
//! The main entry point is the [analyze] function which generates
//! a comprehensive report of the store's contents.
//!
//! # Examples
//!
//! ```rust,no_run
//! use netpulse::{store::Store, analyze};
//!
//! let store = Store::load(true).unwrap();
//! let report = analyze::analyze(&store).unwrap();
//! println!("{}", report);
//! ```
//!
//! # Report Sections
//!
//! The analysis report contains several sections:
//! - General statistics (total checks, success rates)
//! - HTTP-specific metrics
//! - Outage analysis
//! - Store metadata (hashes, versions)

use chrono::{DateTime, Local};
use deepsize::DeepSizeOf;
use tracing::{error, trace};

use crate::errors::AnalysisError;
use crate::records::{display_group, Check, CheckType, IpType};
use crate::store::Store;

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Write};
use std::hash::Hash;
use std::os::unix::fs::MetadataExt;

/// Formatting rules for timestamps that are easily readable by humans.
///
/// ```rust
/// use chrono::{DateTime, Local};
/// # use netpulse::analyze::TIME_FORMAT_HUMANS;
/// let datetime: DateTime<Local> = Local::now();
/// println!("it is now: {}", datetime.format(TIME_FORMAT_HUMANS));
/// ```
pub const TIME_FORMAT_HUMANS: &str = "%Y-%m-%d %H:%M:%S %Z";
/// A group of [Checks](Check)
pub type CheckGroup<'check> = Vec<&'check Check>;

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
    /// First check that failed, marking the start of the outage
    start: &'check Check,
    /// Last failed check before connectivity was restored
    /// [None] if the outage is still ongoing
    end: Option<&'check Check>,
    /// All checks that failed during this outage period
    all: Vec<&'check Check>,
}

impl<'check> Outage<'check> {
    /// Creates a new outage from its constituent checks.
    ///
    /// # Arguments
    ///
    /// * `start` - The first failed check
    /// * `end` - Optional last failed check (None if ongoing)
    /// * `all_checks` - Slice of all failed checks in this period
    pub(crate) fn new(
        start: &'check Check,
        end: Option<&'check Check>,
        all_checks: &[&'check Check],
    ) -> Self {
        {
            let mut f = String::new();
            display_group(all_checks, &mut f).expect("could not dump checks");
            trace!("dumping outage at creation: {f}",);
        }
        Self {
            start,
            end: if Some(start) == end { None } else { end },
            all: all_checks.to_vec(),
        }
    }

    /// Display information about that [Outage] in a short format
    pub fn short_report(&self) -> Result<String, std::fmt::Error> {
        let mut buf: String = String::new();
        if self.end.is_some() {
            write!(
                &mut buf,
                "From {}",
                fmt_timestamp(self.start.timestamp_parsed()),
            )?;
            write!(
                &mut buf,
                " To {}",
                fmt_timestamp(self.end.unwrap().timestamp_parsed()),
            )?;
        } else {
            write!(
                &mut buf,
                "From {}",
                fmt_timestamp(self.start.timestamp_parsed()),
            )?;
            write!(&mut buf, " To (None)")?;
        }
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
}

impl<'check> From<CheckGroup<'check>> for Outage<'check> {
    fn from(value: CheckGroup<'check>) -> Self {
        if value.is_empty() {
            panic!("tried to make an outage from an empty check group");
        }
        Outage::new(value.first().unwrap(), value.last().copied(), &value)
    }
}

impl Display for Outage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf: String = String::new();
        if self.end.is_some() {
            key_value_write(
                &mut buf,
                "From",
                fmt_timestamp(self.start.timestamp_parsed()),
            )?;
            key_value_write(
                &mut buf,
                "To",
                fmt_timestamp(self.end.unwrap().timestamp_parsed()),
            )?;
        } else {
            key_value_write(
                &mut buf,
                "From",
                fmt_timestamp(self.start.timestamp_parsed()),
            )?;
            key_value_write(&mut buf, "To", "(None)")?;
        }
        key_value_write(&mut buf, "Total", self.len())?;
        writeln!(buf, "\nFirst\n{}", self.start)?;
        writeln!(
            buf,
            "\nLast\n{}",
            if let Some(c) = self.end {
                c.to_string()
            } else {
                "(None)".to_string()
            }
        )?;
        write!(f, "{buf}")?;
        Ok(())
    }
}

fn more_indent(buf: &str) -> String {
    format!("\t{}", buf.to_string().replace("\n", "\n\t"))
}

/// Generate a comprehensive analysis report for the given store.
///
/// The report includes:
/// - General check statistics
/// - HTTP-specific metrics
/// - Outage analysis
/// - Store metadata
///
/// # Errors
///
/// Returns [AnalysisError] if:
/// - Report string formatting fails
/// - Store hash calculation fails
///
/// # Example
///
/// ```rust,no_run
/// use netpulse::{store::Store, analyze};
///
/// let store = Store::load(true).unwrap();
/// let report = analyze::analyze(&store).unwrap();
/// println!("{}", report);
/// ```
pub fn analyze(store: &Store) -> Result<String, AnalysisError> {
    let mut f = String::new();
    barrier(&mut f, "General")?;
    generalized(store, &mut f)?;
    barrier(&mut f, "HTTP")?;
    generic_type_analyze(store, &mut f, CheckType::Http)?;
    barrier(&mut f, "ICMP")?;
    generic_type_analyze(store, &mut f, CheckType::Icmp)?;
    barrier(&mut f, "IPv4")?;
    gereric_ip_analyze(store, &mut f, IpType::V4)?;
    barrier(&mut f, "IPv6")?;
    gereric_ip_analyze(store, &mut f, IpType::V6)?;
    barrier(&mut f, "Outages")?;
    outages(store, &mut f)?;
    barrier(&mut f, "Store Metadata")?;
    store_meta(store, &mut f)?;

    Ok(f)
}

/// Formats a [SystemTime](std::time::SystemTime) as an easily readable timestamp for humans.
///
/// Works with [`std::time::SystemTime`] and [`chrono::DateTime<Local>`].
///
/// # Examples
///
/// ```rust
/// # use netpulse::analyze::fmt_timestamp;
/// use std::time::SystemTime;
/// use chrono;
/// let datetime: SystemTime = SystemTime::now();
/// println!("it is now: {}", fmt_timestamp(datetime));
/// let datetime: chrono::DateTime<chrono::Local> = chrono::Local::now();
/// println!("it is now: {}", fmt_timestamp(datetime));
/// let datetime: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
/// println!("it is now: {}", fmt_timestamp(datetime));
/// ```
pub fn fmt_timestamp(timestamp: impl Into<DateTime<Local>>) -> String {
    let a: chrono::DateTime<chrono::Local> = timestamp.into();
    format!("{}", a.format(TIME_FORMAT_HUMANS))
}

/// Adds a section divider to the report with a title.
///
/// Creates a divider line of '=' characters with the title centered.
///
/// # Errors
///
/// Returns [AnalysisError] if string formatting fails.
fn barrier(f: &mut String, title: &str) -> Result<(), AnalysisError> {
    writeln!(f, "{:=<10}{:=<48}", "", format!(" {title} "))?;
    Ok(())
}

/// Writes a key-value pair to the report in aligned columns.
///
/// Format: `<key>: <value>`
fn key_value_write(
    f: &mut String,
    title: &str,
    content: impl Display,
) -> Result<(), std::fmt::Error> {
    writeln!(f, "{:<24}: {}", title, content)
}

/// Analyzes and formats outage information from the store.
///
/// Groups consecutive failed checks by check type and creates
/// Outage records for reporting.
fn outages(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    let all: Vec<&Check> = store.checks().iter().collect();
    let fails_exist = !all.iter().all(|c| c.is_success());
    if !fails_exist || all.is_empty() {
        writeln!(f, "None\n")?;
        return Ok(());
    }

    let fail_groups = fail_groups(&all);
    for (outage_idx, group) in fail_groups.into_iter().rev().enumerate() {
        if group.is_empty() {
            error!("empty outage group");
            continue;
        }
        let outage = Outage::from(group);
        writeln!(f, "{outage_idx}:\t{}", &outage.short_report()?)?;
        if outage_idx >= 9 {
            writeln!(f, "\nshowing only the 10 latest outages...")?;
            break;
        }
    }
    writeln!(f)?;
    Ok(())
}

/// Analyzes and formats outage information from the store.
///
/// Groups consecutive failed checks by check type and creates
/// Outage records for reporting. This is the more detailed version of [outages]
pub fn outages_detailed(all: &[&Check], f: &mut String) -> Result<(), AnalysisError> {
    let fails_exist = !all.iter().all(|c| c.is_success());
    if !fails_exist || all.is_empty() {
        writeln!(f, "None\n")?;
        return Ok(());
    }

    let fail_groups = fail_groups(all);
    for (outage_idx, group) in fail_groups.into_iter().enumerate() {
        if group.is_empty() {
            error!("empty outage group");
            continue;
        }
        let outage = Outage::from(group);
        writeln!(f, "{outage_idx}:\n{}", more_indent(&outage.to_string()))?;
    }
    writeln!(f)?;

    Ok(())
}

/// Groups checks by time
fn group_by_time<'check>(checks: &[&'check Check]) -> HashMap<i64, CheckGroup<'check>> {
    let mut groups: HashMap<i64, CheckGroup<'check>> = HashMap::new();

    for check in checks {
        groups.entry(check.timestamp()).or_default().push(check);
    }

    groups
}

fn fail_groups<'check>(checks: &[&'check Check]) -> Vec<CheckGroup<'check>> {
    let by_time = group_by_time(checks);
    let mut groups = Vec::new();
    let mut processed_times: HashSet<i64> = HashSet::new();

    for (time, current_checks) in by_time.iter() {
        // Skip if we've already processed this time as part of another group
        if processed_times.contains(time) {
            continue;
        }
        // Skip if this check series did not fail
        if current_checks.iter().all(|a| a.is_success()) {
            continue;
        }

        let mut current_group = Vec::new();

        let keys_after: Vec<&i64> = by_time.keys().filter(|v| **v > *time).collect();
        let keys_before: Vec<&i64> = by_time.keys().filter(|v| **v <= *time).collect();

        // find the start of the outage
        for t in keys_before.iter().rev() {
            let checks: &CheckGroup<'_> = &by_time[t];
            if checks.iter().all(|a| a.is_success()) {
                continue;
            } else {
                current_group.extend(checks);
                processed_times.insert(**t);
            }
        }

        // find the end of the outage
        for t in keys_after.iter() {
            let checks: &CheckGroup<'_> = &by_time[t];
            if checks.iter().all(|a| a.is_success()) {
                continue;
            } else {
                current_group.extend(checks);
                processed_times.insert(**t);
            }
        }

        if !current_group.is_empty() {
            current_group.sort();
            groups.push(current_group);
        }
    }

    groups.sort();
    groups
}

/// Analyze metrics for a specific check type.
///
/// Calculates and formats:
/// - Total check count
/// - Success/failure counts
/// - Success ratio
/// - First/last check timestamps
///
/// # Errors
///
/// Returns [AnalysisError] if formatting fails.
fn analyze_check_type_set(
    f: &mut String,
    all: &[&Check],
    successes: &[&Check],
) -> Result<(), AnalysisError> {
    if all.is_empty() {
        writeln!(f, "None\n")?;
        return Ok(());
    }
    key_value_write(f, "checks", format!("{:08}", all.len()))?;
    key_value_write(f, "checks ok", format!("{:08}", successes.len()))?;
    key_value_write(
        f,
        "checks bad",
        format!("{:08}", all.len() - successes.len()),
    )?;
    key_value_write(
        f,
        "success ratio",
        format!(
            "{:03.02}%",
            success_ratio(all.len(), successes.len()) * 100.0
        ),
    )?;
    key_value_write(
        f,
        "first check at",
        fmt_timestamp(all.first().unwrap().timestamp_parsed()),
    )?;
    key_value_write(
        f,
        "last check at",
        fmt_timestamp(all.last().unwrap().timestamp_parsed()),
    )?;
    writeln!(f)?;
    Ok(())
}

/// Write general check statistics section of the report.
///
/// Includes metrics across all check types combined.
fn generalized(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    if store.checks().is_empty() {
        writeln!(f, "Store has no checks yet\n")?;
        return Ok(());
    }
    let all: Vec<&Check> = store.checks().iter().collect();
    let successes: Vec<&Check> = store.checks().iter().filter(|c| c.is_success()).collect();
    analyze_check_type_set(f, &all, &successes)?;
    Ok(())
}

/// Write check statistics section of the report for `check_type`.
///
/// Analyzes and formats statistics for IPv4/IPv6 checks.
///
/// Collects all checks that used that IP and generates a statistical report including:
/// - Total number of that IP checks
/// - Success/failure counts
/// - Success ratio
/// - First/last check timestamps
///
/// Checks with ambiguous or invalid IP flags are excluded and logged as errors.
///
/// # Errors
///
/// Returns [AnalysisError] if:
/// - Report formatting fails
/// - Check type analysis fails
///
/// # Warning Messages
///
/// Prints warning to stderr if:
/// - Check has both IPv4 and IPv6 flags set
/// - Check has no IP version flags set
fn gereric_ip_analyze(store: &Store, f: &mut String, ip_type: IpType) -> Result<(), AnalysisError> {
    let all: Vec<&Check> = store
        .checks()
        .iter()
        .filter(|c| c.ip_type() == ip_type)
        .collect();
    let successes: Vec<&Check> = all.clone().into_iter().filter(|c| c.is_success()).collect();
    analyze_check_type_set(f, &all, &successes)?;
    Ok(())
}
/// Includes metrics across all check types combined.
fn generic_type_analyze(
    store: &Store,
    f: &mut String,
    check_type: CheckType,
) -> Result<(), AnalysisError> {
    let all: Vec<&Check> = store
        .checks()
        .iter()
        .filter(|c| c.calc_type().unwrap_or(CheckType::Unknown) == check_type)
        .collect();
    let successes: Vec<&Check> = all.clone().into_iter().filter(|c| c.is_success()).collect();
    analyze_check_type_set(f, &all, &successes)?;
    Ok(())
}

/// Write store metadata section of the report.
///
/// Includes:
/// - Hash of in-memory data structure
/// - Hash of store file on disk
/// - Size of in memory [Store], including all children (the actual checks)
/// - Size of the [Store] file
/// - Ratio of [Store] file size and in memory [Store]
fn store_meta(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    let store_size_mem = store.deep_size_of();
    let store_size_fs = std::fs::metadata(Store::path())?.size();

    key_value_write(f, "Hash mem blake3", store.get_hash())?;
    key_value_write(f, "Hash file sha256", store.get_hash_of_file()?)?;
    key_value_write(f, "Store Version (mem)", store.version())?;
    key_value_write(f, "Store Version (file)", Store::peek_file_version()?)?;
    key_value_write(f, "Store Size (mem)", store_size_mem)?;
    key_value_write(f, "Store Size (file)", store_size_fs)?;
    key_value_write(
        f,
        "File to Mem Ratio",
        store_size_fs as f64 / store_size_mem as f64,
    )?;
    Ok(())
}

/// Calculate the success ratio of a subset compared to total.
///
/// Returns value between 0.0 and 1.0.
#[inline]
fn success_ratio(all_checks: usize, subset: usize) -> f64 {
    subset as f64 / all_checks as f64
}
