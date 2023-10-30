use crate::element::{Circle, DynElement, IntoDynElement, PathElement};
use crate::style::ShapeStyle;
use plotters_backend::DrawingBackend;
use std::marker::PhantomData;

/**
The line series object, which takes an iterator of data points in guest coordinate system
and creates appropriate lines and points with the given style.

# Example

```
use plotters::prelude::*;
let x_values = [0.0f64, 1., 2., 3., 4.];
let drawing_area = SVGBackend::new("line_series_point_size.svg", (300, 200)).into_drawing_area();
drawing_area.fill(&WHITE).unwrap();
let mut chart_builder = ChartBuilder::on(&drawing_area);
chart_builder.margin(10).set_left_and_bottom_label_area_size(20);
let mut chart_context = chart_builder.build_cartesian_2d(0.0..4.0, 0.0..3.0).unwrap();
chart_context.configure_mesh().draw().unwrap();
chart_context.draw_series(LineSeries::new(x_values.map(|x| (x, 0.3 * x)), BLACK)).unwrap();
chart_context.draw_series(LineSeries::new(x_values.map(|x| (x, 2.5 - 0.05 * x * x)), RED)
    .point_size(5)).unwrap();
chart_context.draw_series(LineSeries::new(x_values.map(|x| (x, 2. - 0.1 * x * x)), BLUE.filled())
    .point_size(4)).unwrap();
```

The result is a chart with three line series; two of them have their data points highlighted:

![](https://cdn.jsdelivr.net/gh/facorread/plotters-doc-data@64e0a28/apidoc/line_series_point_size.svg)
*/
pub struct LineSeries<DB: DrawingBackend, Coord> {
    style: ShapeStyle,
    data: Vec<Coord>,
    point_idx: usize,
    point_size: u32,
    phantom: PhantomData<DB>,
}

impl<DB: DrawingBackend, Coord: Clone + 'static> Iterator for LineSeries<DB, Coord> {
    type Item = DynElement<'static, DB, Coord>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.data.is_empty() {
            if self.point_size > 0 && self.point_idx < self.data.len() {
                let idx = self.point_idx;
                self.point_idx += 1;
                return Some(
                    Circle::new(self.data[idx].clone(), self.point_size, self.style)
                        .into_dyn(),
                );
            }
            let mut data = vec![];
            std::mem::swap(&mut self.data, &mut data);
            Some(PathElement::new(data, self.style).into_dyn())
        } else {
            None
        }
    }
}

impl<DB: DrawingBackend, Coord> LineSeries<DB, Coord> {
    /**
    Creates a new line series based on a data iterator and a given style.

    See [`LineSeries`] for more information and examples.
    */
    pub fn new<I: IntoIterator<Item = Coord>, S: Into<ShapeStyle>>(iter: I, style: S) -> Self {
        Self {
            style: style.into(),
            data: iter.into_iter().collect(),
            point_size: 0,
            point_idx: 0,
            phantom: PhantomData,
        }
    }

    /**
    Sets the size of the points in the series, in pixels.

    See [`LineSeries`] for more information and examples.
    */
    pub fn point_size(mut self, size: u32) -> Self {
        self.point_size = size;
        self
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn test_line_series() {
        let drawing_area = create_mocked_drawing_area(200, 200, |m| {
            m.check_draw_path(|c, s, _path| {
                assert_eq!(c, RED.to_rgba());
                assert_eq!(s, 3);
                // TODO when cleanup the backend coordinate defination, then we uncomment the
                // following check
                //for i in 0..100 {
                //    assert_eq!(path[i], (i as i32 * 2, 199 - i as i32 * 2));
                //}
            });

            m.drop_check(|b| {
                assert_eq!(b.num_draw_path_call, 1);
                assert_eq!(b.draw_count, 1);
            });
        });

        let mut chart = ChartBuilder::on(&drawing_area)
            .build_cartesian_2d(0..100, 0..100)
            .expect("Build chart error");

        chart
            .draw_series(LineSeries::new(
                (0..100).map(|x| (x, x)),
                Into::<ShapeStyle>::into(&RED).stroke_width(3),
            ))
            .expect("Drawing Error");
    }
}
