#[cfg(feature = "gradient")]
use iced::widget::canvas::gradient::{Gradient, Location, Position};

use iced::widget::canvas::{self, Cache, Canvas, Cursor, Geometry};
use iced::{
    executor, Application, Color, Command, Element, Length, Point, Rectangle,
    Renderer, Settings, Size, Theme,
};
use rand::{thread_rng, Rng};

fn main() -> iced::Result {
    Bench::run(Settings {
        antialiasing: true,
        try_opengles_first: true,
        ..Settings::default()
    })
}

#[derive(Debug, Clone, Copy)]
enum Message {}

#[derive(Default)]
struct Bench {
    cache: Cache,
}

impl Application for Bench {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Bench::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Bench")
    }

    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn random_color() -> Color {
    Color::from_rgb(
        thread_rng().gen_range(0.0..1.0),
        thread_rng().gen_range(0.0..1.0),
        thread_rng().gen_range(0.0..1.0),
    )
}

impl<Message> canvas::Program<Message> for Bench {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(bounds.size(), |frame| {
            let size = Size::new(bounds.width / 50.0, bounds.height / 12.0);

            for row in 0..12 {
                for rect in 0..50 {
                    let top_left = Point::new(
                        size.width * rect as f32,
                        size.height * row as f32,
                    );

                    let fill = random_color();

                    #[cfg(feature = "gradient")]
                    let fill = &Gradient::linear(Position::Relative {
                        top_left,
                        size,
                        start: Location::TopLeft,
                        end: Location::BottomRight,
                    })
                    .add_stop(0.0, random_color())
                    .add_stop(0.5, random_color())
                    .add_stop(1.0, random_color())
                    .build()
                    .unwrap();

                    frame.fill_rectangle(top_left, size, fill)
                }
            }
        });

        vec![geometry]
    }
}
