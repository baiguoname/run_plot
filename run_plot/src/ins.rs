use std::ops::Range;

use crate::prelude::*;
use crate::hover_cpn::RecordBox;
use plotters::prelude::*;

// pub mod my_line {

//     use super::*;

//     pub struct MyLine(pub Vec<f32>, pub Vec<f32>);

//     impl InitFig for MyLine {
//         type X = f32;
//         type Y = f32;
//         type CoorX = f32;
//         type CoorY = f32;
//         type Xaxis = Rcf32;
//         type Yaxis = Rcf32;

//         fn get_xaxis(&self) -> Self::Xaxis {
//             (self.0[0] .. *self.0.last().unwrap()).into()
//         }

//         fn get_yaxis(&self) -> Self::Yaxis {
//             (self.1[0] .. *self.1.last().unwrap()).into()
//         }

//         fn init_fig<'a, DB: DrawingBackend>(
//             &self,
//             root: &'a DrawingArea<DB, Shift>
//         ) -> ChartContext<'a, DB, Cts<Self::Xaxis, Self::Yaxis>>
//         {
//             let mut chart = ChartBuilder::on(root)
//                 .margin(20i32)
//                 .x_label_area_size(30)
//                 .y_label_area_size(70)
//                 .build_cartesian_2d(self.get_xaxis(), self.get_yaxis())
//                 .unwrap();
//             chart
//                 .configure_mesh()
//                 .disable_x_mesh()
//                 .disable_y_mesh()
//                 .draw()
//                 .unwrap();
//             let init_line = LineSeries::new(
//                 self.0
//                     .iter()
//                     .zip(self.1.iter())
//                     .map(|(x, y)| (*x, *y)),
//                 &RED
//             );
//             // chart.draw_secondary_se
//             chart.draw_series(init_line).unwrap();
//             chart
//         }

//         fn hover_point(
//             &self,
//             posi: (Self::CoorX, Self::CoorY),
//         ) -> Circle<(Self::CoorX, Self::CoorY), i32> {
//             let posi = self.get_xy(posi).unwrap();
//             Circle::new(posi, 5i32, colors::BLUE.filled())
//         }

//         fn hover_axis<DB: DrawingBackend>(
//             &self,
//             posi: (Self::X, Self::Y),
//             bounds: (Self::CoorX, Self::CoorY),
//         ) -> LineSeries<DB, (Self::CoorX, Self::CoorY)> {
//             let posi = self.get_xy(posi).unwrap();
//             LineSeries::new([(bounds.0, posi.1), posi, (posi.0, bounds.1)], &RED)
//         }

//         fn hover_text(
//             &self,
//             coor: (Self::CoorX, Self::CoorY),
//             dim: (u32, u32),
//         ) -> MultiLineText<(Self::CoorX, Self::CoorY), String> {
//             let coor = self.get_xy(coor).unwrap();
//             let style = ("Consolas", RelativeSize::Smaller(0.02)).into_text_style(&dim);
//             let mut multi_line = MultiLineText::new(
//                 coor,
//                 style,
//             );
//             multi_line.push_line(format!("x: {}", coor.0));
//             multi_line.push_line(format!("y: {}", coor.1));
//             multi_line
//         }
//     }
// }

pub mod my_kline {

    

    use super::*;
    use chrono::NaiveDateTime as dt;
    use plotters::coord::{
        ranged1d::{DefaultFormatting, KeyPointHint},
        ReverseCoordTranslate,
    };
    use std::{cell::RefCell, ops::Range};

    #[derive(Debug, Clone)]
    pub struct MyAxis<T>(pub Vec<T>);

