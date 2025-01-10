use std::path::Path;

use charming::{
    component::{Axis, Legend},
    element::AxisType,
    series::Line,
    Chart, HtmlRenderer,
};
use chrono::{DateTime, Local, TimeZone};

use crate::errors::AnalysisError;
use crate::records::Check;

use super::group_by_time;
use super::outage::Severity;

pub fn draw_outages_over_all_time(
    checks: &[Check],
    file: impl AsRef<Path>,
) -> Result<(), AnalysisError> {
    if checks.is_empty() {
        return Err(AnalysisError::NoChecksToAnalyze);
    }
    let outfile: &Path = file.as_ref();
    let time_grouped = group_by_time(checks.iter());
    let mut times: Vec<_> = time_grouped.values().collect();
    let mut severity_data: Vec<(DateTime<Local>, Severity)> = Vec::new();

    times.sort();
    let timespan =
        times.first().unwrap()[0].timestamp_parsed()..times.last().unwrap()[0].timestamp_parsed();

    for group in time_grouped.values() {
        severity_data.push((
            group[0].timestamp_parsed(),
            Severity::from(group.as_slice()),
        ));
    }
    severity_data.sort_by_key(|a| a.0);

    let cpt = super::checks_per_time_group(checks.iter());
    let mut checks_per_time: Vec<(DateTime<Local>, usize)> = cpt
        .iter()
        .map(|(k, v)| (Local.timestamp_opt(*k, 0).unwrap(), *v))
        .collect();
    checks_per_time.sort_by_key(|a| a.0);

    let chart = chart_a(&checks_per_time, &severity_data);
    let mut renderer = HtmlRenderer::new("test", 1600, 900);
    renderer.save(&chart, outfile)?;

    Ok(())
}

pub fn chart_a(
    checks_per_time: &Vec<(DateTime<Local>, usize)>,
    severity_data: &Vec<(DateTime<Local>, Severity)>,
) -> Chart {
    let times: Vec<DateTime<Local>> = checks_per_time.iter().map(|a| a.0).collect();
    let mut severities: Vec<Vec<String>> = Vec::new();
    let severity_times: Vec<DateTime<Local>> = severity_data.iter().map(|a| a.0).collect();

    let mut severities_idx = 0;
    for time in times.iter() {
        if severity_times.contains(time) {
            let point = severity_data[severities_idx];
            severities.push(vec![point.0.to_rfc3339(), point.1.raw().to_string()]);
            severities_idx += 1;
        } else {
            severities.push(vec![time.to_rfc3339(), 0.0.to_string()]);
        }
    }
    Chart::new()
        .legend(Legend::new())
        .y_axis(Axis::new().type_(AxisType::Value))
        .x_axis(Axis::new().type_(AxisType::Time))
        .series(Line::new().data(severities))
}

#[cfg(test)]
mod tests {
    use crate::analyze::{graph::draw_outages_over_all_time, testset::default_dataset};

    #[test]
    fn test_draw_default_dataset() {
        let virtual_store = default_dataset();
        draw_outages_over_all_time(
            virtual_store.checks(),
            "./examples/media/severity_over_time_default_dataset.html",
        )
        .expect("could not draw default dataset");
    }
}
