mod db;
mod screen;

use db::db_module::DbConnection;
use iced::advanced::graphics::text::cosmic_text::Command;
use iced::{Application, Element, Event, Length, Subscription, Task, Theme, event, keyboard};
use iced::widget::{Container, button, column, container, row, text, text_input};
use iced_aw::sidebar::TabLabel;
use iced_aw::widget::Sidebar;
use rusqlite::Connection;
use screen::main_menu::main_menu;
use screen::document_list::document_list;

use crate::screen::{Document, MainMenu, document};
use crate::screen::DocumentList;


pub fn main() -> iced::Result {
    // let conn: Result<Connection, rusqlite::Error> = db_init();
    iced::application("Doc Manager", State::update, State::view)
    .theme(|_doc_manager| Theme::Nightfly)
    .subscription(State::subscription)
    .run()
}


#[derive(Debug, Clone)]
enum Message {
    SelectedTab(Tab),
    MainMenu(main_menu::Message),
    DocumentList(document_list::Message),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, Copy)]
    pub(crate) enum Tab {
        #[default]
        Home,
        DocumentList,
    }

#[derive(Debug, Clone)]
enum Screen {
    MainMenu(screen::MainMenu),
    DocumentList(screen::DocumentList),
}

impl Default for Screen {
    fn default() -> Self {
        Screen::MainMenu(screen::MainMenu)
    }
}

#[derive(Default)]
struct State {
    current_screen: Screen,
    current_tab: Tab,
    main_menu: MainMenu,
    document_list: DocumentList,
    current_theme: Theme,
    search_text: String,
}

impl State {
    fn new() -> State {
        State {
            current_screen: Screen::default(),
            current_tab: Tab::default(),
            main_menu: MainMenu::new(),
            document_list: DocumentList::new(),
            current_theme: Theme::default(),
            search_text: String::default(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectedTab(tab) => {
                match tab {
                    Tab::Home => {
                        self.current_tab = tab;
                        self.current_screen = Screen::MainMenu(screen::MainMenu::new());
                    },
                    Tab::DocumentList => {
                        self.current_tab = tab;
                        self.current_screen = Screen::DocumentList(screen::DocumentList::new());
                    },
                }
            },
            Message::MainMenu(main_menu_message) => {
                match main_menu_message {
                    _ => {
                        let Screen::MainMenu(screen) = &mut self.current_screen else { return Task::none(); };
                        screen.update(main_menu_message);
                    }
                }
            },
            Message::DocumentList(document_list_message) => {
                match document_list_message {
                    document_list::Message::Back => {
                        self.current_tab = Tab::Home;
                        self.current_screen = Screen::MainMenu(screen::MainMenu::new());
                    }
                    _ => {
                        let Screen::DocumentList(screen) = &mut self.current_screen else { return Task::none(); };
                        let _ = screen.update(document_list_message);
                    }
                }
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let screen = match &self.current_screen {
            Screen::MainMenu(main_menu) => main_menu.view().map(Message::MainMenu),
            Screen::DocumentList(document_list) => document_list.view().map(Message::DocumentList),
        };
        Container::new(row![
            Sidebar::new(Message::SelectedTab)
                .push(Tab::Home, TabLabel::Text(String::from("Home")))
                .push(Tab::DocumentList, TabLabel::Text(String::from("Document List")))
                .align_tabs(iced::Alignment::Start)
                .set_active_tab(&self.current_tab),
            container(screen)
        ]).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.current_screen {
            Screen::DocumentList(document_list) => {
                document_list.subscription().map(Message::DocumentList)
            }
            Screen::MainMenu(main_menu) => {
                Subscription::none()
            },
        }
    }

}