    impl<T> Ranged for MyAxis<T>
    where
        T: PartialOrd + Clone,
    {
        type FormatOption = DefaultFormatting;
        type ValueType = T;

        fn range(&self) -> Range<Self::ValueType> {
            self.0.first().unwrap().clone()..self.0.last().unwrap().clone()
        }

        fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
            let g = &self.0;
            let a = (g.iter().position(|v| v >= value).unwrap_or_default() as f64) / (g.len() as f64);
            limit.0 + ((a * f64::from(limit.1 - limit.0)) as i32)
        }
        fn key_points<Hint: KeyPointHint>(&self, _hint: Hint) -> Vec<Self::ValueType> {
            let step_size = (self.0.len() / 10).max(1);
            (0..self.0.len())
                .step_by(step_size)
                .fold(vec![], |mut accu, i| {
                    accu.push(self.0[i].clone());
                    accu
                })
        }
    }

    impl<T> DiscreteRanged for MyAxis<T>
    where
        T: PartialOrd + Clone,
        MyAxis<T>: Ranged<ValueType = T>,
    {
        fn size(&self) -> usize {
            self.0.len()
        }

        fn index_of(&self, value: &Self::ValueType) -> Option<usize> {
            self.0.iter().position(|x| value >= x)
        }

        fn from_index(&self, index: usize) -> Option<Self::ValueType> {
            self.0.get(index).cloned()
        }
    }

    pub struct MyKline(pub Vec<dt>, pub Vec<[f32; 4]>, RefCell<Range<usize>>);

    impl MyKline {
        pub fn new(x: Vec<dt>, y: Vec<[f32; 4]>) -> Self {
            let l = x.len();
            MyKline(x, y, RefCell::new((l - 30)..l))
        }

        fn get_zoom_bounds(&self, coor: PosiPixcel, zoom_ratio: f32, w: i32) -> Range<usize> {
            let r = self.2.borrow();
            let l = self.0.len() as f32;
            let percent_x = (coor.0 as f32 - 20.) / (w as f32 - 40.);
            let zoom_len = (r.end - r.start) as f32;
            let pinned_x_i = r.start + (zoom_len * percent_x) as usize;
            let mut start_step = ((pinned_x_i - r.start) as f32 * zoom_ratio.abs()) as usize;
            let mut end_step = ((r.end - pinned_x_i) as f32 * zoom_ratio.abs()) as usize;
            if zoom_ratio != 0. {
                start_step = start_step.max(3);
                end_step = end_step.max(3);
            }
            let (start, end) = if zoom_ratio > 0. {
                let start = (r.start + start_step).min(pinned_x_i - 5);
                let end = (r.end - end_step.min(r.end)).max(pinned_x_i + 5);
                (start, end)
            } else {
                let start = r.start - start_step.min(r.start);
                let end = (r.end + end_step).min(l as usize);
                (start, end)
            };
            start .. end
        }

        fn get_drag_bounds(&self, posi1: PosiPixcel, posi2: PosiPixcel, _w: i32) -> Range<usize> {
            let r = self.2.borrow();
            let l = self.0.len() as f32;
            // let draged_len = posi2.0 - posi1.0;
            // let draged_percent = draged_len as f32 / (w as f32 - 40.);
            let draged_percent = if posi2.0 >= posi1.0 { 0.03 } else { -0.03 };
            let size_now = r.end - r.start;
            let draged_size_f32 = draged_percent * (size_now as f32);
            let draged_size = (draged_size_f32.abs() as usize / 2).max(1);
            let (start, end) = if draged_size_f32 > 0. {
                let start = r.start - draged_size.min(r.start);
                let end = (r.end - draged_size.min(r.end)).max(5);
                (start, end)
            } else {
                let start = (r.start + draged_size).min(l as usize - 5);
                let end = (r.end + draged_size).min(l as usize);
                (start, end)
            };
            start .. end
        }

        fn plot_chart<'a, DB: DrawingBackend>(
            &self, 
            root: &'a DrawingArea<DB, Shift>
        ) -> CC<'a, DB, MyAxis<dt>, Rcf32>
        {
            let w = root.dim_in_pixel().0 as i32;
            let r = self.2.borrow();
            let x_axis = MyAxis(self.0[r.clone()].to_vec());
            let y_data = &self.1[r.clone()];
            let y_axis = get_range_from_k(y_data);
            let width = w as u32 / ((x_axis.0.len() as f32 * 1.5) as u32).max(1);
            let init_line = x_axis
                .0
                .iter()
                .zip(y_data.iter())
                .map(|(x, y)| {
                    CandleStick::new(
                        *x,
                        y[0],
                        y[1],
                        y[2],
                        y[3],
                        RED.filled(),
                        GREEN.filled(),
                        width,
                    )
                })
                .collect::<Vec<_>>();
            let mut chart = ChartBuilder::on(root)
                .margin(10)
                .x_label_area_size(10)
                .y_label_area_size(10)
                .build_cartesian_2d(x_axis, y_axis)
                .unwrap();
            chart
                .configure_mesh()
                .disable_x_mesh()
                .disable_y_mesh()
                .draw()
                .unwrap();
            chart.draw_series(init_line).unwrap();
            chart
        }
    }

    impl InitFig for MyKline {
        type X = dt;
        type Y = [f32; 4];
        type CoorX = dt;
        type CoorY = f32;
        type Xaxis = MyAxis<dt>;
        type Yaxis = Rcf32;

        fn get_xaxis(&self) -> Self::Xaxis {
            MyAxis(self.0[self.2.borrow().clone()].to_vec())
        }

        fn get_yaxis(&self) -> Self::Yaxis {
            get_range_from_k(&self.1[self.2.borrow().clone()]).into()
        }

        fn get_xy(&self, posi: (Self::CoorX, Self::CoorY)) -> Option<(Self::X, Self::Y)> {
            let x = self.0.iter().position(|x| x >= &posi.0)?;
            let y = self.1[x];
            (posi.0, y).into()
        }

        fn zoom_fig<'a, DB: DrawingBackend>(
            &self,
            root: &'a DrawingArea<DB, Shift>,
            rect: RecordBox,
        ) -> Option<CC<'a, DB, Self::Xaxis, Self::Yaxis>> {
            let w = root.dim_in_pixel().0 as i32;
            *self.2.borrow_mut() = self.get_zoom_bounds(rect.zoom.0, rect.zoom.1, w);
            if let Some((posi1, posi2)) = rect.drag_fig.into_posi_pixcel() {
                *self.2.borrow_mut() = self.get_drag_bounds(posi1, posi2, w);
            }
            let chart = self.plot_chart(root);
            Some(chart)
        }

        fn hover_point(
            &self,
            posi: (Self::CoorX, Self::CoorY),
        ) -> Option<Circle<(Self::CoorX, Self::CoorY), i32>> {
            let (x, y) = self.get_xy(posi)?;
            Circle::new((x, (y[0] + y[3]) / 2.0), 2i32, colors::BLUE.filled()).into()
        }

        fn hover_axis<DB: DrawingBackend>(
            &self,
            posi: (Self::CoorX, Self::CoorY),
            bounds: (Self::CoorX, Self::CoorY),
        ) -> Option<LineSeries<DB, (Self::CoorX, Self::CoorY)>> {
            let (x, y) = self.get_xy(posi)?;
            let y = (y[0] + y[3]) / 2.0;
            LineSeries::new([(bounds.0, y), (x, y), (x, bounds.1)], &RED).into()
        }

        fn hover_text(
            &self,
            coor: (Self::CoorX, Self::CoorY),
            dim: (u32, u32),
        ) -> Option<MultiLineText<(Self::CoorX, Self::CoorY), String>> {
            let xy = self.get_xy(coor)?;
            let style = ("Consolas", RelativeSize::Smaller(0.02)).into_text_style(&dim);
            let mut multi_line = MultiLineText::new((xy.0, (xy.1[0] + xy.1[3]) / 2.0), style);
            multi_line.push_line(format!("{}", xy.0));
            multi_line.push_line(format!("open  : {}", xy.1[0]));
            multi_line.push_line(format!("high  : {}", xy.1[1]));
            multi_line.push_line(format!("low   : {}", xy.1[2]));
            multi_line.push_line(format!("close : {}", xy.1[3]));
            multi_line.into()
        }

        fn add_box<DB: DrawingBackend>(
            &self,
            chart: &mut ChartContext<DB, Cts<Self::Xaxis, Self::Yaxis>>,
            rect: RecordBox,
        ) -> Option<()>
        where
            Cts<Self::Xaxis, Self::Yaxis>: ReverseCoordTranslate<From = (Self::CoorX, Self::CoorY)>,
        {
            let f = chart.plotting_area().as_coord_spec();
            let (posi1, posi2) = rect.drag_state.into_posi_pixcel()?;
            let posi3 = f.reverse_translate(posi1)?;
            let posi4 = f.reverse_translate(posi2)?;
            let rect = Rectangle::new([posi3, posi4], colors::CYAN);
            chart.plotting_area().draw(&rect).unwrap();
            Some(())
        }
    }
}

