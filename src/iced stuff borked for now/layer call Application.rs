use iced::{event, Color, Element, Event, Task as Command, Theme};
use iced_layershell::reexport::{Anchor, Layer, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use iced_layershell::Application;


#[tokio::main]
async fn main() -> Result<(), iced_layershell::Error> {

    loop {
        println!("Please input height of the window");
        let mut heightin = String::new();
        std::io::stdin().read_line(&mut heightin).unwrap();
        println!("You typed: {}", heightin);
        let height: u32 = heightin.trim().parse().unwrap();
        let mut widthin = String::new();
        println!("Please input width of the window");
        std::io::stdin().read_line(&mut widthin).unwrap();
        println!("You typed: {}", widthin);
        let width: u32 = widthin.trim().parse().unwrap();
        // call genUi function without waiting for response
        tokio::spawn(async move {
            genUi(width, height).await.unwrap();
        });
    }

    Ok(())
}

pub async fn genUi(width: u32, height: u32) -> Result<(), iced_layershell::Error> {
    let settings = Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Right,
            layer: Layer::Overlay,
            exclusive_zone: 0,
            size: Some((width, height)),
            margin: (10, 10, 10, 10),
            keyboard_interactivity: KeyboardInteractivity::None,
            binded_output_name: Some("test".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };
    tokio::spawn(async move {
        Counter::run(settings)
    });
    std::future::pending::<()>().await;
    Ok(())
}

struct Counter {
    text: String,
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    TextInput(String),
    IcedEvent(Event),
}

impl Application for Counter {
    type Message = Message;
    type Flags = ();
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            Self {
                text: "hello, write something here".to_string(),
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("Counter - Iced")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen().map(Message::IcedEvent)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(event) => {
                //println!("hello {event:?}");
                match event {
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right)) => {
                        println!("Right mouse button pressed");
                    }
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                        println!("Left mouse button pressed");
                    }
                    _ => {}
                }
                
                Command::none()
            }
            Message::TextInput(text) => {
                self.text = text;
                Command::none()
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<Message> {
        iced::widget::container("text container")
            .padding(10)
            .center(800)
            .style(move |_| {

                iced::widget::container::Style {
                    //border: borders.border.rounded(iced::border::radius(10)),
                    text_color: Some(iced::Color::parse("#ff0000").unwrap()),
                    border: iced::Border{
                        color: iced::Color::parse("#ff00ff").unwrap(),
                        width: 2.0,
                        radius: iced::border::radius(10),
                    },
                    shadow: iced::Shadow {
                        color: iced::Color::parse("#ff0000").unwrap(),
                        offset: iced::Vector {
                            x: 10.0,
                            y: 10.0,
                        },
                        blur_radius: 15.0,
                    },
                    background: Some(iced::Background::Color(iced::Color::parse("#000000").unwrap()))
                }
            })
            .into()
    }

    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        use iced_layershell::Appearance;
        Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.palette().text,
        }
    }
}