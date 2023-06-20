mod camera;
mod cube;
mod cubes;
mod pipeline;

use crate::cubes::{Cubes, Pipeline};
use iced::{executor, Application, Command, Element, Length, Renderer, Theme};
use iced_graphics::Shader;

fn main() -> iced::Result {
    Example::run(iced::Settings::default())
}

#[derive(Default)]
struct Example {
    cubes: Cubes,
}

#[derive(Debug, Clone)]
enum Message {}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Example::default(), Command::none())
    }

    fn title(&self) -> String {
        "Iced Cubes".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {}

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Shader::new(&self.cubes)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
