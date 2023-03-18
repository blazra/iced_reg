use iced::theme::Theme;
use iced::widget::{container, Row};
use iced::{
    executor, futures::channel::mpsc, subscription, Application, Command, Element,
    Settings, Subscription,
};
use std::collections::BTreeMap;
use std::{thread, time};

use iced_aw::Wrap;

use iced_reg::{register, Register, Field};

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

struct Reg {
    name: &'static str,
    address: u16,
    read_value: u16,
    write_value: u16,
    fields: &'static [Field],
}

struct RegApp {
    state: SenderState,
    reg_map: BTreeMap<u16, Reg>,
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
        let reg_map = BTreeMap::from([
            (
                0x00,
                Reg {
                    name: "REG_1",
                    address: 0,
                    read_value: 0,
                    write_value: 0,
                    fields: &[
                        Field {
                            name: "Field A",
                            width: 8,
                        },
                        Field {
                            name: "Field B",
                            width: 8,
                        },
                    ],
                },
            ),
            (
                0x01,
                Reg {
                    name: "REG_2",
                    address: 1,
                    read_value: 0,
                    write_value: 0,
                    fields: &[
                        Field {
                            name: "Field A",
                            width: 8,
                        },
                        Field {
                            name: "Field B",
                            width: 8,
                        },
                    ],
                },
            ),
            (
                0x02,
                Reg {
                    name: "REG_3",
                    address: 2,
                    read_value: 0,
                    write_value: 0,
                    fields: &[
                        Field {
                            name: "Field A",
                            width: 8,
                        },
                        Field {
                            name: "Field B",
                            width: 8,
                        },
                    ],
                },
            ),
            (
                0x03,
                Reg {
                    name: "REG_4",
                    address: 3,
                    read_value: 0,
                    write_value: 0,
                    fields: &[
                        Field {
                            name: "Field A",
                            width: 8,
                        },
                        Field {
                            name: "Field B",
                            width: 8,
                        },
                    ],
                },
            ),
        ]);
        let reg = RegApp {
            state: SenderState::default(),
            reg_map,
        };
        (reg, Command::none())
    }

    fn title(&self) -> String {
        String::from("Iced_reg example")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::RegReadValChanged(address, value) => self.reg_map.get_mut(&address).unwrap().read_value = value,
            Message::RegWriteValChanged(address, value) => self.reg_map.get_mut(&address).unwrap().write_value = value,
            Message::RegRead(address) => match &mut self.state {
                SenderState::Starting => {
                    println!("Error: trying to read register, but we are not connected")
                }
                SenderState::Ready(sender) => sender
                    .try_send(WorkerInput::ReadReg(address))
                    .expect("Send msg to background worker"),
            },
            Message::RegWrite(address) => match &mut self.state {
                SenderState::Starting => {
                    println!("Error: trying to write register, but we are not connected")
                }
                SenderState::Ready(sender) => sender
                    .try_send(WorkerInput::WriteReg(address, self.reg_map.get(&address).unwrap().write_value))
                    .expect("Send msg to background worker"),
            },
            Message::Echo(event) => match event {
                Event::Ready(sender) => {
                    self.state = SenderState::Ready(sender);
                    println!("worker Ready");
                }
                Event::ReadFinished(Ok((address, value))) => {
                    self.reg_map.get_mut(&address).unwrap().read_value = value;
                }
                Event::ReadFinished(Err(address)) => {
                    eprintln!("update: Error while reading reg at address 0x{:04X}", address)
                }
                Event::WriteFinished(Ok((address, value))) => {
                    println!(
                        "Write data 0x{:04X} to address 0x{:04X} finished",
                        value, address
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
        let mut content = Wrap::new();

        for reg in self.reg_map.values() {
            content = content.push(container(register(
                reg.read_value,
                reg.write_value,
                Message::RegReadValChanged,
                Message::RegWriteValChanged,
                Message::RegRead,
                Message::RegWrite,
                reg.name,
                reg.address,
                reg.fields,
            )));
        }
        container(content.spacing(20.0).padding(20.0)).into()
    }
}
