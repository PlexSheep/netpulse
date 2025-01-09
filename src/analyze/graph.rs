use std::fmt::Write;
use std::path::Path;

use chrono::{DateTime, Local, TimeZone, Utc};
use plotters::prelude::*;
use tracing::trace;

use crate::errors::AnalysisError;
use crate::records::Check;

use super::group_by_time;
use super::outage::Severity;

pub fn draw_checks(checks: &[Check], file: impl AsRef<Path>) -> Result<(), AnalysisError> {
    if checks.is_empty() {
        panic!("need at least one check to draw the diagram");
    }
    let outfile: &Path = file.as_ref();
    let mut data: Vec<(DateTime<Local>, Severity)> = Vec::new();
    let time_grouped = group_by_time(checks.iter());
    assert!(!time_grouped.is_empty());
    let mut times: Vec<_> = time_grouped.values().collect();
    times.sort();
    let timespan =
        times.first().unwrap()[0].timestamp_parsed()..times.last().unwrap()[0].timestamp_parsed();
    let mut x_axis: Vec<_> = Vec::new();
    for t in 0..time_grouped.len() {
        x_axis.push(t);
    }

    for group in time_grouped.iter().map(|(k, v)| v) {
        data.push((
            group[0].timestamp_parsed(),
            Severity::from(group.as_slice()),
        ));
    }
    data.sort_by_key(|a| a.0);

    let root = BitMapBackend::new(outfile, (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).map_err(|e| AnalysisError::GraphDraw {
        reason: e.to_string(),
    })?;

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption("Outage Severity over all time", ("sans-serif", 60))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 30)
        .build_cartesian_2d(timespan, 0.0..1.0)
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;

    chart
        .configure_mesh()
        .x_labels(10)
        .max_light_lines(4)
        .y_desc("Severity")
        .x_desc("Time")
        .draw()
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;

    let processed_data = data.iter().map(|(a, b)| (*a, f64::from(*b)));
    trace!("dumping whole processed data: \n{}", {
        let mut buf = String::new();
        for (idx, row) in processed_data.clone().enumerate() {
            writeln!(buf, "{:08},{},{:.06}", idx, row.0, row.1).unwrap();
        }
        buf
    });
    chart
        .draw_series(AreaSeries::new(processed_data, 0.0, RED.mix(0.2)).border_style(RED))
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().map_err(|e| AnalysisError::GraphDraw {
        reason: e.to_string(),
    })?;
    Ok(())
}
