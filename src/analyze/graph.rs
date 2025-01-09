use std::path::Path;

use chrono::{DateTime, Local, TimeZone, Utc};
use plotters::prelude::*;

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

    for group in time_grouped.iter().map(|(k, v)| v) {
        data.push((
            group[0].timestamp_parsed(),
            Severity::from(group.as_slice()),
        ));
    }

    let root = BitMapBackend::new(outfile, (1024, 768)).into_drawing_area();
    let timespan =
        times.first().unwrap()[0].timestamp_parsed()..times.last().unwrap()[0].timestamp_parsed();

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(
            "Monthly Average Temperate in Salt Lake City, UT",
            ("sans-serif", 40),
        )
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d((timespan.clone()).monthly(), 14.0..104.0)
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?
        .set_secondary_coord(timespan.monthly(), -10.0..40.0);

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(30)
        .max_light_lines(4)
        .y_desc("Average Temp (F)")
        .draw()
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;
    chart
        .configure_secondary_axes()
        .y_desc("Average Temp (C)")
        .draw()
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;

    let processed_data = data.iter().map(|(a, b)| (*a, f64::from(*b)));
    chart
        .draw_series(LineSeries::new(processed_data, &BLUE))
        .map_err(|e| AnalysisError::GraphDraw {
            reason: e.to_string(),
        })?;

    // chart
    //     .draw_series(
    //         DATA.iter()
    //             .map(|(y, m, t)| Circle::new((Local.ymd(*y, *m, 1), *t), 3, BLUE.filled())),
    //     )
    //     .map_err(|e| AnalysisError::GraphDraw {
    //         reason: e.to_string(),
    //     })?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().map_err(|e| AnalysisError::GraphDraw {
        reason: e.to_string(),
    })?;
    Ok(())
}
