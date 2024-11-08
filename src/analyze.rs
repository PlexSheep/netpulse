use crate::errors::AnalysisError;
use crate::records::{Check, CheckType};
use crate::store::Store;

use std::fmt::{Display, Write};

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

    Ok(f)
}

fn barrier(f: &mut String, title: &str) -> Result<(), AnalysisError> {
    writeln!(f, "{:=<10}{:=<70}", "", format!(" {title} "))?;
    Ok(())
}

fn outages(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    let all_checks: Vec<&Check> = store.checks().iter().collect();
    let mut outages: Vec<Outage> = Vec::new();
    let fails_exist = all_checks
        .iter()
        .fold(true, |fails_exist, c| fails_exist & !c.is_success());
    if !fails_exist {
        writeln!(f, "No outages")?;
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

fn generalized(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    let checks: Vec<&Check> = store.checks().iter().collect();
    let successes: Vec<&Check> = checks
        .clone()
        .into_iter()
        .filter(|c| c.is_success())
        .collect();
    writeln!(f, "store contains {:09} checks.", checks.len())?;
    writeln!(
        f,
        "store contains {:09} successful checks.",
        successes.len()
    )?;
    writeln!(
        f,
        "success ratio: {:02.02}%",
        success_ratio(&checks, &successes) * 100.0
    )?;
    writeln!(f)?;
    Ok(())
}

fn http(store: &Store, f: &mut String) -> Result<(), AnalysisError> {
    let checks: Vec<&Check> = store
        .checks()
        .iter()
        .filter(|c| c.calc_type() == CheckType::Http)
        .collect();
    let successes: Vec<&Check> = checks
        .clone()
        .into_iter()
        .filter(|c| c.is_success())
        .collect();
    writeln!(f, "store contains {:09} HTTP checks.", checks.len())?;
    writeln!(
        f,
        "store contains {:09} successful HTTP checks.",
        successes.len()
    )?;
    writeln!(
        f,
        "success ratio: {:02.02}%",
        success_ratio(&checks, &successes) * 100.0
    )?;
    writeln!(f)?;
    Ok(())
}

#[inline]
fn success_ratio(all_checks: &[&Check], subset: &[&Check]) -> f64 {
    subset.len() as f64 / all_checks.len() as f64
}
