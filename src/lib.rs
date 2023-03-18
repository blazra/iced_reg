use iced::theme;
use iced::widget::{
    button, column, container, row, text, vertical_space, Button, Row,
};
use iced::{Element, Length, Renderer};
use iced_lazy::{component, Component};


const WIDTH: usize = 16;

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
    fields: &'static [Field]
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
    fields: &'static [Field]
) -> Register<Message> {
    Register::new(read_value, write_value, on_read_val_changed, on_write_val_changed, on_read, on_write, name, address, fields)
}

#[derive(Debug, Clone)]
pub enum RegisterMessage {
    ReadRegChanged(u16),
    WriteRegChanged(u16),
    RegRead,
    RegWrite,
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
        fields: &'static [Field]
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
            fields
        }
    }
}

impl<Message> Component<Message, Renderer> for Register<Message> {
    type State = ();
    type Event = RegisterMessage;

    fn update(&mut self, _state: &mut Self::State, event: RegisterMessage) -> Option<Message> {
        match event {
            RegisterMessage::ReadRegChanged(value) => Some((self.on_read_val_changed)(self.address, value)),
            RegisterMessage::WriteRegChanged(value) => Some((self.on_write_val_changed)(self.address, value)),
            RegisterMessage::RegRead => Some((self.on_read)(self.address)),
            RegisterMessage::RegWrite => Some((self.on_write)(self.address)),
        }
    }

    fn view(&self, _state: &Self::State) -> Element<RegisterMessage, Renderer> {
        fn bit_button(state: u16) -> Button<'static, RegisterMessage, Renderer> {
            button(container(text(state)).center_x().width(Length::Fill))
                .padding(2)
                .width(Length::FillPortion(1))
        }
        let numbering = (0..WIDTH)
            .rev()
            .fold(Row::new().spacing(2), |row, offset| {
                row.push(
                    button(container(text(offset)).center_x().width(Length::Fill))
                        .padding(2)
                        .on_press(RegisterMessage::WriteRegChanged(self.write_value))
                        .width(Length::FillPortion(1)),
                )
            })
            .height(Length::FillPortion(1));

        let fields = self
            .fields
            .iter()
            .rev()
            .fold(
                Row::new().spacing(2),
                |row: Row<'_, RegisterMessage, Renderer>, field| {
                    row.push(
                        button(container(text(field.name)).center_x().width(Length::Fill))
                            .padding(2)
                            .on_press(RegisterMessage::WriteRegChanged(self.write_value))
                            .width(Length::FillPortion(field.width as u16)),
                    )
                },
            )
            .height(Length::FillPortion(8));

        let read_bits = (0..WIDTH)
            .rev()
            .fold(Row::new(), |row, offset| {
                let state = (self.read_value >> offset) & 1;
                let style = if state == 1 {
                    theme::Button::Positive
                } else {
                    theme::Button::Destructive
                };
                row.push(
                    button(container(text(state)).center_x().width(Length::Fill))
                        .padding(2)
                        .width(Length::FillPortion(1))
                        .style(style),
                )
            })
            .spacing(2);

        let write_bits = (0..WIDTH)
            .rev()
            .fold(Row::new(), |row: Row<'_, RegisterMessage, Renderer>, offset| {
                let state = (self.write_value >> offset) & 1;
                let style = if state == 1 {
                    theme::Button::Positive
                } else {
                    theme::Button::Destructive
                };
                row.push(
                    bit_button(state)
                        .on_press(RegisterMessage::WriteRegChanged(self.write_value ^ (1 << offset)))
                        .width(Length::FillPortion(1))
                        .style(style),
                )
            })
            .spacing(2);

        let header = button(
            container(text(format!(
                "{}@0x{:04X} = R 0x{:04X} W 0x{:04X}",
                self.name, self.address, self.read_value, self.write_value
            )))
            .center_x()
            .width(Length::Fill),
        )
        .on_press(RegisterMessage::WriteRegChanged(self.write_value))
        .width(Length::Fill); //.height(Length::FillPortion(1));

        let reg = row![
            column![
                vertical_space(Length::Fill),
                button(container(text("R")).center_x().width(Length::Fill))
                    .height(25)
                    .width(Length::Fill)
                    .padding(2)
                    .on_press(RegisterMessage::RegRead),
                button(container(text("W")).center_x().width(Length::Fill))
                    .height(25)
                    .width(Length::Fill)
                    .padding(2)
                    .on_press(RegisterMessage::RegWrite),
            ]
            .spacing(2.0)
            .width(Length::FillPortion(1)),
            column![header, numbering, fields, read_bits, write_bits,]
                .spacing(2.0)
                .width(Length::FillPortion(WIDTH as u16))
        ]
        .spacing(2.0)
        .width(400)
        .height(300);

        let content = column![reg,].spacing(20.0).padding(20).max_width(600);

        container(content).into()
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
