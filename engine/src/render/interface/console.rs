use iced_wgpu::Renderer;
use iced_winit::{
    Container, TextInput, Column, Row, Button, Text, Scrollable,
    Length, Color, Background, Align,
    text_input::State as TextState,
    button::State as ButtonState,
    scrollable::State as ScrollState,
    Command, Program, Element,
};

#[derive(Debug,Default)]
pub struct Console {
    visible: bool,
    input: TextState,
    input_value: String,
    enter_button: ButtonState,
    scroll: ScrollState,
    pub texts: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    VisibilityChanged,
    InputChanged(String),
    NewText(String),
    EnterPressed,
}

impl Console {
    pub fn new() -> Console {
        Console::default()
    }
}

impl Program for Console {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message{
            Message::InputChanged(input) => {
                self.input_value = input;
            },
            Message::NewText(text) => {
                self.texts.push(text.clone());
            },
            Message::EnterPressed => {
                self.texts.push(self.input_value.clone());
                self.input_value.clear();
            },
            Message::VisibilityChanged => {
                self.visible = !self.visible;
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let column = Column::new()
            .push(
                Scrollable::new(&mut self.scroll)
                    .push(
                        self.texts.iter().fold(Column::new().spacing(5), |column, text| {
                            column.push(Text::new(text))
                        })
                    )
                    .width(Length::Fill)
                    .height(Length::FillPortion(10))
                    .padding(10)
            )
            .push(
                Row::new()
                    .push(
                        TextInput::new(
                            &mut self.input,
                            "Type something here",
                            &self.input_value,
                            Message::InputChanged,
                        )
                        .on_submit(Message::EnterPressed)
                        .padding(10)
                    )
                    .push(
                        Button::new(&mut self.enter_button, Text::new("Submit"))
                            .padding(10)
                            .on_press(Message::EnterPressed)
                    )
                    .spacing(10)
                    .align_items(Align::Center)
                    .height(Length::FillPortion(1))
            );

        if self.visible{
            Container::new(column)
            // .width(Length::Units(500))
            // .height(Length::Units(500))
            .padding(10)
            .style(ContainerStyle)
            .into()
        }else{
            Row::new().into()
        }
    }
}

use iced::container;
struct ContainerStyle;

impl container::StyleSheet for ContainerStyle{
    fn style(&self) -> container::Style{
        container::Style {
            background: Some(Background::Color(Color::from_rgba8(
                0x36, 0x39, 0x3F, 0.75
            ))),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}
