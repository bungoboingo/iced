mod camera;
mod cubes;

use crate::cubes::Pipeline;
use iced::{executor, Application, Command, Element, Length, Renderer, Theme};
use iced_graphics::Custom;

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
        Custom::new(Pipeline::init, 0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
