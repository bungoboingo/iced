mod triangle;

use crate::triangle::Triangle;
use iced::widget::container;
use iced::{executor, Application, Command, Element, Length, Renderer, Theme, Color};

fn main() -> iced::Result {
    Example::run(iced::Settings::default())
}

struct Example;

#[derive(Debug, Clone)]
enum Message {}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Example {}, Command::none())
    }

    fn title(&self) -> String {
        "Example".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {}

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Element::from(container(Triangle::new().width(Length::Fill).height(Length::Fill).id(0))
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y())
            .explain(Color::from_rgb8(255, 0, 0))
    }
}
