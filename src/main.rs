use iced::theme::Theme;
use iced::widget::{column, container};
use iced::{
    executor, futures::channel::mpsc, subscription, Application, Command, Element, Length,
    Settings, Subscription,
};
use std::{thread, time};

use iced_reg::{register, Field};

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = (800, 600);
    RegApp::run(settings)
}

#[derive(Debug, Clone)]
enum Event {
    Ready(mpsc::Sender<WorkerInput>),
    ReadFinished(Result<(u16, u16), u16>),
    WriteFinished(Result<(u16, u16), u16>),
}

#[derive(Debug, Clone)]
enum WorkerInput {
    ReadReg(u16),
    WriteReg(u16, u16),
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

fn bg_worker() -> Subscription<Event> {
    struct SomeWorker;

    subscription::unfold(
        std::any::TypeId::of::<SomeWorker>(),
        ReceiverState::Disconnected,
        |state| async move {
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
                        WorkerInput::ReadReg(address) => {
                            println!("bg worker: -> R @0x{:04X}", address);
                            thread::sleep(time::Duration::from_secs(1)); // Simulate transfer
                            let value = 0x5AA5;
                            println!("bg worker: <- R @0x{:04X} 0x{:04X}", address, value);

                            // Finally, we can optionally return a message to tell the
                            // `Application` the work is done
                            (
                                Some(Event::ReadFinished(Ok((address, value)))),
                                ReceiverState::Ready(receiver),
                            )
                        }
                        WorkerInput::WriteReg(address, value) => {
                            println!("bg worker: -> W @0x{:04X} 0x{:04X}", address, value);
                            thread::sleep(time::Duration::from_secs(1)); // Simulate transfer
                            println!("bg worker: <- W @0x{:04X} 0x{:04X}", address, value);

                            // Finally, we can optionally return a message to tell the
                            // `Application` the work is done
                            (
                                Some(Event::WriteFinished(Ok((address, value)))),
                                ReceiverState::Ready(receiver),
                            )
                        }
                    }
                }
            }
        },
    )
}

struct RegApp {
    read_reg_value: u16,
    write_reg_value: u16,
    state: SenderState,
}

#[derive(Debug, Clone)]
enum Message {
    RegReadValChanged(u16, u16),
    RegWriteValChanged(u16, u16),
    RegRead(u16),
    RegWrite(u16),
    Echo(Event),
}

impl Application for RegApp {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = Theme;

    fn new(_flags: ()) -> (RegApp, Command<Message>) {
        let reg = RegApp {
            read_reg_value: 0x00AA,
            write_reg_value: 7,
            state: SenderState::default(),
        };
        (reg, Command::none())
    }

    fn title(&self) -> String {
        String::from("Iced register widget")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RegReadValChanged(address, value) => self.read_reg_value = value,
            Message::RegWriteValChanged(address, value) => self.write_reg_value = value,
            Message::RegRead(address) => match &mut self.state {
                SenderState::Starting => {
                    println!("Error: trying to read register, but we are not connected")
                }
                SenderState::Ready(sender) => sender
                    .try_send(WorkerInput::ReadReg(0x3))
                    .expect("Send msg to background worker"),
            },
            Message::RegWrite(address) => match &mut self.state {
                SenderState::Starting => {
                    println!("Error: trying to write register, but we are not connected")
                }
                SenderState::Ready(sender) => sender
                    .try_send(WorkerInput::WriteReg(0x3, self.write_reg_value))
                    .expect("Send msg to background worker"),
            },
            Message::Echo(event) => match event {
                Event::Ready(sender) => {
                    self.state = SenderState::Ready(sender);
                    println!("worker Ready");
                }
                Event::ReadFinished(Ok((address, value))) => {
                    self.read_reg_value = value;
                }
                Event::ReadFinished(Err(address)) => {
                    eprintln!("update: Error while reg at address 0x{:04X}", address)
                }
                Event::WriteFinished(Ok((address, value))) => {
                    println!(
                        "Write of 0x{:04X} to address 0x{:04X} finished",
                        address, value
                    )
                }
                Event::WriteFinished(Err(address)) => {
                    eprintln!(
                        "update: Error while writing reg at address 0x{:04X}",
                        address
                    )
                }
            },
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        bg_worker().map(Message::Echo)
    }

    fn view(&self) -> Element<Message> {
        let content = column![register(
            self.read_reg_value,
            self.write_reg_value,
            Message::RegReadValChanged,
            Message::RegWriteValChanged,
            Message::RegRead,
            Message::RegWrite,
            "TEST_REG",
            0x0012,
            &[
                Field {
                    name: "Field A",
                    width: 2
                },
                Field {
                    name: "Field B",
                    width: 8
                },
                Field {
                    name: "Field C",
                    width: 6
                }
            ]
        ),]
        .spacing(20.0)
        .padding(20)
        .max_width(600);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
