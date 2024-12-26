use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::RelativeSize;

/// Produce un diagramma cartesiano delle quantità di un certo GoodKind:
/// - L'asse x del diagramma rappresenta le iterazioni della simulazione.
/// - L'asse y del diagramma rappresenta la quantità del GoodKind.
pub fn make_chart(quantities: impl Iterator<Item=f32> + Clone, drawing_area: &DrawingArea<SVGBackend, Shift>, caption: &str) {
    // La lunghezza dell'asse x è pari al numero di iterazioni (-1, in quanto la prima ha indice 0)
    let x_axis_len = quantities.clone().count() as f32 - 1.;

    // L'altezza dell'asse y è pari al valore più alto di quantità del GoodKind
    let y_min = quantities.clone().fold(f32::MAX, f32::min);
    let y_max = quantities.clone().fold(0., f32::max);

    let mut chart = ChartBuilder::on(drawing_area)
        .caption(caption, ("sans-serif", 20))
        .x_label_area_size(20)
        .y_label_area_size(80)
        .margin(20)
        .build_cartesian_2d(0.0..x_axis_len, y_min..y_max)
        .unwrap();

    chart
        .configure_mesh()
        .bold_line_style(BLACK.mix(0.25)) // big grid lines
        .light_line_style(TRANSPARENT) // little grid lines
        .draw()
        .unwrap();

    let series = LineSeries::new(quantities.enumerate().map(|(iteration, quantity)| (iteration as f32, quantity)), RED);

    chart.draw_series(series).unwrap();
}

#[test]
fn test_make_chart() {
    let quantities = [35.44, 73.21, 84.699, 220.2, 23.6, 14.0];

    let root = SVGBackend::new("make_chart_for_kind_test.svg", (1920, 1080)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    make_chart(quantities.into_iter(), &root, "EUR quantity variation");

    root.present().unwrap();
}