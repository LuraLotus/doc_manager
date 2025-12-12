mod db;
mod screen;

use std::fs;
use std::path::Path;

use iced::alignment::Horizontal::Left;
use iced::{Element, Subscription, Task, Theme};
use iced::widget::{Container, container, row, rule};
use iced_aw::sidebar::TabLabel;
use iced_aw::style::card;
use iced_aw::widget::Sidebar;
use screen::main_menu::main_menu;
use screen::document_list::document_list;
use screen::settings::settings;
use serde::{Deserialize, Serialize};

use crate::screen::{MainMenu, document};
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) enum LocalTheme {
    Light,
    Dark,
    Dracula,
    Nord,
    SolarizedLight,
    SolarizedDark,
    GruvboxLight,
    GruvboxDark,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightLight,
    KanagawaWave,
    KanagawaDragon,
    KanagawaLotus,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
    Unknown
}

impl From<Theme> for LocalTheme {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => LocalTheme::Light,
            Theme::Dark => LocalTheme::Dark,
            Theme::Dracula => LocalTheme::Dracula,
            Theme::Nord => LocalTheme::Nord,
            Theme::SolarizedLight => LocalTheme::SolarizedLight,
            Theme::SolarizedDark => LocalTheme::SolarizedDark,
            Theme::GruvboxLight => LocalTheme::GruvboxLight,
            Theme::GruvboxDark => LocalTheme::GruvboxDark,
            Theme::CatppuccinLatte => LocalTheme::CatppuccinLatte,
            Theme::CatppuccinFrappe => LocalTheme::CatppuccinFrappe,
            Theme::CatppuccinMacchiato => LocalTheme::CatppuccinMacchiato,
            Theme::CatppuccinMocha => LocalTheme::CatppuccinMocha,
            Theme::TokyoNight => LocalTheme::TokyoNight,
            Theme::TokyoNightStorm => LocalTheme::TokyoNightStorm,
            Theme::TokyoNightLight => LocalTheme::TokyoNightLight,
            Theme::KanagawaWave => LocalTheme::KanagawaWave,
            Theme::KanagawaDragon => LocalTheme::KanagawaDragon,
            Theme::KanagawaLotus => LocalTheme::KanagawaLotus,
            Theme::Moonfly => LocalTheme::Moonfly,
            Theme::Nightfly => LocalTheme::Nightfly,
            Theme::Oxocarbon => LocalTheme::Oxocarbon,
            Theme::Ferra => LocalTheme::Ferra,
            Theme::Custom(custom) => LocalTheme::Unknown,
        }
    }
}