pub mod my_kline2 {
    use crate::hover_cpn::DragState;

    use super::my_kline::{MyAxis, MyKline};
    use super::*;
    use chrono::NaiveDateTime as dt;
    use plotters::coord::ReverseCoordTranslate;
    use std::cell::RefCell;
    use std::collections::HashMap;

    pub struct MyKline2(f32, Vec<Vec<(f32, i32)>>, MyKline, RefCell<Vec<(dt, dt)>>);

    type Hm = HashMap<i32, i32>;
    type ChartK<'a, DB> = ChartContext<'a, DB, Cts<MyAxis<dt>, Rcf32>>;
    impl MyKline2 {

        pub fn new(tz: f32, volume: Vec<Vec<(f32, i32)>>, my_kline: MyKline) -> Self {
            MyKline2(tz, volume, my_kline, RefCell::new(vec![]))
        } 
        
        fn draw_volume<DB: DrawingBackend>(
            &self, 
            chart: &mut ChartK<DB>,
            rect: RecordBox,
        ) -> Option<()>
        {
            let f = chart.plotting_area().as_coord_spec();
            if let Some(delete) = rect.delete {
                if let Some(posi_delete) = f.reverse_translate(delete) {
                    let posi_delete = posi_delete.0;
                    self.3.borrow_mut().retain(|x| x.0 > posi_delete || x.1 < posi_delete);
                }
            }
            self
                .3
                .borrow()
                .iter()
                .for_each(|(date_start, date_end)| {
                    self.draw_volume_date(chart, *date_start, *date_end);
                });
            let f = chart.plotting_area().as_coord_spec();
            if let Some((posi1, posi2)) = rect.drag_state.into_posi_pixcel() {
                if let (Some(posi3), Some(posi4)) = (f.reverse_translate(posi1), f.reverse_translate(posi2)) {
                    let (posi3, posi4) = (posi3.0, posi4.0);
                    let date1 = posi3.min(posi4);
                    let date2 = posi3.max(posi4);
                    self.draw_volume_date(chart, date1, date2);
                    if let DragState::PressedAndReleased(_, _) = rect.drag_state {
                        self.3.borrow_mut().push((date1, date2));
                    }
                }
            }
            Some(())
        }

