use std::path::Path;

use chrono::{DateTime, Local, TimeZone};
use plotters::element::Drawable;
use plotters::prelude::*;

use crate::errors::AnalysisError;
use crate::records::Check;

use super::group_by_time;
use super::outage::Severity;

pub fn draw_checks(checks: &[Check], file: impl AsRef<Path>) -> Result<(), AnalysisError> {
    if checks.is_empty() {
        return Err(AnalysisError::NoChecksToAnalyze);
    }
    let outfile: &Path = file.as_ref();
    let mut data: Vec<(DateTime<Local>, Severity)> = Vec::new();
    let time_grouped = group_by_time(checks.iter());
    assert!(!time_grouped.is_empty());
    let mut times: Vec<_> = time_grouped.values().collect();
    times.sort();
    let timespan =
        times.first().unwrap()[0].timestamp_parsed()..times.last().unwrap()[0].timestamp_parsed();

    for group in time_grouped.values() {
        data.push((
            group[0].timestamp_parsed(),
            Severity::from(group.as_slice()),
        ));
    }
    data.sort_by_key(|a| a.0);

    let cpt = super::checks_per_time_group(checks.iter());
    let mut checks_per_time: Vec<(DateTime<Local>, usize)> = cpt
        .iter()
        .map(|(k, v)| (Local.timestamp_opt(*k, 0).unwrap(), *v))
        .collect();
    checks_per_time.sort_by_key(|a| a.0);

    let root = BitMapBackend::new(outfile, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| AnalysisError::GraphDraw {
        reason: e.to_string(),
    })?;

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("Outage Severity over all time", ("sans-serif", 60))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .build_cartesian_2d(timespan.clone(), 0.0..1.0)
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?
        .set_secondary_coord(
            timespan,
            0f64..checks_per_time.last().map(|a| a.1 as f64).unwrap(),
        );

    chart
        .configure_mesh()
        .x_labels(10)
        .max_light_lines(4)
        .y_desc("Severity (Red)")
        .x_desc("Time")
        .draw()
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;
    chart
        .configure_secondary_axes()
        .y_desc("Amount of Checks (Blue)")
        .draw()
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;

    let processed_serevity_data = data.iter().map(|(a, b)| (*a, f64::from(*b)));
    chart
        .draw_series(AreaSeries::new(processed_serevity_data, 0.0, RED.mix(0.2)).border_style(RED))
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;

    let checks_per_time_f64 = checks_per_time.into_iter().map(|(t, v)| (t, v as f64));
    chart
        .draw_series(AreaSeries::new(checks_per_time_f64, 0.0, BLUE.mix(0.2)))
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;
    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().map_err(|e| AnalysisError::GraphDraw {
        reason: e.to_string(),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::analyze::{graph::draw_checks, testset::default_dataset};

    #[test]
    fn test_draw_default_dataset() {
        let virtual_store = default_dataset();
        draw_checks(
            virtual_store.checks(),
            "./examples/media/severity_over_time_default_dataset.png",
        )
        .expect("could not draw default dataset");
    }
}
