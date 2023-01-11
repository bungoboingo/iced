use iced::widget::{container, vertical_space};
use iced::{executor, Application, Color, Command, Element, Gradient, Radians, Renderer, Theme, Length, Settings};
use iced::theme::Container;

struct GradientDithering {}

fn main() -> iced::Result {
    GradientDithering::run(Settings::default())
}

impl Application for GradientDithering {
    type Executor = executor::Default;
    type Message = ();
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (GradientDithering{}, Command::none())
    }

    fn title(&self) -> String {
        "gradient_dithering".to_string()
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        container(vertical_space(Length::Fill))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(Container::Custom(Box::new(ContainerStyle)))
            .center_x()
            .center_y()
            .into()
    }
}

struct ContainerStyle;

impl container::StyleSheet for ContainerStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Gradient::linear(Radians(std::f32::consts::PI/2.0))
                .add_stop(0.0, Color::from_rgb8(0, 0, 0))
                .add_stop(1.0, Color::from_rgb8(50, 50, 50))
                .build()
                .expect("Build gradient")
                .into(),
            ..Default::default()
        }
    }
}