impl Into<Theme> for LocalTheme {
    fn into(self) -> Theme {
        match self {
            LocalTheme::Light => Theme::Light,
            LocalTheme::Dark => Theme::Dark,
            LocalTheme::Dracula => Theme::Dracula,
            LocalTheme::Nord => Theme::Nord,
            LocalTheme::SolarizedLight => Theme::SolarizedLight,
            LocalTheme::SolarizedDark => Theme::SolarizedDark,
            LocalTheme::GruvboxLight => Theme::GruvboxLight,
            LocalTheme::GruvboxDark => Theme::GruvboxDark,
            LocalTheme::CatppuccinLatte => Theme::CatppuccinLatte,
            LocalTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
            LocalTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
            LocalTheme::CatppuccinMocha => Theme::CatppuccinMocha,
            LocalTheme::TokyoNight => Theme::TokyoNight,
            LocalTheme::TokyoNightStorm => Theme::TokyoNightStorm,
            LocalTheme::TokyoNightLight => Theme::TokyoNightLight,
            LocalTheme::KanagawaWave => Theme::KanagawaWave,
            LocalTheme::KanagawaDragon => Theme::KanagawaDragon,
            LocalTheme::KanagawaLotus => Theme::KanagawaLotus,
            LocalTheme::Moonfly => Theme::Moonfly,
            LocalTheme::Nightfly => Theme::Nightfly,
            LocalTheme::Oxocarbon => Theme::Oxocarbon,
            LocalTheme::Ferra => Theme::Ferra,
            _ => Theme::Dark
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    current_theme: LocalTheme
}

impl Config {
    fn new() -> Config {
        let toml = fs::read_to_string("./config.toml").unwrap_or_else(|err| {
            println!("Error reading config file: {}", err);
            String::new()
        });
        match toml::from_str::<Config>(&toml) {
            Ok(config) => config,
            Err(err) => {
                println!("Error deserializing config file: {}", err);
                Config::default()
            },
        }
    }

    fn change_theme(&mut self, theme: Theme) {
        self.current_theme = LocalTheme::from(theme);
    }

    fn current_theme(&self) -> LocalTheme {
        return self.current_theme.clone()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            current_theme: LocalTheme::from(Theme::CatppuccinMacchiato)
        }
    }
}

struct State {
    current_tab: Tab,
    main_menu: MainMenu,
    document_list: DocumentList,
    settings: Settings,
    config: Config,
    previous_tab: Option<Tab>
}

impl State {
    fn new() -> State {
        let config = Config::new();
        let initial_theme = config.current_theme();
        let mut settings = Settings::new();
        settings.set_theme(
            match initial_theme {
                LocalTheme::Light => Theme::Light,
                LocalTheme::Dark => Theme::Dark,
                LocalTheme::Dracula => Theme::Dracula,
                LocalTheme::Nord => Theme::Nord,
                LocalTheme::SolarizedLight => Theme::SolarizedLight,
                LocalTheme::SolarizedDark => Theme::SolarizedDark,
                LocalTheme::GruvboxLight => Theme::GruvboxLight,
                LocalTheme::GruvboxDark => Theme::GruvboxDark,
                LocalTheme::CatppuccinLatte => Theme::CatppuccinLatte,
                LocalTheme::CatppuccinFrappe => Theme::CatppuccinFrappe,
                LocalTheme::CatppuccinMacchiato => Theme::CatppuccinMacchiato,
                LocalTheme::CatppuccinMocha => Theme::CatppuccinMocha,
                LocalTheme::TokyoNight => Theme::TokyoNight,
                LocalTheme::TokyoNightStorm => Theme::TokyoNightStorm,
                LocalTheme::TokyoNightLight => Theme::TokyoNightLight,
                LocalTheme::KanagawaWave => Theme::KanagawaWave,
                LocalTheme::KanagawaDragon => Theme::KanagawaDragon,
                LocalTheme::KanagawaLotus => Theme::KanagawaLotus,
                LocalTheme::Moonfly => Theme::Moonfly,
                LocalTheme::Nightfly => Theme::Nightfly,
                LocalTheme::Oxocarbon => Theme::Oxocarbon,
                LocalTheme::Ferra => Theme::Ferra,
                _ => Theme::Dark
            }
        );
        State {
            current_tab: Tab::default(),
            main_menu: MainMenu::new(),
            document_list: DocumentList::new(),
            settings,
            config,
            previous_tab: None
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectedTab(tab) => {
                match tab {
                    Tab::Home => {
                        if tab != self.current_tab {
                            self.previous_tab = Some(self.current_tab);
                        }
                        self.current_tab = tab;
                    },
                    Tab::DocumentList => {
                        if tab != self.current_tab {
                            self.previous_tab = Some(self.current_tab);
                        }
                        self.document_list.set_current_theme(self.config.current_theme().into());
                        self.current_tab = tab;
                    },
                    Tab::Settings => {
                        if tab != self.current_tab {
                            self.previous_tab = Some(self.current_tab);
                        }
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
                        if self.current_tab != self.previous_tab.unwrap() {
                            self.current_tab = self.previous_tab.unwrap_or_else(|| {
                                println!("No previous tab");
                                self.current_tab
                            });
                        }
                        else {
                            self.current_tab = Tab::Home;
                        }
                        
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
                        self.config.change_theme(theme.clone());
                        let serialized = toml::to_string(&self.config).unwrap_or_else(|err| {
                            println!("Error serializing config to toml: {}", err);
                            String::new()
                        });
                        fs::write("./config.toml", serialized).unwrap_or_else(|err| {
                            println!("Error writing to config file: {}", err);
                        });
                        self.settings.set_theme(theme.clone());
                        self.document_list.set_current_theme(theme.clone().into());
                    }
                    settings::Message::Back => {
                        if self.current_tab != self.previous_tab.unwrap() {
                            self.current_tab = self.previous_tab.unwrap_or_else(|| {
                                println!("No previous tab");
                                self.current_tab
                            });
                        } else {
                            self.current_tab = Tab::Home;
                        }
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
        ].spacing(5)).into()
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
        self.config.current_theme().into()
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localtheme_from() {
        let theme = LocalTheme::from(Theme::Dark);
        assert_eq!(theme, LocalTheme::Dark);
    }

    #[test]
    fn test_localtheme_into() {
        assert_eq!(Into::<Theme>::into(LocalTheme::Dark), Theme::Dark);
    }

    #[test]
    fn test_config() {
        let config = Config::new();
        assert!(fs::exists("./config.toml").expect("File not found"));
        assert_eq!(config.current_theme(), LocalTheme::from(Theme::CatppuccinMacchiato));

    }
}
