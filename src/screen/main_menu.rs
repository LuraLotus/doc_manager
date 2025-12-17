pub(crate) mod main_menu {
    use iced::{Alignment::Center, Element, Event, Length, Task, widget::{Container, Image, button, column, container, image::{Handle, Viewer}, row, text}};

    use crate::HOME_IMAGE;

    #[derive(Default, Debug, Clone)]
    pub(crate) struct MainMenu;
    
    impl MainMenu {
        pub(crate) fn new() -> MainMenu {
            MainMenu
        }
        pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::ToDocumentList => Task::none(),
                Message::NewDocument => todo!(),
                Message::KeyEvent(event) => Task::none(),
                Message::None => Task::none(),
            }
        }

        pub(crate) fn view(&self) -> Element<Message> {
            Container::new(
                Image::new(Handle::from_bytes(HOME_IMAGE))
                    .expand(true)
                    .content_fit(iced::ContentFit::Cover)
                    .border_radius(5.0)
            ).align_x(Center).align_y(Center).width(Length::Fill).height(Length::Fill).style(container::bordered_box).into()
        }
        
    }
    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        ToDocumentList,
        NewDocument,
        KeyEvent(Event),
        None,
    }

}
