use iced::alignment::Horizontal;
use iced::widget::{column, container, row, scrollable, text, vertical_space};
use iced::{
    executor, theme, widget, Alignment, Application, Background, Color,
    Command, Element, Length, Rectangle, Renderer, Theme,
};
use std::array;
use viewer::Viewer;

fn main() -> iced::Result {
    Example::run(iced::Settings::default())
}

struct Example {
    current_bounds: Option<Rectangle>,
    buttons: [Button; 7],
}

#[derive(Clone, Debug)]
enum Message {
    RequestBounds(container::Id),
    BoundsReceived(Rectangle),
}

impl Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Example {
                current_bounds: None,
                buttons: array::from_fn(|_| Button::new()),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Example".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::RequestBounds(id) => {
                return widget::bounds(id, Message::BoundsReceived);
            }
            Message::BoundsReceived(bounds) => {
                self.current_bounds = Some(bounds);
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let top_row = row![
            self.buttons[0].view("Button 1"),
            self.buttons[1].view("Button 2"),
            self.buttons[2].view("Button 3"),
            self.buttons[3].view("Button 5"),
        ]
        .spacing(20);

        let scrollable = scrollable(
            container(
                column![
                    vertical_space(40),
                    self.buttons[4].view("Start"),
                    vertical_space(400),
                    self.buttons[5].view("Middle!"),
                    vertical_space(400),
                    self.buttons[6].view("End!"),
                    vertical_space(40),
                ]
                .width(Length::Fill)
                .align_items(Alignment::Center)
                .spacing(20),
            )
            .style(theme::Container::Custom(Box::new(ScrollableStyle)))
            .width(Length::Fill),
        )
        .height(Length::Fill);

        let bounds_text = if let Some(bounds) = self.current_bounds {
            text(format!("{:?}", bounds))
        } else {
            text("Click a button to draw its bounds!")
        };

        let content = column![
            vertical_space(80),
            top_row,
            scrollable,
            bounds_text,
            vertical_space(80)
        ]
        .align_items(Alignment::Center)
        .spacing(40);

        Viewer::new(
            container(content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y(),
            self.current_bounds,
        )
        .into()
    }
}

struct Button(container::Id);

impl Button {
    fn new() -> Self {
        Self(container::Id::unique())
    }

    fn view(&self, label: &'static str) -> widget::Container<'_, Message> {
        container(
            iced::widget::button(
                text(label).horizontal_alignment(Horizontal::Center),
            )
            .width(100)
            .padding([8, 12, 8, 12])
            .on_press(Message::RequestBounds(self.0.clone())),
        )
        .id(self.0.clone())
    }
}

struct ScrollableStyle;

impl container::StyleSheet for ScrollableStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Background::Color(Color::from_rgb8(220, 220, 220))
                .into(),
            ..Default::default()
        }
    }
}

mod viewer {
    use iced::advanced::layout::{Limits, Node};
    use iced::advanced::widget::{Operation, Tree};
    use iced::advanced::{
        self, overlay, renderer, Clipboard, Layout, Shell, Widget,
    };
    use iced::event::Status;
    use iced::mouse::Interaction;
    use iced::{Color, Element, Event, Length, Point, Rectangle};

    pub struct Viewer<'a, Message, Renderer> {
        content: Element<'a, Message, Renderer>,
        bounds: Option<Rectangle>,
    }

    impl<'a, Message, Renderer> Viewer<'a, Message, Renderer> {
        pub fn new(
            content: impl Into<Element<'a, Message, Renderer>>,
            bounds: Option<Rectangle>,
        ) -> Self {
            Self {
                content: content.into(),
                bounds,
            }
        }

        pub fn set_bounds(&mut self, bounds: Rectangle) {
            self.bounds = Some(bounds);
        }
    }

    impl<'a, Message, Renderer> Widget<Message, Renderer>
        for Viewer<'a, Message, Renderer>
    where
        Renderer: advanced::Renderer,
    {
        fn width(&self) -> Length {
            Length::Fill
        }

        fn height(&self) -> Length {
            Length::Fill
        }

        fn layout(&self, renderer: &Renderer, limits: &Limits) -> Node {
            let content = self.content.as_widget().layout(renderer, limits);

            Node::with_children(limits.resolve(content.size()), vec![content])
        }

        fn draw(
            &self,
            tree: &Tree,
            renderer: &mut Renderer,
            theme: &Renderer::Theme,
            style: &renderer::Style,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
        ) {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor_position,
                viewport,
            );

            if let Some(bounds) = self.bounds {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: 0.0.into(),
                        border_width: 2.0,
                        border_color: Color::from_rgb8(255, 0, 0),
                    },
                    Color::TRANSPARENT,
                );
            }
        }

        fn children(&self) -> Vec<Tree> {
            vec![Tree::new(&self.content)]
        }

        fn diff(&self, tree: &mut Tree) {
            tree.diff_children(&[&self.content]);
        }

        fn operate(
            &self,
            tree: &mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
            operation: &mut dyn Operation<Message>,
        ) {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            )
        }

        fn on_event(
            &mut self,
            tree: &mut Tree,
            event: Event,
            layout: Layout<'_>,
            cursor_position: Point,
            renderer: &Renderer,
            clipboard: &mut dyn Clipboard,
            shell: &mut Shell<'_, Message>,
        ) -> Status {
            self.content.as_widget_mut().on_event(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor_position,
                renderer,
                clipboard,
                shell,
            )
        }

        fn overlay<'b>(
            &'b mut self,
            tree: &'b mut Tree,
            layout: Layout<'_>,
            renderer: &Renderer,
        ) -> Option<overlay::Element<'b, Message, Renderer>> {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
            )
        }

        fn mouse_interaction(
            &self,
            tree: &Tree,
            layout: Layout<'_>,
            cursor_position: Point,
            viewport: &Rectangle,
            renderer: &Renderer,
        ) -> Interaction {
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor_position,
                viewport,
                renderer,
            )
        }
    }

    impl<'a, Message, Renderer> From<Viewer<'a, Message, Renderer>>
        for Element<'a, Message, Renderer>
    where
        Renderer: 'a + advanced::Renderer,
        Message: 'a + Clone,
    {
        fn from(viewer: Viewer<'a, Message, Renderer>) -> Self {
            Element::new(viewer)
        }
    }
}
