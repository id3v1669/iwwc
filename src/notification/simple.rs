use iced::{event, Color, Element, Event, Task as Command, Theme};
use iced::widget::{text, row};
use iced_layershell::reexport::{Anchor, Layer, KeyboardInteractivity};
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::to_layer_message;
use iced_layershell::Application;
use iced::window::Id;

use once_cell::sync::Lazy;
use std::collections::{ VecDeque, HashMap };
use std::sync::{ Mutex, Arc };
use std::path::PathBuf;

//use gtk::prelude::*;
//use gtk::{IconTheme, IconLookupFlags};

use crate::shared_data::{GLOBAL_DATA_MAP, ID_QUEUE, ACTIVE_NOTIFICATIONS};

pub async fn gen_ui(width: u32, height: u32, id: Option<String>) -> Result<(), iced_layershell::Error> {
  println!("id: {:?}", id);
  {
    let mut id_queue = ID_QUEUE.lock().unwrap();
    id_queue.push_back(id.clone());
  }
  let mut offset = {
    let mut active_notifications = ACTIVE_NOTIFICATIONS.lock().unwrap();
    *active_notifications += 1;
    (height * (*active_notifications - 1))+(10 * (*active_notifications - 1)) + 10
  };
//   println!("before active_notifications");
//   let mut active_notifications = ACTIVE_NOTIFICATIONS.lock().unwrap();
//     println!("after active_notifications");
//   let offset = (height * *active_notifications)+(10 * *active_notifications) + 10;
  println!("offset: {:?}", offset);
  
  //*active_notifications += 1;

  let settings = Settings {
    layer_settings: LayerShellSettings {
      anchor: Anchor::Top | Anchor::Right,
      layer: Layer::Overlay,
      exclusive_zone: 400,
      size: Some((width, height)),
      margin: (offset.try_into().unwrap(), 10, 10, 10),
      keyboard_interactivity: KeyboardInteractivity::None,
      ..Default::default()
    },
    id: id.clone(),
    ..Default::default()
  };
    
  let _ = SimpleNotification::run(settings);

//   tokio::spawn(async move {
//     SimpleNotification::run(settings)
// });
// std::future::pending::<()>().await;
  Ok(())
}

struct SimpleNotification {
    id: Option<String>,
    notification: crate::daemon::nf_struct::Notification,
    icon_path: Option<PathBuf>,
}

impl SimpleNotification {
  async fn sleep_timer(sleep_time: u64) {
    tokio::time::sleep(std::time::Duration::from_secs(sleep_time)).await;
  }
}

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    IcedEvent(Event),
    Exit,
}

impl Application for SimpleNotification {
  type Message = Message;
  type Flags = ();
  type Theme = Theme;
  type Executor = iced::executor::Default;

  fn new(_flags: ()) -> (Self, Command<Message>) {
    let id = {
      let mut id_queue = ID_QUEUE.lock().unwrap();
      id_queue.pop_front().flatten()
    };
    let notification: crate::daemon::nf_struct::Notification = {
      let data_map = GLOBAL_DATA_MAP.lock().unwrap();
      data_map.get(&id).unwrap().clone()
    };
    let icon_path: Option<PathBuf> = None;
    // let icon_path: Option<PathBuf> = {
    //     let mut gtk_active = crate::shared_data::GTK_ACTIVE.lock().unwrap();
    //     if *gtk_active {
    //         find_icon_with_gtk(&notification.app_icon, 16)
    //     } else {
    //         None
    //     }
    // };
    (
      Self { 
        id,
        notification,
        icon_path,
      },
      Command::perform(Self::sleep_timer(5 as u64), |_| Message::Exit),
    )
  }

    fn namespace(&self) -> String {
        String::from("SimpleNotification - Iced")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        event::listen().map(Message::IcedEvent)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(event) => {
                match event {
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right)) => {
                        println!("Right mouse button pressed");
                        {
                            let mut data_map = GLOBAL_DATA_MAP.lock().unwrap();
                            data_map.remove(&self.id);
                        }
                        // the only way I found to kill needed layer without id
                        // messages do not contain info about ids of invokers of those messages
                        // possible solution to use stack in iced and set button as bg, but 
                        // did not find action on right click
                        return Command::done(Message::Exit);
                    }
                    iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                        println!("id: {:?}", self.id);
                        println!("Left mouse button pressed");
                    }
                    _ => {}
                }
                
                Command::none()
            }
            Message::Exit => {
                println!("Exit");
                {
                  let mut active_notifications = ACTIVE_NOTIFICATIONS.lock().unwrap();
                  *active_notifications -= 1;
                }
                return iced_runtime::task::effect(iced_runtime::Action::Exit);
            }
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<Message> {
        
        log::debug!("view");
        
        iced::widget::container(
            iced::widget::row![
                // take system icon name from notification and take icon from system
                // if let Some(self.notification.app_icon) = "firefox" {
                //     iced::widget::image::Image::new("firefox.png")
                // } else {
                //     iced::widget::image::Image::new("default.png")
                // },
                iced::widget::column![
                    iced::widget::text(self.notification.summary.clone()).size(30),
                    iced::widget::text(self.notification.body.clone()).size(20),
                ]
            ]
                .padding(10)
                //.center(800)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
            )
            .padding(10)
            .center(800)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(move |_| {

                iced::widget::container::Style {
                    //border: borders.border.rounded(iced::border::radius(10)),
                    //background_color: iced::Color::TRANSPARENT,
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
            background_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::parse("#ff0000").unwrap(),
        }
    }
}


//TODO: implement icon search or move that code to main to get initial icon path
// fn find_icon_with_gtk(icon_name: &str, size: i32) -> Option<PathBuf> {
//     let icon_theme = gtk::IconTheme::default().unwrap();
//     println!("icon_theme: {:?}", icon_theme);
//     if let Some(info) = icon_theme.lookup_icon(
//         icon_name,
//         size,
//         IconLookupFlags::FORCE_SIZE,
//     ) {
//         if let Some(filename) = info.filename() {
//             println!("filename: {:?}", filename);
//             return Some(PathBuf::from(filename));
//         }
//     }

//     None
// }