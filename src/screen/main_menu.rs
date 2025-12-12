pub(crate) mod main_menu {
    use iced::{Element, Event, Length, Task, widget::{Container, button, column, container, row, text}};

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
            Container::new(row![
                column![
                    text("Main Menu"),
                    text("Ignore this screen").size(20),
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
