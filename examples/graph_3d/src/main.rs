mod graph_3d;
mod pipeline;
mod camera;
pub mod util;

use iced::widget::container;
use iced::{executor, Application, Command, Element, Renderer, Theme};
use crate::graph_3d::Graph3D;

fn main() -> iced::Result {
    Example::run(iced::Settings::default())
}

struct Example {}

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
        "3D Graph".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {}

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Graph3D::new().into()
    }
}