mod triangle;
mod program;

use crate::triangle::Triangle;
use iced::{executor, Application, Command, Element, Length, Renderer, Theme};
use crate::program::Program;

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
        iced_graphics::Custom::new(Program::init)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
