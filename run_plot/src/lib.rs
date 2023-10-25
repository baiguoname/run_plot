pub mod hover;
pub mod hover_cpn;
pub mod ins;

pub mod prelude {
    pub use crate::hover::{InitFig, RunPlot};
    pub use crate::hover_cpn::{PosiPixcel, PosiBackend, PosiChart};
    use plotters::prelude::ChartContext;
    use plotters::prelude::{RangedDate, Cartesian2d};
    pub use chrono::NaiveDate as da;
    pub use plotters_iced::{
        DrawingArea,
        ChartBuilder,
    };
    pub use plotters_backend::DrawingBackend;
    pub use plotters::{
        coord::{
            types::RangedCoordf32,
            Shift,
        }, 
        style::{Color, colors, RelativeSize, IntoTextStyle},
    };
    pub type Rcf32 = RangedCoordf32;
    pub type Rcda = RangedDate<da>;
    pub type Cts<T, N> = Cartesian2d<T, N>;
    pub type CC<'a, DB, X, Y> = ChartContext<'a, DB, Cts<X, Y>>;
    pub use crate::ins::my_kline::MyKline;
}
