#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::{thread, time};
use iced::alignment::Vertical;
use iced::theme::{self, Theme};
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, progress_bar, radio,
    row, scrollable, slider, text, text_input, toggler, vertical_rule,
    vertical_space, Button, Column, Row,
};
use iced::{
    executor, Alignment, Application, Color, Command, Element, Length,
    Renderer, Settings, Subscription, futures::channel::mpsc, subscription
};

const WIDTH: usize = 16;

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = (800, 600);
    RegApp::run(settings)
}

#[derive(Debug, Clone)]
enum Event {
    Ready(mpsc::Sender<WorkerInput>),
    ReadFinished(Result<(RegInfo, u16), RegInfo>),
    WriteFinished(Result<RegInfo, RegInfo>),
}

#[derive(Debug, Clone, Default)]
struct RegInfo {
    name: &'static str,
    address: u16,
    fields: &'static[u8],
}

struct RegisterState16 {
    info: RegInfo,
    read_value: u16,
    write_value: u16,
}

#[derive(Debug, Clone)]
enum WorkerInput {
    ReadReg(RegInfo),
    WriteReg(RegInfo, u16),
}

#[derive(Default)]
enum SenderState {
    #[default]
    Starting,
    Ready(mpsc::Sender<WorkerInput>),
}

#[derive(Default)]
enum ReceiverState {
    #[default]
    Disconnected,
    Ready(mpsc::Receiver<WorkerInput>),
}


fn some_worker() -> Subscription<Event> {
    struct SomeWorker;

    subscription::unfold(std::any::TypeId::of::<SomeWorker>(), ReceiverState::Disconnected, |state| async move {
        match state {
            ReceiverState::Disconnected => {
                // Create channel
                let (sender, receiver) = mpsc::channel(100);

                (Some(Event::Ready(sender)), ReceiverState::Ready(receiver))
            }
            ReceiverState::Ready(mut receiver) => {
                use iced::futures::StreamExt;

                // Read next input sent from `Application`
                let input = receiver.select_next_some().await;

                match input {
                    WorkerInput::ReadReg(reg_info) => {
                        println!("bg worker: -> R {} @0x{:04X}", reg_info.name, reg_info.address);
                        thread::sleep(time::Duration::from_secs(1)); // Simulate transfer
                        println!("bg worker: <- R {} @0x{:04X} 0x{:04X}", reg_info.name, reg_info.address, 0x5AA5);

                        // Finally, we can optionally return a message to tell the
                        // `Application` the work is done
                        (Some(Event::ReadFinished(Ok((reg_info, 0x1015)))), ReceiverState::Ready(receiver))
                    },
                    WorkerInput::WriteReg(reg_info, value) => {
                        println!("bg worker: -> W {} @0x{:04X} 0x{:04X}", reg_info.name, reg_info.address, value);
                        thread::sleep(time::Duration::from_secs(1)); // Simulate transfer
                        println!("bg worker: <- W {} @0x{:04X} 0x{:04X}", reg_info.name, reg_info.address, value);

                        // Finally, we can optionally return a message to tell the
                        // `Application` the work is done
                        (Some(Event::WriteFinished(Ok(reg_info))), ReceiverState::Ready(receiver))
                    }
                }
            }
        }
    })
}



struct RegApp {
    theme: Theme,
    read_reg_value: u16,
    write_reg_value: u16,
    state: SenderState,
    reg_info: RegInfo,
    register_state: RegisterState16,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ThemeType {
    Dark,
    StDark,
}

#[derive(Debug, Clone)]
enum Message {
    ReadRegChanged(u16),
    WriteRegChanged(u16),
    RegRead,
    RegWrite,
    Echo(Event)
}

fn custom_theme() -> Theme {
    Theme::custom(theme::Palette {
        background: Color::from_rgb(0.1, 0.1, 0.1),
        text: Color::from_rgb8(15, 34, 65),
        primary: Color::from_rgb8(15, 34, 65),
        success: Color::from_rgb(0.0, 1.0, 0.0),
        danger: Color::from_rgb8(208, 0, 112),
    })
}

const BIT_SIZE: usize = 25;

fn bit_button(state: u16) -> Button<'static, Message, Renderer> {
    button(container(text(state)).center_x().width(Length::Fill))
        .padding(2)
        .width(Length::FillPortion(1))
}

impl Application for RegApp {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (RegApp, Command<Message>) {
        let reg = RegApp {
            theme: custom_theme(),
            read_reg_value: 10,
            write_reg_value: 5,
            state: SenderState::default(),
            reg_info: RegInfo { name: "TEST_REG_NAME", address: 0xFFEE, fields: &[2, 8, 6] },
            register_state: RegisterState16 {
                info: RegInfo { name: "TEST_REG_NAME", address: 0xFFEE, fields: &[2, 8, 6] },
                read_value: 10,
                write_value: 5,
            }
        };
        (reg, Command::none())
    }

