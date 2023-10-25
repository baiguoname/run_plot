#![allow(clippy::type_complexity)]


use iced::mouse::Cursor;
use iced::event;
use iced::{Alignment, Length, Application, executor, Theme, Command};

use plotters::style::colors;
use iced_widget::{button, column, row, Container};
use plotters::coord::ReverseCoordTranslate;

use plotters::{coord::Shift, prelude::*};
use plotters_backend::DrawingBackend;
use plotters_iced::{Chart, ChartBuilder, ChartWidget, DrawingArea};
use iced::Result;
use iced::Settings;
use crate::hover_cpn::{PosiBackend, PosiPixcel, Message, RecordBox};
type Cts<T, N> = Cartesian2d<T, N>;
use crate::prelude::CC;

pub trait InitFig {
    type X;
    type Y;
    type CoorX;
    type CoorY;
    type Xaxis: Ranged<ValueType = Self::CoorX>;
    type Yaxis: Ranged<ValueType = Self::CoorY>;

    fn get_xaxis(&self) -> Self::Xaxis;

    fn get_yaxis(&self) -> Self::Yaxis;

    fn get_xy(&self, _posi: (Self::CoorX, Self::CoorY)) -> Option<(Self::X, Self::Y)> {
        None
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
        let posi1 = f.reverse_translate(posi1)?;
        let posi2 = f.reverse_translate(posi2)?;
        let rect = Rectangle::new([posi1, posi2], colors::CYAN);
        chart.plotting_area().draw(&rect).unwrap();
        Some(())
    }

    fn add_hover<DB: DrawingBackend>(
        &self,
        chart: &mut ChartContext<DB, Cts<Self::Xaxis, Self::Yaxis>>,
        posi: PosiPixcel,
    )
    where
        Self::Xaxis: Ranged<ValueType = Self::CoorX>,
        Self::Yaxis: Ranged<ValueType = Self::CoorY>,
        Self::CoorX: Clone + 'static,
        Self::CoorY: Clone +  'static,
        Cts<Self::Xaxis, Self::Yaxis>: ReverseCoordTranslate<From = (Self::CoorX, Self::CoorY)>,
    {
        let f = chart.plotting_area().as_coord_spec();
        let posi = if let Some(posi) = f.reverse_translate(posi) { posi } else { return; };


        if let Some(hover_point) = self.hover_point(posi.clone()) {
            chart.draw_series(std::iter::once(hover_point)).unwrap();
        }
        let bounds = (self.get_xaxis().range().start, self.get_yaxis().range().start);
        if let Some(hover_axis) = self.hover_axis(posi.clone(), bounds) {
            chart.draw_series(hover_axis).unwrap();
        }
        
        let dim = chart.plotting_area().dim_in_pixel();
        if let Some(hover_text) = self.hover_text(posi.clone(), dim) {
            chart.plotting_area().draw(&hover_text).unwrap();
        }
    }

    fn hover_point(
        &self, 
        _posi: (Self::CoorX, Self::CoorY),
    ) -> Option<Circle<(Self::CoorX, Self::CoorY), i32>>
    {
        None
    }

    fn hover_axis<DB: DrawingBackend>(
        &self,
        _posi: (Self::CoorX, Self::CoorY),
        _bounds: (Self::CoorX, Self::CoorY),
    ) -> Option<LineSeries<DB, (Self::CoorX, Self::CoorY)>>
    {
        None
    }

    fn hover_text(
        &self,
        _coor: (Self::CoorX, Self::CoorY),
        _dim: (u32, u32),
    ) -> Option<MultiLineText<(Self::CoorX, Self::CoorY), String>>
    {
        None
    }

    fn init_fig<'a, DB: DrawingBackend>(
        &self, 
        _root: &'a DrawingArea<DB, Shift>
    ) -> Option<CC<'a, DB, Self::Xaxis, Self::Yaxis>>
    {
        None
    }

    fn zoom_fig<'a, DB: DrawingBackend>(
        &self,
        _root: &'a DrawingArea<DB, Shift>,
        _rect: RecordBox,
    ) -> Option<CC<'a, DB, Self::Xaxis, Self::Yaxis>>
    {
        None
    }
}

pub struct MyChart<T> {
    rect: RecordBox,
    on_hover: bool,
    on_rect: bool,
    on_drag: bool,
    bg_fig: T,
}

