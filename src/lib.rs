use iced::alignment::{Horizontal, Vertical};
use iced::theme;
use iced::widget::{
    button, column, component, container, row, text, vertical_space, Button, Component, Row,
};
use iced::{Element, Length, Renderer};

const WIDTH: u16 = 16;
const SPACING: f32 = 2.0;
const BUTTON_WIDTH: f32 = 22.0;
const BUTTON_HEIGHT: f32 = 22.0;
const WIDGET_WIDTH: f32 =
    BUTTON_WIDTH * (WIDTH as f32 + 2.0) + (WIDTH as f32 + 2.0 - 1.0) * SPACING;
const WIDGET_HEIGHT: f32 = BUTTON_HEIGHT * 13.0;

#[derive(Debug, Clone)]
pub struct Field {
    pub name: &'static str,
    pub width: u8,
}

pub struct Register<Message> {
    read_value: u16,
    write_value: u16,
    on_read_val_changed: Box<dyn Fn(u16, u16) -> Message>,
    on_write_val_changed: Box<dyn Fn(u16, u16) -> Message>,
    on_read: Box<dyn Fn(u16) -> Message>,
    on_write: Box<dyn Fn(u16) -> Message>,
    name: &'static str,
    address: u16,
    fields: &'static [Field],
}

pub fn register<Message>(
    read_value: u16,
    write_value: u16,
    on_read_val_changed: impl Fn(u16, u16) -> Message + 'static,
    on_write_val_changed: impl Fn(u16, u16) -> Message + 'static,
    on_read: impl Fn(u16) -> Message + 'static,
    on_write: impl Fn(u16) -> Message + 'static,
    name: &'static str,
    address: u16,
    fields: &'static [Field],
) -> Register<Message> {
    Register::new(
        read_value,
        write_value,
        on_read_val_changed,
        on_write_val_changed,
        on_read,
        on_write,
        name,
        address,
        fields,
    )
}

#[derive(Debug, Clone)]
pub enum RegisterMessage {
    ReadRegChanged(u16),
    WriteRegChanged(u16),
    RegRead,
    RegWrite,
    RegReadToWrite,
}

impl<Message> Register<Message> {
    pub fn new(
        read_value: u16,
        write_value: u16,
        on_read_val_changed: impl Fn(u16, u16) -> Message + 'static,
        on_write_val_changed: impl Fn(u16, u16) -> Message + 'static,
        on_read: impl Fn(u16) -> Message + 'static,
        on_write: impl Fn(u16) -> Message + 'static,
        name: &'static str,
        address: u16,
        fields: &'static [Field],
    ) -> Self {
        Self {
            read_value,
            write_value,
            on_read_val_changed: Box::new(on_read_val_changed),
            on_write_val_changed: Box::new(on_write_val_changed),
            on_read: Box::new(on_read),
            on_write: Box::new(on_write),
            name,
            address,
            fields,
        }
    }
}

impl<Message> Component<Message, Renderer> for Register<Message> {
    type State = ();
    type Event = RegisterMessage;

    fn update(&mut self, _state: &mut Self::State, event: RegisterMessage) -> Option<Message> {
        match event {
            RegisterMessage::ReadRegChanged(value) => {
                Some((self.on_read_val_changed)(self.address, value))
            }
            RegisterMessage::WriteRegChanged(value) => {
                Some((self.on_write_val_changed)(self.address, value))
            }
            RegisterMessage::RegRead => Some((self.on_read)(self.address)),
            RegisterMessage::RegWrite => Some((self.on_write)(self.address)),
            RegisterMessage::RegReadToWrite => {
                Some((self.on_write_val_changed)(self.address, self.read_value))
            }
        }
    }

    fn view(&self, _state: &Self::State) -> Element<RegisterMessage, Renderer> {
        fn bit_button(state: u16) -> Button<'static, RegisterMessage, Renderer> {
            button(
                container(
                    text(state)
                        .vertical_alignment(Vertical::Center)
                        .horizontal_alignment(Horizontal::Center),
                )
                .center_x()
                .center_y()
                .width(Length::Fill),
            )
            .padding(0)
            .width(BUTTON_WIDTH)
            .height(BUTTON_HEIGHT)
        }

