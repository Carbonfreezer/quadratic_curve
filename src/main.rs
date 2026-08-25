#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::quadratic_fitter::{PointSettingCommand, QuadPoint, QuadraticFitter};
use iced::mouse::{Cursor, Interaction};
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{Cache, Geometry, LineCap, Path, Stroke, stroke};
use iced::widget::{Action, canvas, container};
use iced::window::Settings;
use iced::{Element, Event, Fill, Point, Rectangle, Renderer, Size, Theme, Vector, mouse, window};
use std::f32::consts::PI;

pub mod quadratic_fitter;
/// Radius of the grabble point in world coordinates (x, y ∈ [-1, 1]).
const HANDLE_RADIUS: f32 = 1.0 / 30.0;

/// Internal state used for the GUI.
#[derive(Debug, Clone, Copy, Default)]
enum DragState {
    /// Currently not dragging.
    #[default]
    Idle,
    /// Dragging the indicated control point.
    Dragging(usize),
}

/// Screen to world coordinates.
fn to_world(bounds: &Rectangle, position: Point) -> QuadPoint {
    let (center, radius) = get_center_radius(bounds);
    [
        (position.x - center.x) / radius,
        -(position.y - center.y) / radius,
    ]
}

fn main() -> iced::Result {
    iced::application(QuadPainter::new, QuadPainter::update, QuadPainter::view)
        .theme(QuadPainter::theme)
        .window(Settings {
            size: Size {
                width: 500.0,
                height: 500.0,
            },
            icon: window::icon::from_file_data(include_bytes!("../assets/icon.png"), None).ok(),
            ..Settings::default()
        })
        .title("Quadratic function")
        .run()
}

/// The logic structure of the iced application.
struct QuadPainter {
    /// The fitting core to compute the curve.
    fitter: QuadraticFitter,
    /// A cache for the render geometry to avoid unnecessary recalculations.
    cache: Cache,
}

impl QuadPainter {
    fn new() -> Self {
        Self {
            fitter: QuadraticFitter::default(),
            cache: Cache::new(),
        }
    }

    /// Logic is contained here by applying changes from the ui.
    fn update(&mut self, message: PointSettingCommand) {
        self.fitter.apply_command(message);
        self.cache.clear();
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    /// Rendering is essentially only the canvas.
    fn view(&self) -> Element<'_, PointSettingCommand> {
        let canvas = canvas(self).width(Fill).height(Fill);
        container(canvas).into()
    }
}

/// The center and the radius of the drawing area. Automatically grabs a square area in the middle.
fn get_center_radius(rect: &Rectangle) -> (Point, f32) {
    (rect.center(), rect.width.min(rect.height) / 2.0)
}

impl canvas::Program<PointSettingCommand> for QuadPainter {
    type State = DragState;

    /// Does the mouse drag logic for the control points.
    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Action<PointSettingCommand>> {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let world = to_world(&bounds, cursor.position_over(bounds)?);
                if let Some(index) = self.fitter.closest_point(world, HANDLE_RADIUS) {
                    *state = DragState::Dragging(index);
                }
                None
            }
            Event::Mouse(mouse::Event::CursorMoved { position, .. }) => {
                let DragState::Dragging(index) = *state else {
                    return None;
                };
                Some(Action::publish(PointSettingCommand {
                    index_to_set: index,
                    point: to_world(&bounds, *position),
                }))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                *state = DragState::Idle;
                None
            }
            _ => None,
        }
    }

    /// Does the drawing over a cache only to synthesize geometry if needed.
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

            let arrow_stroke = Stroke {
                width,
                style: stroke::Style::Solid(palette.text),
                line_cap: LineCap::Round,
                ..Stroke::default()
            };

            for angle in [0.0, PI / 2.0] {
                frame.with_save(|frame| {
                    frame.rotate(angle);
                    frame.stroke(
                        &Path::line(Point::new(0.0, -radius), Point::new(0.0, radius)),
                        arrow_stroke,
                    );
                    frame.stroke(
                        &Path::line(
                            Point::new(-arrow_offset, -radius + arrow_offset),
                            Point::new(0.0, -radius),
                        ),
                        arrow_stroke,
                    );
                    frame.stroke(
                        &Path::line(
                            Point::new(arrow_offset, -radius + arrow_offset),
                            Point::new(0.0, -radius),
                        ),
                        arrow_stroke,
                    );
                })
            }

            // Now we build the line.
            let make_point = |x: QuadPoint| Point::new(x[0] * radius, -x[1] * radius);
            let mut builder = Builder::new();
            let mut que = self.fitter.get_line_points();
            builder.move_to(make_point(que.next().unwrap()));
            for point in que {
                builder.line_to(make_point(point));
            }

            let mut line_stroke = arrow_stroke;
            line_stroke.style = stroke::Style::Solid(palette.primary);
            frame.stroke(&builder.build(), line_stroke);

            // Now paint the three circles.
            let circle_radius = radius * HANDLE_RADIUS;
            for raw_point in self.fitter.get_base_points() {
                let point = Point::new(raw_point[0] * radius, -raw_point[1] * radius);
                frame.fill(&Path::circle(point, circle_radius), palette.warning);
            }
        });

        vec![content]
    }

    ///  Adjusts the cursor to mouse dragging ability.
    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Interaction {
        if matches!(state, DragState::Dragging(_)) {
            return Interaction::Grabbing;
        }
        match cursor.position_over(bounds) {
            Some(position)
                if self
                    .fitter
                    .closest_point(to_world(&bounds, position), HANDLE_RADIUS)
                    .is_some() =>
            {
                Interaction::Grab
            }
            _ => Interaction::default(),
        }
    }
}