impl<T> MyChart<T> {
    pub fn from(bg_fig: T) -> Self {
        Self {
            rect: RecordBox::default(),
            on_hover: false,
            on_rect: false,
            on_drag: false,
            bg_fig,
        }
    }
}

impl<T, X, Y, Xaxis, Yaxis, CoorX, CoorY> Chart<Message> for MyChart<T>
where
    T: InitFig<X = X, Y = Y, CoorX = CoorX, CoorY = CoorY, Xaxis = Xaxis, Yaxis = Yaxis>,
    Xaxis: Ranged<ValueType = CoorX>,
    Yaxis: Ranged<ValueType = CoorY>,
    X: PartialOrd + Clone,
    Y: Clone,
    CoorX: Clone + 'static,
    CoorY: Clone + 'static,
    Cts<Xaxis, Yaxis>: ReverseCoordTranslate<From = (CoorX, CoorY)> + Clone,
{
    type State = RecordBox;
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, state: &Self::State, root: DrawingArea<DB, Shift>) {
        let mut chart = match self.bg_fig.zoom_fig(&root, state.clone()) {
            Some(chart) => chart,
            None => self.bg_fig.init_fig(&root).unwrap(),
        };
        if self.on_hover {
            self.bg_fig.add_hover(&mut chart, self.rect.point);
        }
        if self.on_rect {
            self.bg_fig.add_box(&mut chart, self.rect.clone());
        }
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: iced_widget::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        use iced_widget::canvas::Event::{ Mouse, Keyboard };
        if let Cursor::Available(point) = cursor {
            let p_origin = bounds.position();
            let p = point - p_origin;
            let posi_backend = PosiBackend(p.x, p.y);
            let posi_pixcel: PosiPixcel = posi_backend.into();
            state.reset_delete();
            match event {
                Mouse(event) if bounds.contains(point) => {
                    state.change_from_point(posi_pixcel);
                    state.change_from(event, posi_pixcel);
                    if self.on_drag {
                        state.change_drag_fig(event, posi_pixcel);
                    }
                }
                Keyboard(event) => {
                    state.change_from_keyboard(event, posi_pixcel);
                }
                _ => {}
            }
        }
        (event::Status::Ignored, Some(Message::Rect(state.clone())))
    }
}

pub struct State<T> {
    chart: MyChart<T>,
}

impl<T, X, Y, Xaxis, Yaxis, CoorX, CoorY> Application for State<T>
where
    T: InitFig<X = X, Y = Y, CoorX = CoorX, CoorY = CoorY, Xaxis = Xaxis, Yaxis = Yaxis>,
    Xaxis: Ranged<ValueType = CoorX>,
    Yaxis: Ranged<ValueType = CoorY>,
    X: PartialOrd + Clone,
    Y: Clone,
    CoorX: Clone + 'static + std::fmt::Debug,
    CoorY: Clone + 'static + std::fmt::Debug,
    Cts<Xaxis, Yaxis>: ReverseCoordTranslate<From = (CoorX, CoorY)> + Clone,   
{
    type Executor = executor::Default;
    type Theme = Theme;
    type Message = Message;
    type Flags = T;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let chart = MyChart::from(flags);
        (State { chart }, Command::none())
    }

    fn title(&self) -> String {
        "Run Chart".into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Rect(record_box) => {
                self.chart.rect = record_box;
            }
            Message::OnHover => {
                self.chart.on_hover = !self.chart.on_hover;
            }
            Message::OnRect => {
                self.chart.on_rect = !self.chart.on_rect;
            }
            Message::OnDrag => {
                self.chart.on_drag = !self.chart.on_drag;
            }
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let chart_widget = ChartWidget::new(&self.chart)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);
        let col = column![
            row![
                button("Hover").on_press(Message::OnHover),
                button("Rect").on_press(Message::OnRect),
                button("Drag").on_press(Message::OnDrag),
            ],
            chart_widget,
        ]
            .spacing(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center)
            .padding(15);
        Container
            ::new(col)
            .padding(5)
            .center_x()
            .center_y()
            .into()
    }
}

pub trait RunPlot {
    fn run_plot(self) -> Result;
}

impl<T> RunPlot for T
where
    State<T>: Application<Flags = T>,
    T: 'static,
{
    fn run_plot(self) -> Result {
        <State<T> as Application>::run(Settings::with_flags(self))
    }
}


