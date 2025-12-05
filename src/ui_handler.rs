pub(crate) mod ui_handler {
    use std::default;

    use iced::{Element, widget::Column, widget::column, widget::text};
    use iced::{Application, Result, Theme, application};

    use crate::screen::main_menu::main_menu::MainMenu;
    use crate::screen::main_menu::main_menu;
    use crate::screen;

    pub(crate) fn run_app() -> iced::Result {
        iced::application("Doc Manager", State::update, State::view)
        .theme(|_doc_manager| Theme::TokyoNightStorm)
        .run()
    }

    #[derive(Default)]
    pub(crate) struct State {
        screen: Screens,
    }

    impl State {
        fn new() -> Self {
            Self {
                screen: Screens::MainMenu(()),
            }
        }

        fn default() -> Self{
            Self {
                screen: Screens::MainMenu(()),
            }
        }

        fn switch_screen(&mut self, target_screen: Screens) {
            match target_screen {
                Screens::MainMenu => {},
                Screens::DocumentList => {},
                Screens::NewDocument => {},
                Screens::EditDocument => {},
                Screens::NewAttachment => {},
                Screens::EditAttachment => {},
            }
        }

        fn update(&mut self, message: Message) {
            self.screen = match message {
                Message::EnterMainMenu => Screens::MainMenu,
            }
        }

        fn view(&self) -> Element<Message> {
            match &self.screen {
                Screens::MainMenu(main_menu) => MainMenu::view(state),
                Screens::DocumentList | Screens::NewDocument | Screens::EditDocument | Screens::NewAttachment | Screens::EditAttachment => {},

            }
        }

        fn display_screen(&self) {
            
        }
    }


    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        EnterMainMenu,
        MainMenu(main_menu::Message),
    }

    #[derive(Default)]
    pub(crate) enum Screens {
        #[default]
        MainMenu(MainMenu),
        DocumentList,
        NewDocument,
        EditDocument,
        NewAttachment,
        EditAttachment,
    }

    struct DocumentList {

    }

    struct NewDocument {

    }
}