use crate::errors::AnalysisError;
use crate::records::{Check, CheckType};
use crate::store::Store;

use std::fmt::{Display, Write};
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Outage<'check> {
    /// Check that started the [Outage]
    start: &'check Check,
    /// Last [Check] the [Outage], after this it works again
    end: Option<&'check Check>,
    /// All failed [Checks](Check) in this [Outage]
    all: Vec<&'check Check>,
}

impl<'check> Outage<'check> {
    pub(crate) fn new(
        start: &'check Check,
        end: Option<&'check Check>,
        all_checks: &[&'check Check],
    ) -> Self {
        Self {
            start,
            end,
            all: all_checks.to_vec(),
        }
    }
}

impl Display for Outage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.end.is_some() {
            writeln!(
                f,
                "From {} To {}",
                humantime::format_rfc3339_seconds(self.start.timestamp_parsed()),
                humantime::format_rfc3339_seconds(self.end.unwrap().timestamp_parsed())
            )?;
        } else {
            writeln!(
                f,
                "From {} STILL ONGOING",
                humantime::format_rfc3339_seconds(self.start.timestamp_parsed()),
            )?;
        }
        writeln!(f, "Checks: {}", self.all.len())?;
        writeln!(f, "Type: {}", self.start.calc_type())?;
        Ok(())
    }
}

/// Display a group of [Checks](Check)
pub fn display_group(group: &[&Check], f: &mut String) -> Result<(), AnalysisError> {
    if group.is_empty() {
        writeln!(f, "\t<Empty>")?;
        return Ok(());
    }
    for (cidx, check) in group.iter().enumerate() {
        writeln!(f, "{cidx}:")?;
        writeln!(f, "\t{}", check.to_string().replace("\n", "\n\t"))?;
    }
    Ok(())
}

/// Forge a Report about the [Checks](Check) in the given [Store].
pub fn analyze(store: &Store) -> Result<String, AnalysisError> {
    let mut f = String::new();
    barrier(&mut f, "General")?;
    generalized(store, &mut f)?;
    barrier(&mut f, "HTTP")?;
    http(store, &mut f)?;
    barrier(&mut f, "Outages")?;
    outages(store, &mut f)?;
    barrier(&mut f, "Store Metadata")?;
    store_meta(store, &mut f)?;

    Ok(f)
}

fn barrier(f: &mut String, title: &str) -> Result<(), AnalysisError> {
    writeln!(f, "{:=<10}{:=<90}", "", format!(" {title} "))?;
    Ok(())
}

fn key_value_write(
    f: &mut String,
    title: &str,
    content: impl Display,
) -> Result<(), std::fmt::Error> {
    writeln!(f, "{:<20}: {:<78}", title, content.to_string())
}

fn outages(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    let all_checks: Vec<&Check> = store.checks().iter().collect();
    let mut outages: Vec<Outage> = Vec::new();
    let fails_exist = all_checks
        .iter()
        .fold(true, |fails_exist, c| fails_exist & !c.is_success());
    if !fails_exist {
        writeln!(f, "None\n")?;
        return Ok(());
    }

    for check_type in CheckType::all() {
        let checks: Vec<&&Check> = all_checks
            .iter()
            .filter(|c| c.calc_type() == *check_type)
            .collect();

        let fail_groups = fail_groups(&checks);
        for group in fail_groups {
            // writeln!(f, "Group {gidx}:")?;
            // display_group(group, f)?;
            if !group.is_empty() {
                outages.push(Outage::new(
                    checks.first().unwrap(),
                    Some(checks.last().unwrap()),
                    &group,
                ));
            }
        }
    }

    for outage in outages {
        writeln!(f, "{outage}")?;
    }
    Ok(())
}

fn fail_groups<'check>(checks: &[&&'check Check]) -> Vec<Vec<&'check Check>> {
    let failed_idxs: Vec<usize> = checks
        .iter()
        .enumerate()
        .filter(|(_idx, c)| !c.is_success())
        .map(|(idx, _c)| idx)
        .collect();
    if failed_idxs.is_empty() {
        return Vec::new();
    }
    let mut groups: Vec<Vec<&Check>> = Vec::new();

    let mut first = failed_idxs[0];
    let mut last = first;
    for idx in failed_idxs {
        if idx == last + 1 {
            last = idx;
        } else {
            let mut group: Vec<&Check> = Vec::new();
            for check in checks.iter().take(last + 1).skip(first) {
                group.push(*check);
            }
            groups.push(group);

            first = idx;
        }
    }

    groups
}

fn analyze_check_type_set(
    f: &mut String,
    all: &Vec<&Check>,
    successes: &Vec<&Check>,
) -> Result<(), AnalysisError> {
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
        humantime::format_rfc3339_seconds(all.first().unwrap().timestamp_parsed()),
    )?;
    key_value_write(
        f,
        "last check at",
        humantime::format_rfc3339_seconds(all.last().unwrap().timestamp_parsed()),
    )?;
    writeln!(f)?;
    Ok(())
}

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

fn http(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    let all: Vec<&Check> = store
        .checks()
        .iter()
        .filter(|c| c.calc_type() == CheckType::Http)
        .collect();
    let successes: Vec<&Check> = store.checks().iter().filter(|c| c.is_success()).collect();
    analyze_check_type_set(f, &all, &successes)?;
    Ok(())
}

fn store_meta(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    key_value_write(f, "Hash Datastructure", store.display_hash())?;
    key_value_write(f, "Hash Store File", store.display_hash_of_file()?)?;
    Ok(())
}

#[inline]
fn success_ratio(all_checks: usize, subset: usize) -> f64 {
    subset as f64 / all_checks as f64
}
