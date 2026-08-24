use std::f32::consts::PI;
use crate::quadratic_fitter::{PointSettingCommand, QuadraticFitter};
use iced::mouse::{Cursor, Interaction};
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{Cache, Geometry, LineCap, Path, Stroke, stroke};
use iced::widget::{Action, canvas, container};
use iced::window::Settings;
use iced::{Element, Event, Fill, Point, Rectangle, Renderer, Size, Theme, Vector, mouse};

pub mod quadratic_fitter;

#[derive(Debug, Clone,Default)]
struct DragState {
    is_mouse_pressed: bool,
    is_dragging: bool,
}

fn main() -> iced::Result {
    iced::application(QuadPainter::new, QuadPainter::update, QuadPainter::view)
        .theme(QuadPainter::theme)
        .window(Settings {
            size: Size {
                width: 500.0,
                height: 500.0,
            },
            ..Settings::default()
        })
        .title("Quadratic function")
        .run()
}

struct QuadPainter {
    fitter: QuadraticFitter,
    cache: Cache,
}

impl QuadPainter {
    fn new() -> Self {
        Self {
            fitter: QuadraticFitter::new(),
            cache: Cache::new(),
        }
    }

    fn update(&mut self, message: PointSettingCommand) {
        self.fitter.apply_command(message);
        self.cache.clear();
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNightStorm
    }

    fn view(&self) -> Element<'_, PointSettingCommand> {
        let canvas = canvas(self as &Self).width(Fill).height(Fill);

        container(canvas).into()
    }
}

/// The center and the radius of the drawing area.
fn get_center_radius(rect: &Rectangle) -> (Point, f32) {
    (rect.center(), rect.width.min(rect.height) / 2.0)
}

fn circle_radius(drawing_radius: f32) -> f32 {
    drawing_radius / 30.0
}

/// Simply sets the command

impl canvas::Program<PointSettingCommand> for QuadPainter {
    type State = DragState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<Action<PointSettingCommand>> {
        let (center, radius) = get_center_radius(&bounds);
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {state.is_mouse_pressed = true;None},
            Event::Mouse(mouse::Event::CursorMoved { position, .. }) if state.is_mouse_pressed => {
                // Now we need to get the mouse position into the coordinates of system
                let x = (position.x - center.x) / radius;
                let y = -(position.y - center.y) / radius;
                let circle_radius = if state.is_dragging {f32::INFINITY} else {circle_radius(radius) / radius};

                let res = self.fitter.get_closest_point([x,y], circle_radius).map(Action::publish);
                state.is_dragging = res.is_some();
                res
            },
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {state.is_mouse_pressed = false; state.is_dragging=false; None}
            _ => None,
        }

           }

    fn draw(
        &self,
        _: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let content = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.palette();

            let (center, radius) = get_center_radius(&bounds);

            frame.translate(Vector::new(center.x, center.y));

            // Draw the coordinate axis:
            let width = radius / 200.0;
            let arrow_offset = radius / 20.0;

            let arrow_stroke = || -> Stroke {
                Stroke {
                    width,
                    style: stroke::Style::Solid(palette.text),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            for angle in [0.0, PI / 2.0] {
                frame.with_save(|frame| {
                    frame.rotate(angle);
                    frame.stroke(
                        &Path::line(Point::new(0.0, -radius), Point::new(0.0, radius)),
                        arrow_stroke(),
                    );
                    frame.stroke(
                        &Path::line(
                            Point::new(-arrow_offset, -radius + arrow_offset),
                            Point::new(0.0, -radius),
                        ),
                        arrow_stroke(),
                    );
                    frame.stroke(
                        &Path::line(
                            Point::new(arrow_offset, -radius + arrow_offset),
                            Point::new(0.0, -radius),
                        ),
                        arrow_stroke(),
                    );
                })
            }

            // Now we build the line.
            let make_point = |x: [f32; 2]| Point::new(x[0] * radius, -x[1] * radius);
            let mut builder = Builder::new();
            let mut que = self.fitter.get_line_points();
            builder.move_to(make_point(que.next().unwrap()));
            for point in que {
                builder.line_to(make_point(point));
            }

            let mut line_stroke = arrow_stroke();
            line_stroke.style = stroke::Style::Solid(palette.primary);
            frame.stroke(&builder.build(), line_stroke);

            // Now paint the three circles.
            let circle_radius = circle_radius(radius);
            for raw_point in self.fitter.get_base_points() {
                let point = Point::new(raw_point[0] as f32 * radius, -raw_point[1] as f32 * radius);
                frame.fill(&Path::circle(point, circle_radius), palette.warning);
            }
        });

        vec![content]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Interaction {
        let (center, radius) = get_center_radius(&bounds);
        let width = radius / 15.0;
        if self.fitter.get_base_points().iter().any(|point| {
            cursor.is_over(Rectangle::new(
                Point::new(center.x + point[0] * radius - width / 2.0, center.y - point[1] * radius - width / 2.0),
                Size::new( width,  width),
            ))
        }) {
            Interaction::Grab
        } else {
            Interaction::default()
        }
    }
}
