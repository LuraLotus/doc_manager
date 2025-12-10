mod db;
mod screen;

use iced::alignment::Horizontal::Left;
use iced::{Element, Subscription, Task, Theme};
use iced::widget::{Container, container, row};
use iced_aw::sidebar::TabLabel;
use iced_aw::widget::Sidebar;
use screen::main_menu::main_menu;
use screen::document_list::document_list;
use screen::settings::settings;

use crate::screen::MainMenu;
use crate::screen::DocumentList;
use crate::screen::Settings;


pub fn main() -> iced::Result {
    // let conn: Result<Connection, rusqlite::Error> = db_init();
    iced::application(State::new, State::update, State::view)
    .theme(State::current_theme)
    .subscription(State::subscription)
    .run()
}


#[derive(Debug, Clone)]
enum Message {
    SelectedTab(Tab),
    MainMenu(main_menu::Message),
    DocumentList(document_list::Message),
    Settings(settings::Message)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, Copy)]
    pub(crate) enum Tab {
        #[default]
        Home,
        DocumentList,
        Settings
    }

struct State {
    current_tab: Tab,
    main_menu: MainMenu,
    document_list: DocumentList,
    settings: Settings,
    current_theme: Theme,
    search_text: String,
}

impl State {
    fn new() -> State {
        let initial_theme = Theme::CatppuccinMacchiato;
        let mut settings = Settings::new();
        settings.set_theme(initial_theme.clone());
        State {
            current_tab: Tab::default(),
            main_menu: MainMenu::new(),
            document_list: DocumentList::new(),
            settings,
            current_theme: initial_theme,
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
                    Tab::Settings => {
                        self.current_tab = tab;
                    }
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
            Message::Settings(settings_message) => {
                match settings_message {
                    settings::Message::ChangeTheme(theme) => {
                        self.current_theme = theme.clone();
                        self.settings.set_theme(theme);
                    }
                    _ => {
                        return self.settings.update(settings_message).map(Message::Settings)
                    }
                }
            }
            
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let screen = match &self.current_tab {
            Tab::Home => self.main_menu.view().map(Message::MainMenu),
            Tab::DocumentList => self.document_list.view().map(Message::DocumentList),
            Tab::Settings => self.settings.view().map(Message::Settings)
        };
        Container::new(row![
            Sidebar::new(Message::SelectedTab)
                .push(Tab::Home, TabLabel::Text(String::from("Home")))
                .push(Tab::DocumentList, TabLabel::Text(String::from("Document List")))
                .push(Tab::Settings, TabLabel::Text(String::from("Settings")))
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
            Tab::Settings => {
                Subscription::none()
            }
        }
    }

    fn current_theme(&self) -> Theme {
        self.current_theme.clone()
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}
