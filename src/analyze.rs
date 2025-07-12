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
use tracing::{debug, error, trace};

use crate::errors::AnalysisError;
use crate::records::{display_group, Check, CheckType, IpType};
use crate::store::{Store, OUTAGE_TIME_SPAN};

use std::collections::HashMap;
use std::fmt::{Display, Write};
use std::os::unix::fs::MetadataExt;

use self::outage::Outage;

pub mod outage;

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

/// This enum describes which ip address types should be considered
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Default)]
pub enum IpAddrConstraint {
    /// Any IP (does not matter)
    #[default]
    Any,
    /// Only V4
    V4,
    /// Only V6
    V6,
}

/// This struct is used to filter out [Checks](Check) that are not relevant
///
/// It is supposed to be used with [get_checks].
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord, Default)]
pub struct CheckAccessConstraints {
    /// Only consider failed checks
    pub failed_only: bool,
    /// Only consider checks of a certain ip type
    pub ip: IpAddrConstraint,
    /// Only consider checks made since a certain date (or ignore date if [`None`])
    pub since_date: Option<DateTime<Local>>,
    /// Only consider complete outages, not partial ones
    pub only_complete: bool,
}

impl IpAddrConstraint {
    /// check if an [IpAddr](std::net::IpAddr) matches and therefore should be filtered out
    pub fn ip_matches(&self, ip: &std::net::IpAddr) -> bool {
        if matches!(self, Self::Any) {
            return true;
        }
        match ip {
            std::net::IpAddr::V4(_) => matches!(self, Self::V4),
            std::net::IpAddr::V6(_) => matches!(self, Self::V6),
        }
    }
    /// check if an [IpType] matches and therefore should be filtered out
    pub fn ip_type_matches(&self, ip: &IpType) -> bool {
        if matches!(self, Self::Any) {
            return true;
        }
        match ip {
            IpType::V4 => matches!(self, Self::V4),
            IpType::V6 => matches!(self, Self::V6),
        }
    }
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
pub fn analyze(store: &Store, checks: &[&Check]) -> Result<String, AnalysisError> {
    let mut f = String::new();
    barrier(&mut f, "General")?;
    generalized(checks, &mut f)?;
    barrier(&mut f, "HTTP")?;
    generic_type_analyze(checks, &mut f, CheckType::Http)?;
    barrier(&mut f, "ICMP")?;
    generic_type_analyze(checks, &mut f, CheckType::Icmp)?;
    barrier(&mut f, "IPv4")?;
    gereric_ip_analyze(checks, &mut f, IpType::V4)?;
    barrier(&mut f, "IPv6")?;
    gereric_ip_analyze(checks, &mut f, IpType::V6)?;
    barrier(&mut f, "Outages")?;
    outages(checks, &mut f)?;
    barrier(&mut f, "Store Metadata")?;
    store_meta(store, &mut f)?;

    Ok(f)
}

/// Get all [Checks](Check) from a [Store] and filter out according to [CheckAccessConstraints]
pub fn get_checks(
    store: &Store,
    constraints: CheckAccessConstraints,
) -> Result<Vec<&Check>, AnalysisError> {
    debug!("Getting checks with the following constraints: {constraints:#?}");

    let checks: Vec<&Check> = store.checks().iter().collect();

    let checks: Vec<&Check> = if constraints.only_complete && constraints.failed_only {
        debug!("Processing outages because only complete outages should be considered");
        let outages = Outage::make_outages(checks.as_ref());

        fn is_in_outage(outages: &[Outage], check: &Check) -> bool {
            outages
                .binary_search_by(|outage| {
                    outage[0].timestamp_parsed().cmp(&check.timestamp_parsed())
                })
                .is_ok()
        }
        checks
            .into_iter()
            .filter(|c| is_in_outage(&outages, c))
            .collect()
    } else {
        checks
    };

    let checks: Vec<&Check> = checks
        .into_iter()
        .filter(|c| {
            (if constraints.failed_only && !constraints.only_complete {
                !c.is_success()
            } else {
                true
            }) && constraints.ip.ip_type_matches(&c.ip_type())
                && ({
                    if let Some(since_date) = constraints.since_date {
                        c.timestamp_parsed() >= since_date
                    } else {
                        true
                    }
                })
        })
        .collect();

    Ok(checks)
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
    writeln!(f, "{title:<24}: {content}")
}

/// Analyzes and formats outage information from the store.
///
/// Groups consecutive failed checks by check type and creates
/// Outage records for reporting.
fn outages(all: &[&Check], f: &mut String) -> Result<(), AnalysisError> {
    let fails_exist = !all.iter().all(|c| c.is_success());
    if !fails_exist || all.is_empty() {
        writeln!(f, "None\n")?;
        return Ok(());
    }

    let mut outages = Outage::make_outages(all);

    writeln!(f, "Latest\n")?;

    for (outage_idx, outage) in outages.iter().rev().enumerate() {
        writeln!(f, "{outage_idx}:\t{}", &outage.short_report()?)?;
        if outage_idx >= 9 {
            writeln!(f, "\nshowing only the 10 latest outages...\n")?;
            break;
        }
    }

    writeln!(f, "\nMost severe\n")?;

    outages.sort_by(Outage::cmp_severity);

    for (outage_idx, outage) in outages.iter().rev().enumerate() {
        writeln!(f, "{outage_idx}:\t{}", &outage.short_report()?)?;
        if outage_idx >= 9 {
            writeln!(f, "\nshowing only the 10 most severe outages...")?;
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
pub fn outages_detailed(all: &[&Check], f: &mut String, dump: bool) -> Result<(), AnalysisError> {
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
        let outage = Outage::try_from(group).expect("fail group was empty");
        writeln!(f, "{outage_idx}:\n{}", more_indent(&outage.to_string()))?;
        if dump {
            let mut buf = String::new();
            display_group(outage.all(), &mut buf)?;
            writeln!(f, "\tAll contained:\n{}", more_indent(&buf))?;
        }
    }
    writeln!(f)?;

    Ok(())
}

fn group_by_time<'check>(checks: &[&'check Check]) -> HashMap<i64, CheckGroup<'check>> {
    let mut groups: HashMap<i64, CheckGroup<'check>> = HashMap::new();

