use crate::canvas::{Cursor, Geometry, Path};
use iced::widget::canvas::Cache;
use iced::widget::{canvas, Canvas};
use iced::{
    executor, Application, Color, Command, Element, Length, Rectangle,
    Renderer, Size, Theme,
};

fn main() -> iced::Result {
    Snowfield::run(iced::Settings::default())
}

#[derive(Default)]
struct Snowfield {
    cache: Cache,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application for Snowfield {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Snowfield::default(), Command::none())
    }

    fn title(&self) -> String {
        "Snowfield".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl canvas::Program<Message, Theme> for Snowfield {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry> {
        let circle = self.cache.draw(Size::new(200.0, 200.0), |frame| {
            let path = Path::circle(bounds.center(), 20.0);
            frame.fill(&path, Color::from_rgb8(180, 120, 120));
        });

        vec![circle]
    }
}
