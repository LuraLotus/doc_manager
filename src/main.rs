mod db;
mod screen;

use iced::{Element, Subscription, Task, Theme};
use iced::widget::{Container, container, row};
use iced_aw::sidebar::TabLabel;
use iced_aw::widget::Sidebar;
use screen::main_menu::main_menu;
use screen::document_list::document_list;

use crate::screen::MainMenu;
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

struct State {
    current_tab: Tab,
    main_menu: MainMenu,
    document_list: DocumentList,
    current_theme: Theme,
    search_text: String,
}

impl State {
    fn new() -> State {
        State {
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
                    },
                    Tab::DocumentList => {
                        self.current_tab = tab;
                    },
                }
            },
            Message::MainMenu(main_menu_message) => {
                match main_menu_message {
                    _ => {
                        return self.main_menu.update(main_menu_message).map(Message::MainMenu)
                    }
                }
            },
            Message::DocumentList(document_list_message) => {
                match document_list_message {
                    document_list::Message::Back => {
                        self.current_tab = Tab::Home;
                    }
                    _ => {
                        //let Screen::DocumentList(screen) = &mut self.current_screen else { return Task::none(); };
                        return self.document_list.update(document_list_message).map(Message::DocumentList)
                    }
                }
            },
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let screen = match &self.current_tab {
            Tab::Home => self.main_menu.view().map(Message::MainMenu),
            Tab::DocumentList => self.document_list.view().map(Message::DocumentList),
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
        match &self.current_tab {
            Tab::DocumentList => {
                self.document_list.subscription().map(Message::DocumentList)
            }
            Tab::Home => {
                Subscription::none()
            },
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}