    for check in checks {
        groups.entry(check.timestamp()).or_default().push(check);
    }

    groups
}

pub(crate) fn fail_groups<'check>(checks: &[&'check Check]) -> Vec<CheckGroup<'check>> {
    trace!("calculating fail groups");
    let by_time = group_by_time(checks);
    let mut time_sorted_values: Vec<&Vec<&Check>> = by_time.values().collect();
    time_sorted_values.sort();
    let max_time_inbetween = chrono::TimeDelta::seconds(OUTAGE_TIME_SPAN);
    let mut continuous_outage_groups: Vec<Vec<Vec<&Check>>> = Vec::new();
    let mut group_first_time: DateTime<chrono::Local> = chrono::DateTime::UNIX_EPOCH.into();
    let mut group_current = Vec::new();
    let mut first;

    for time_group in time_sorted_values {
        first = time_group[0];
        if group_current.is_empty() {
            group_first_time = first.timestamp_parsed();
        }
        if first.timestamp_parsed() - group_first_time > max_time_inbetween {
            continuous_outage_groups.push(group_current.clone());
            group_current.clear();
        }
        group_current.push(time_group.clone());
    }

    continuous_outage_groups.sort();
    continuous_outage_groups
        .into_iter()
        .map(|v| v.into_iter().flatten().collect())
        .collect()
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
fn generalized(checks: &[&Check], f: &mut String) -> Result<(), AnalysisError> {
    if checks.is_empty() {
        writeln!(f, "no checks to analyze\n")?;
        return Ok(());
    }
    let all: Vec<&Check> = checks.to_vec();
    let successes: Vec<&Check> = checks.iter().copied().filter(|c| c.is_success()).collect();
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
fn gereric_ip_analyze(
    checks: &[&Check],
    f: &mut String,
    ip_type: IpType,
) -> Result<(), AnalysisError> {
    let all: Vec<&Check> = checks
        .iter()
        .copied()
        .filter(|c| c.ip_type() == ip_type)
        .collect();
    let successes: Vec<&Check> = all.clone().into_iter().filter(|c| c.is_success()).collect();
    analyze_check_type_set(f, &all, &successes)?;
    Ok(())
}
/// Includes metrics across all check types combined.
fn generic_type_analyze(
    checks: &[&Check],
    f: &mut String,
    check_type: CheckType,
) -> Result<(), AnalysisError> {
    let all: Vec<&Check> = checks
        .iter()
        .copied()
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

#[inline]
fn more_indent(buf: &str) -> String {
    format!("\t{}", buf.to_string().replace("\n", "\n\t"))
}

#[cfg(test)]
mod tests {

    use chrono::{Timelike, Utc};
    use tracing_test::traced_test;

    use crate::analyze::Outage;
    use crate::records::{Check, CheckFlag, TARGETS};

    use super::{fail_groups, group_by_time};

    #[rustfmt::skip]
    fn basic_check_set() -> Vec<Check>{
        let ip4 = TARGETS[0].parse().unwrap();
        let ip6 = TARGETS[1].parse().unwrap();
        let time = Utc::now().with_minute(0).unwrap();
        let time2 = Utc::now().with_minute(time.minute()+1).unwrap();
        let time3 = Utc::now().with_minute(time.minute()+2).unwrap();
        let time4 = Utc::now().with_minute(time.minute()+3).unwrap();
        let time5 = Utc::now().with_minute(time.minute()+4).unwrap();

        let mut a = vec![
            Check::new(time, CheckFlag::Success | CheckFlag::TypeHTTP, None, ip4),
            Check::new(time, CheckFlag::Success | CheckFlag::TypeIcmp, None, ip4),
            Check::new(time, CheckFlag::Success | CheckFlag::TypeHTTP, None, ip6),
            Check::new(time, CheckFlag::Success | CheckFlag::TypeIcmp, None, ip6),

            Check::new(time2, CheckFlag::Unreachable | CheckFlag::TypeHTTP, None, ip4),
            Check::new(time2, CheckFlag::Unreachable | CheckFlag::TypeIcmp, None, ip4),
            Check::new(time2, CheckFlag::Unreachable | CheckFlag::TypeHTTP, None, ip6),
            Check::new(time2, CheckFlag::Unreachable | CheckFlag::TypeIcmp, None, ip6),

            Check::new(time3, CheckFlag::Unreachable | CheckFlag::TypeHTTP, None, ip4),
            Check::new(time3, CheckFlag::Unreachable | CheckFlag::TypeIcmp, None, ip4),
            Check::new(time3, CheckFlag::Unreachable | CheckFlag::TypeHTTP, None, ip6),
            Check::new(time3, CheckFlag::Unreachable | CheckFlag::TypeIcmp, None, ip6),

            Check::new(time4, CheckFlag::Success | CheckFlag::TypeHTTP, None, ip4),
            Check::new(time4, CheckFlag::Success | CheckFlag::TypeIcmp, None, ip4),
            Check::new(time4, CheckFlag::Success | CheckFlag::TypeHTTP, None, ip6),
            Check::new(time4, CheckFlag::Success | CheckFlag::TypeIcmp, None, ip6),

            Check::new(time5, CheckFlag::Unreachable | CheckFlag::TypeHTTP, None, ip4),
            Check::new(time5, CheckFlag::Unreachable | CheckFlag::TypeIcmp, None, ip4),
            Check::new(time5, CheckFlag::Unreachable | CheckFlag::TypeHTTP, None, ip6),
            Check::new(time5, CheckFlag::Unreachable | CheckFlag::TypeIcmp, None, ip6),
        ]    ;
        a.sort();
        a
    }

    #[test]
    #[traced_test]
    fn test_fail_groups() {
        let base_checks = basic_check_set();
        let checks: Vec<&Check> = base_checks.iter().collect();

        // fail_groups has been non deterministic in the past, because of not-sorting
        for _ in 0..40 {
            let fg = fail_groups(&checks);
            assert_eq!(fg.len(), 2);
            assert_eq!(fg[0].len(), 8);
            assert_eq!(fg[1].len(), 4);

            let _outages = [
                Outage::try_from(fg[0].clone()).unwrap(),
                Outage::try_from(fg[1].clone()).unwrap(),
            ];
        }
    }

    #[test]
    #[traced_test]
    fn test_group_by_time() {
        let base_checks = basic_check_set();
        let checks: Vec<&Check> = base_checks.iter().collect();

        let tg = group_by_time(&checks);
        assert_eq!(tg.len(), 5);
        for (k, v) in tg {
            assert_eq!(v.len(), 4);
            for c in v {
                assert_eq!(k, c.timestamp())
            }
        }
    }
}
