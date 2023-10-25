use iced::Point;

#[derive(Clone, Copy, Debug, Default)]
pub struct PosiBackend(pub f32, pub f32);

pub type PosiPixcel = (i32, i32);

#[derive(Clone, Copy, Debug, Default)]
pub struct PosiChart<T, N>(pub T, pub N);

impl From<PosiBackend> for PosiPixcel {
    fn from(value: PosiBackend) -> Self {
        (value.0 as i32 , value.1 as i32)
    }
}


#[derive(Debug, Clone)]
pub enum DragState {
    PressedOnly(PosiPixcel),
    PressedAndDraging(PosiPixcel, PosiPixcel),
    PressedAndReleased(PosiPixcel, PosiPixcel),
    Idle,
}

impl DragState {
    pub fn into_posi_pixcel(&self) -> Option<(PosiPixcel, PosiPixcel)> {
        match self {
            DragState::PressedAndDraging(posi1, posi2) | DragState::PressedAndReleased(posi1, posi2) => {
                Some((*posi1, *posi2))
            }
            _ => None
        }
    }
}

impl Default for DragState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Default, Clone)]
pub struct RecordBox {
    pub point: PosiPixcel,
    pub drag_state: DragState,
    pub zoom: (PosiPixcel, f32),
    pub delete: Option<PosiPixcel>,
    pub drag_fig: DragState,
}

impl RecordBox {
    pub fn reset_delete(&mut self) {
        self.delete = None;
    }
    
    pub fn change_from_point(&mut self, point: PosiPixcel) {
        self.point = point;
    }

    pub fn change_from(&mut self, event: iced::mouse::Event, point: PosiPixcel) {
        use iced::mouse::Button::*;
        use iced::mouse::Event;
        use iced::mouse::ScrollDelta;
        self.zoom.1 = 0.;
        // println!("{:?}", event);
        match (&self.drag_state, event) {
            (DragState::Idle, Event::ButtonPressed(Left)) => {
                self.drag_state = DragState::PressedOnly(point);
            }
            (DragState::PressedOnly(p), Event::CursorMoved { position: _ }) => {
                self.drag_state = DragState::PressedAndDraging(*p, point);
            }
            (DragState::PressedOnly(p),  Event::ButtonReleased(Left)) => {
                self.drag_state = DragState::PressedAndReleased(*p, point);
            }
            (DragState::PressedAndDraging(p1, _p2), Event::ButtonReleased(Left)) => {
                self.drag_state = DragState::PressedAndReleased(*p1, point);
            }
            (DragState::PressedAndDraging(p1, _p2), Event::CursorMoved { position: _ }) => {
                self.drag_state = DragState::PressedAndDraging(*p1, point);
            }
            (DragState::PressedAndReleased(_, _), _) => {
                self.drag_state = DragState::Idle;
            }
            (_, Event::WheelScrolled { delta: ScrollDelta::Lines { x: _, y } }) => {
                self.zoom.1 = y * 0.1;
                self.zoom.0 = point;
            }

            _ => {}
        }
    }

    pub fn change_drag_fig(&mut self, event: iced::mouse::Event, point: PosiPixcel) {
        use iced::mouse::Button::*;
        use iced::mouse::Event;
        fn to(point: Point) -> PosiPixcel {
            (point.x as i32, point.y as i32)
        }
        match (&self.drag_fig, event) {
            (DragState::Idle, Event::ButtonPressed(Left)) => {
                self.drag_fig = DragState::PressedOnly(point);
            }
            (DragState::PressedOnly(_p), Event::CursorMoved { position }) => {
                self.drag_fig = DragState::PressedAndDraging(to(position), to(position));
            }
            (DragState::PressedOnly(p),  Event::ButtonReleased(Left)) => {
                self.drag_fig = DragState::PressedAndReleased(*p, *p);
            }
            (DragState::PressedAndDraging(_p1, p2), Event::ButtonReleased(Left)) => {
                self.drag_fig = DragState::PressedAndReleased(*p2, *p2);
            }
            (DragState::PressedAndDraging(_p1, p2), Event::CursorMoved { position }) => {
                self.drag_fig = DragState::PressedAndDraging(*p2, to(position));
            }
            (DragState::PressedAndReleased(_, _), _) => {
                self.drag_fig = DragState::Idle;
            }
            _ => {}
        }
    }

    pub fn change_from_keyboard(&mut self, event: iced::keyboard::Event, point: PosiPixcel) {
        use iced::keyboard::{ Event::KeyReleased, KeyCode };
        match event {
            KeyReleased { key_code: KeyCode::Backspace, .. } => {
                self.delete = point.into();
            }
            _other => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Rect(RecordBox),
    OnHover,
    OnRect,
    OnDrag,
}
