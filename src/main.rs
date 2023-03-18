use iced::{Element, Sandbox, Settings};
use iced_reg::{register, Field};

pub fn main() -> iced::Result {
    RegApp::run(Settings::default())
}

struct RegApp {
    name: &'static str,
    address: u16,
    read_value: u16,
    write_value: u16,
    fields: &'static [Field],
}

#[derive(Debug, Clone)]
enum Message {
    ReadValChanged(u16, u16),
    WriteValChanged(u16, u16),
    Read(u16),
    Write(u16),
}

impl Sandbox for RegApp {
    type Message = Message;

    fn new() -> Self {
        RegApp {
            name: "REG_NAME",
            address: 0x00AD,
            read_value: 0x5AA5,
            write_value: 0x00A0,
            fields: &[
                Field {
                    name: "Field A",
                    width: 8,
                },
                Field {
                    name: "Field B",
                    width: 7,
                },
                Field {
                    name: "Field C",
                    width: 1,
                },
            ],
        }
    }

    fn title(&self) -> String {
        String::from("Iced register widget")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ReadValChanged(_address, value) => self.read_value = value,
            Message::WriteValChanged(_address, value) => self.write_value = value,
            Message::Read(address) => println!("Read register at address 0x{:04X}", address),
            Message::Write(address) => println!("Write to register at address 0x{:04X}", address),
        }
    }

    fn view(&self) -> Element<Message> {
        register(
            self.read_value,
            self.write_value,
            Message::ReadValChanged,
            Message::WriteValChanged,
            Message::Read,
            Message::Write,
            self.name,
            self.address,
            self.fields,
        )
        .into()
    }
}