        fn draw_volume_date<DB: DrawingBackend>(
            &self, 
            chart: &mut ChartK<DB>,
            date_start: dt,
            date_end: dt,
        ) -> Option<()>
        {
            let volume = self.get_volume_slice_date(date_start, date_end)?;
            let mut top_price = volume[0][0].0;
            let mut bot_price = top_price;
            let g = chart.backend_coord(&(date_end, bot_price));
            if g.0 <= 20 { return None }
            let volume = volume
                .iter()
                .fold(Hm::new(), |mut accu, x| {
                    x.iter()
                        .for_each(|x| {
                            top_price = top_price.max(x.0);
                            bot_price = bot_price.min(x.0);
                            let k = accu.entry(x.0 as i32).or_insert(0);
                            *k += x.1;
                        });
                    accu
                });
            let c = RGBAColor(255, 255, 0, 0.5);
            chart.plotting_area().backend_ops(|db| {
                for (k, v) in volume.into_iter() {
                    let rect = self.get_rect(chart, date_start, (k as f32, v));
                    if rect.0.0 > 20 && rect.1.0 > 20 {
                        db.draw_rect(rect.0, rect.1, &c, true)?;
                    }
                }
                Ok(())
            }).unwrap();
            let rect = Rectangle::new([(date_start, top_price), (date_end, bot_price)], c);
            chart.draw_series(std::iter::once(rect)).unwrap();
            ().into()
        }

