use iced::widget::canvas::{
    self, gradient::Location, gradient::Position, Cache, Canvas, Cursor,
    Geometry, Gradient,
};
use iced::{
    executor, time, Application, Color, Command, Element, Length, Point,
    Rectangle, Renderer, Settings, Size, Subscription, Theme,
};
use std::time::Instant;

fn main() -> iced::Result {
    Bench::run(Settings {
        antialiasing: true,
        try_opengles_first: true,
        ..Settings::default()
    })
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick(Instant),
}

struct Bench {
    cache: Cache,
    start: Instant,
    now: Instant,
}

impl Bench {
    pub fn update(&mut self, now: Instant) {
        self.now = now;
        self.cache.clear();
    }
}

impl Application for Bench {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let now = Instant::now();

        (
            Bench {
                cache: Default::default(),
                start: now,
                now,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Bench")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(instant) => {
                self.update(instant);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        //60 fps
        time::every(time::Duration::from_secs_f32(0.0167)).map(Message::Tick)
    }
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
            //the gradient must change properties every draw to reload uniform data

            let elapsed = self.now - self.start;
            let x = elapsed.as_millis();
            let y = f32::abs(f32::sin(x as f32 / 1000.0));
            frame.scale_x(y);

            let gradient = Gradient::linear(Position::Relative {
                top_left: Point::ORIGIN,
                size: Size {
                    width: frame.width(),
                    height: frame.height(),
                },
                start: Location::Left,
                end: Location::Right,
            })
            .add_stop(0.0, Color::from_rgb(y, 0.0, 0.0))
            .add_stop(0.5, Color::from_rgb(0.0, y, 0.0))
            .add_stop(1.0, Color::from_rgb(0.0, 0.0, y))
            .build()
            .unwrap();

            frame.fill_rectangle(Point::ORIGIN, bounds.size(), &gradient);
        });

        vec![geometry]
    }
}