    fn title(&self) -> String {
        String::from("Register")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ReadRegChanged(value) => self.read_reg_value = value,
            Message::WriteRegChanged(value) => self.write_reg_value = value,
            Message::RegRead => match &mut self.state {
                SenderState::Starting => println!("Error: trying to read register, but we are not connected"),
                SenderState::Ready(sender) => sender.try_send(WorkerInput::ReadReg(self.reg_info.clone())).expect("Send msg to background worker")
            },
            Message::RegWrite => match &mut self.state {
                SenderState::Starting => println!("Error: trying to write register, but we are not connected"),
                SenderState::Ready(sender) => sender.try_send(WorkerInput::WriteReg(self.reg_info.clone(), self.write_reg_value)).expect("Send msg to background worker")
            },
            Message::Echo(event) => match event {
                Event::Ready(sender) => {
                    self.state = SenderState::Ready(sender);
                    println!("update: worker Ready");
                },
                Event::ReadFinished(Ok((reg_info, value))) => {
                    self.read_reg_value = value;
                },
                Event::ReadFinished(Err(reg_info)) => {
                    eprintln!("update: Error while reading {} at address 0x{:04X}", reg_info.name, reg_info.address)
                },
                Event::WriteFinished(Ok(reg_info)) => {
                    println!("Write finished")
                },
                Event::WriteFinished(Err(reg_info)) => {
                    eprintln!("update: Error while writing {} at address 0x{:04X}", reg_info.name, reg_info.address)
                },
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        some_worker().map(Message::Echo)
    }

    fn view(&self) -> Element<Message> {
        let numbering = (0..WIDTH)
            .rev()
            .fold(Row::new().spacing(2), |row, offset| {
                row.push(
                    button(
                        container(text(offset)).center_x().width(Length::Fill),
                    )
                    .padding(2)
                    .on_press(Message::WriteRegChanged(self.write_reg_value))
                    .width(Length::FillPortion(1)),
                )
            })
            .height(Length::FillPortion(1));

        let fields = self.reg_info.fields
            .iter()
            .rev()
            .fold(
                Row::new().spacing(2),
                |row: Row<'_, Message, Renderer>, offset| {
                    row.push(
                        button(
                            container(text(offset))
                                .center_x()
                                .width(Length::Fill),
                        )
                        .padding(2)
                        .on_press(Message::WriteRegChanged(
                            self.write_reg_value,
                        ))
                        .width(Length::FillPortion(*offset as u16)),
                    )
                },
            )
            .height(Length::FillPortion(8));

        let read_bits = (0..WIDTH).rev().fold(Row::new(), |row, offset| {
            let state = (self.read_reg_value >> offset) & 1;
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
        }).spacing(2);

        let write_bits = (0..WIDTH).rev().fold(
            Row::new(),
            |row: Row<'_, Message, Renderer>, offset| {
                let state = (self.write_reg_value >> offset) & 1;
                let style = if state == 1 {
                    theme::Button::Positive
                } else {
                    theme::Button::Destructive
                };
                row.push(
                    bit_button(state)
                        .on_press(Message::WriteRegChanged(
                            self.write_reg_value ^ (1 << offset),
                        ))
                        .width(Length::FillPortion(1))
                        .style(style),
                )
            },
        ).spacing(2);

        let header = button(
            container(text(format!(
                "{} = R:0x{:04X} W:0x{:04X}",
                self.reg_info.name, self.read_reg_value, self.write_reg_value
            )))
            .center_x()
            .width(Length::Fill),
        )
        .on_press(Message::WriteRegChanged(self.write_reg_value))
        .width(Length::Fill)
        ;//.height(Length::FillPortion(1));

        let reg = row![
            column![
                vertical_space(Length::Fill),
                button(container(text("R")).center_x().width(Length::Fill)).height(25).width(Length::Fill)
                    .padding(2)
                    .on_press(Message::RegRead),
                button(container(text("W")).center_x().width(Length::Fill)).height(25).width(Length::Fill)
                    .padding(2)
                    .on_press(Message::RegWrite),
            ].spacing(2).width(Length::FillPortion(1)),
            column![header, numbering, fields, read_bits, write_bits,].spacing(2).width(Length::FillPortion(WIDTH as u16))
        ]
        .spacing(2)
        .width(400)
        .height(300);

        let content = column![reg,].spacing(20).padding(20).max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