        let header = button(
            container(text(format!(
                "{}@0x{:04X} = R 0x{:04X} W 0x{:04X}",
                self.name, self.address, self.read_value, self.write_value
            )))
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill),
        )
        .padding(0)
        .width(16.0 * BUTTON_WIDTH + 15.0 * SPACING)
        .height(BUTTON_HEIGHT);

        let numbering = (0..WIDTH)
            .rev()
            .fold(Row::new(), |row, offset| {
                row.push(
                    button(
                        container(
                            text(offset)
                                .vertical_alignment(Vertical::Center)
                                .horizontal_alignment(Horizontal::Center),
                        )
                        .center_x()
                        .center_y()
                        .width(Length::Fill)
                        .height(Length::Fill),
                    )
                    .padding(0)
                    .height(BUTTON_HEIGHT)
                    .width(BUTTON_WIDTH),
                )
            })
            .spacing(SPACING)
            .height(BUTTON_HEIGHT);

        let fields = self
            .fields
            .iter()
            .rev()
            .fold(
                Row::new(),
                |row: Row<'_, RegisterMessage, Renderer>, field| {
                    row.push(
                        button(container(text(field.name)).center_x().width(Length::Fill))
                            .padding(0)
                            .width(
                                (field.width as f32 * BUTTON_WIDTH)
                                    + (((field.width - 1) as f32) * SPACING),
                            )
                            .height(Length::Fill),
                    )
                    .height(Length::Fill)
                },
            )
            .spacing(SPACING)
            .height(Length::Fill);

        let read_bits = (0..WIDTH)
            .rev()
            .fold(Row::new(), |row, offset| {
                let state = (self.read_value >> offset) & 1;
                let style = if state == 1 {
                    theme::Button::Positive
                } else {
                    theme::Button::Destructive
                };
                row.push(bit_button(state).padding(0).style(style))
            })
            .spacing(SPACING)
            .height(BUTTON_HEIGHT);

        let write_bits = (0..WIDTH)
            .rev()
            .fold(
                Row::new(),
                |row: Row<'_, RegisterMessage, Renderer>, offset| {
                    let state = (self.write_value >> offset) & 1;
                    let style = if state == 1 {
                        theme::Button::Positive
                    } else {
                        theme::Button::Destructive
                    };
                    row.push(
                        bit_button(state)
                            .on_press(RegisterMessage::WriteRegChanged(
                                self.write_value ^ (1 << offset),
                            ))
                            .style(style),
                    )
                },
            )
            .spacing(SPACING)
            .height(BUTTON_HEIGHT);

        let read_write_buttons = column![
            vertical_space(Length::Fill),
            button(
                container(text("R"))
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .height(BUTTON_HEIGHT)
            .width(BUTTON_WIDTH)
            .padding(0)
            .on_press(RegisterMessage::RegRead),
            button(
                container(text("W"))
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .height(BUTTON_HEIGHT)
            .width(BUTTON_WIDTH)
            .padding(0)
            .on_press(RegisterMessage::RegWrite),
        ]
        .spacing(SPACING)
        .width(BUTTON_WIDTH);

        let read_to_write_button = column![
            vertical_space(Length::FillPortion(10)),
            container(
                button("↪")
                    .style(theme::Button::Text)
                    .padding(0)
                    .on_press(RegisterMessage::RegReadToWrite)
            )
            .center_x()
            .center_y()
            .height(2.0 * BUTTON_HEIGHT + SPACING)
            .width(Length::Fill)
        ]
        .width(BUTTON_WIDTH);

        let reg = row![
            read_write_buttons,
            read_to_write_button,
            column![header, numbering, fields, read_bits, write_bits,]
                .spacing(SPACING)
                .width(Length::FillPortion(WIDTH))
        ]
        .spacing(SPACING)
        .width(WIDGET_WIDTH)
        .height(WIDGET_HEIGHT);

        container(reg)
            .padding(10)
            .into()
    }
}

impl<'a, Message> From<Register<Message>> for Element<'a, Message, Renderer>
where
    Message: 'a,
{
    fn from(register: Register<Message>) -> Self {
        component(register)
    }
}