        #[allow(clippy::type_complexity)]
        fn get_volume_slice_date(
            &self,
            posi1: dt,
            posi2: dt,
        ) -> Option<&[Vec<(f32, i32)>]>
        {
            let start_i = self.2.0.iter().position(|x| x >= &posi1)?;
            let end_i = self.2.0.iter().position(|x| x > &posi2)?;
            if start_i >= end_i { return None }
            (&self.1[start_i..end_i]).into()
        }

        fn get_rect<DB: DrawingBackend>(
            &self, 
            chart: &ChartK<DB>,
            date: dt,
            (price, vol): (f32, i32), 
        ) -> (PosiPixcel, PosiPixcel)
        {
            let posi1 = chart.backend_coord(&(date, price - self.0 / 2.));
            let posi2 = chart.backend_coord(&(date, price + self.0 / 2.));
            (posi1, (posi2.0 + vol, posi2.1))
        }
    }

    impl InitFig for MyKline2 {
        type X = dt;
        type Y = [f32; 4];
        type CoorX = dt;
        type CoorY = f32;
        type Xaxis = MyAxis<dt>;
        type Yaxis = Rcf32;

        fn get_xaxis(&self) -> Self::Xaxis {
            self.2.get_xaxis()
        }
        fn get_yaxis(&self) -> Self::Yaxis {
            self.2.get_yaxis()
        }
        fn get_xy(&self, posi: (Self::CoorX, Self::CoorY)) -> Option<(Self::X, Self::Y)> {
            self.2.get_xy(posi)
        }

        fn zoom_fig<'a, DB: DrawingBackend>(
            &self,
            root: &'a DrawingArea<DB, Shift>,
            rect: RecordBox,
        ) -> Option<CC<'a, DB, Self::Xaxis, Self::Yaxis>> {
            self.2.zoom_fig(root, rect)
        }

        fn hover_axis<DB: DrawingBackend>(
            &self,
            posi: (Self::CoorX, Self::CoorY),
            bounds: (Self::CoorX, Self::CoorY),
        ) -> Option<LineSeries<DB, (Self::CoorX, Self::CoorY)>> {
            self.2.hover_axis(posi, bounds)
        }

        fn hover_point(
            &self,
            posi: (Self::CoorX, Self::CoorY),
        ) -> Option<Circle<(Self::CoorX, Self::CoorY), i32>> {
            self.2.hover_point(posi)
        }

        fn hover_text(
            &self,
            coor: (Self::CoorX, Self::CoorY),
            dim: (u32, u32),
        ) -> Option<MultiLineText<(Self::CoorX, Self::CoorY), String>> {
            self.2.hover_text(coor, dim)
        }

        fn add_box<DB: DrawingBackend>(
            &self,
            chart: &mut ChartContext<DB, Cts<Self::Xaxis, Self::Yaxis>>,
            rect: RecordBox,
        ) -> Option<()>
        where
            Cts<Self::Xaxis, Self::Yaxis>: ReverseCoordTranslate<From = (Self::CoorX, Self::CoorY)>,
        {
            self.2.add_box(chart, rect.clone());
            self.draw_volume(chart, rect);
            ().into()
        }
    }
}

fn get_range_from_k(data: &[[f32; 4]]) -> Range<f32> {
    if data.is_empty() {
        return 0f32..1f32;
    }
    let (start, end) = data.iter().fold((data[0][2], data[0][1]), |mut tc, z| {
        tc.0 = tc.0.min(z[2]);
        tc.1 = tc.1.max(z[1]);
        tc
    });
    let pad = (end - start) * 0.1;
    (start - pad)..(end + pad)
}