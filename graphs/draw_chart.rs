use plotters::coord::types::{RangedCoordf32, RangedCoordusize};
use plotters::coord::Shift;
use plotters::prelude::*;

pub fn draw_chart_series<T, F: Fn((usize, &T)) -> (usize, f32)>(
    chart: &mut ChartContext<SVGBackend, Cartesian2d<RangedCoordusize, RangedCoordf32>>,
    y_data: &Vec<T>,
    map_fn: F,
    color_idx: usize,
    label: &str,
) {
    let style = Palette99::pick(color_idx).mix(0.9).stroke_width(3);

    chart
        .draw_series(LineSeries::new(
            y_data.iter().enumerate().map(map_fn),
            style,
        ))
        .expect("cannot draw series")
        .label(label)
        .legend(move |(x, y)| Rectangle::new([(x, y - 5), (x + 10, y + 5)], style.filled()));
}

pub fn draw_chart<T, F: Fn((usize, &T)) -> (usize, f32)>(
    drawing_area: &DrawingArea<SVGBackend, Shift>,
    y_data: &Vec<T>,
    y_data_max: f32,
    caption: &str,
    x_desc: &str,
    y_desc: &str,
    label: &str,
    map_fn: F,
    color_idx: usize,
    label_size: i32,
) {
    let mut chart = ChartBuilder::on(drawing_area)
        .caption(caption, ("sans-serif", 5.percent_height()))
        .set_label_area_size(LabelAreaPosition::Left, (label_size * 2).percent())
        .set_label_area_size(LabelAreaPosition::Bottom, label_size.percent())
        .margin(10.percent())
        .build_cartesian_2d(0..y_data.len(), 0.0..y_data_max * 1.05)
        .expect("cannot build cartesian 2d");

    chart
        .configure_mesh()
        .bold_line_style(&BLACK.mix(0.25)) // big grid lines
        .light_line_style(&TRANSPARENT) // little grid lines
        .x_desc(x_desc)
        .y_desc(y_desc)
        .axis_desc_style(("sans-serif", 5.percent_height()))
        .draw()
        .expect("cannot draw chart (before series)");

    draw_chart_series(&mut chart, y_data, map_fn, color_idx, label);

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        // .position(SeriesLabelPosition::UpperRight)
        // .label_font(("sans-serif", 5.percent_height()))
        .draw()
        .expect("cannot draw chart (after series)");
}
