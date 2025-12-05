pub(crate) mod main_menu {
    use iced::{Element, Event, Length, Subscription, keyboard, widget::{Container, button, column, container, row, text}};
    use iced_aw::{sidebar::TabLabel, widget::Sidebar};

    #[derive(Default, Debug, Clone)]
    pub(crate) struct MainMenu;
    
    impl MainMenu {
        pub(crate) fn new() -> MainMenu {
            MainMenu
        }
        pub(crate) fn update(&mut self, message: Message) {
            match message {
                Message::ToDocumentList => {},
                Message::NewDocument => todo!(),
                Message::KeyEvent(event) => {},
                Message::None => {},
            }
        }

        pub(crate) fn view(&self) -> Element<Message> {
            Container::new(row![
                column![
                    text("Main Menu"),
                    button("Home").on_press(Message::None),
                    button("Document List").on_press(Message::ToDocumentList),
                    button("New Document").on_press(Message::NewDocument),
                    button("Settings").on_press(Message::None),
                    ].spacing(10)
                ]
            ).width(Length::Fill).height(Length::Fill).style(container::rounded_box).into()
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
