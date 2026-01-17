#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db;
mod screen;
mod document;
mod attachment;
mod attachment_page;

use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime};

use hide_console_ng::hide_console;
use iced::alignment::Horizontal::Left;
use iced::{Border, Color, Element, Length, Subscription, Task, Theme};
use iced::widget::{Button, Column, Container, Text, button, column, container, row, rule};
use iced_anim::animated::Mode;
use iced_anim::{Animated, Animation, AnimationBuilder, Easing};
use iced_aw::core::offset;
use iced_aw::drop_down::Offset;
use iced_aw::sidebar::TabLabel;
use iced_aw::style::{card, sidebar};
use iced_aw::widget::Sidebar;
use log::{LevelFilter, error, warn};
use log4rs::append::file::FileAppender;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::{RollingFileAppender, policy};
use log4rs::append::rolling_file::policy::Policy;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use screen::main_menu::main_menu;
use screen::document_list::document_list;
use screen::settings::settings;
use serde::{Deserialize, Serialize};
use time::macros::format_description;
use time::{OffsetDateTime, UtcDateTime, format_description};

use crate::screen::{MainMenu};
use crate::screen::DocumentList;
use crate::screen::Settings;

const ERROR_FERRIS: &[u8] = include_bytes!("../ferris-error-handling.webp");
const HOME_IMAGE: &[u8] = include_bytes!("../home.jpg");


pub fn main() -> iced::Result {
    init_logger();

    iced::application(State::new, State::update, State::view)
    .title(State::window_title)
    .theme(State::current_theme)
    .subscription(State::subscription)
    .run()
}

fn init_logger() {
    let datetime = OffsetDateTime::from(SystemTime::now());
    let date_format = format_description!("[year]-[month]-[day]");
    let offset_date = datetime.to_offset(OffsetDateTime::now_local().expect("Error applying date offset").offset()).format(date_format).unwrap();
    let roller = FixedWindowRoller::builder()
        .build(format!("./logs/{}_{{}}.log", offset_date).as_str(), 5)
        .unwrap();
    let trigger = SizeTrigger::new(10 * 1024 * 1024);
    let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}\n")))
        .build(format!("./logs/{}.log", offset_date), Box::new(policy))
        .unwrap();

    let log_config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Warn))
        .unwrap();

    log4rs::init_config(log_config).unwrap();
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
    Home,
    #[default]
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

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    current_theme: LocalTheme,
    show_console: bool
}

impl Config {
    fn new() -> Config {
        let toml = fs::read_to_string("./config.toml").unwrap_or_else(|err| {
            error!("Error reading config file: {}", err);
            String::new()
        });
        match toml::from_str::<Config>(&toml) {
            Ok(config) => config,
            Err(err) => {
                error!("Error deserializing config file: {}", err);
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

    fn show_console(&self) {
        match self.show_console {
            true => {
                hide_console_ng::show_unconditionally();
            }
            false => {
                hide_console();
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            current_theme: LocalTheme::from(Theme::CatppuccinMacchiato),
            show_console: false
        }
    }
}

struct State {
    current_tab: Tab,
    main_menu: MainMenu,
    document_list: DocumentList,
    settings: Settings,
    config: Config,
    previous_tab: Option<Tab>,
    sidebar_button_color: Color
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
        let mut state = State {
            current_tab: Tab::default(),
            main_menu: MainMenu::new(),
            document_list: DocumentList::new(),
            settings,
            config,
            previous_tab: None,
            sidebar_button_color: Color::default()
        };
        state.document_list.set_current_theme(initial_theme);
        return state
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
                    main_menu::Message::DocumentList => {
                        self.previous_tab = Some(Tab::DocumentList);
                        self.document_list.set_current_theme(self.config.current_theme().into());
                        self.current_tab = Tab::DocumentList;
                    },
                    main_menu::Message::Settings => {
                        self.previous_tab = Some(Tab::Settings);
                        self.current_tab = Tab::Settings;
                    }
                    _ => {
                        return self.main_menu.update(main_menu_message).map(Message::MainMenu)
                    }
                }
            },
            Message::DocumentList(document_list_message) => {
                match document_list_message {
                    document_list::Message::Back => {
                        if self.previous_tab.is_some() {
                            if self.current_tab != self.previous_tab.unwrap() && self.current_tab != Tab::DocumentList {
                                self.current_tab = self.previous_tab.unwrap_or_else(|| {
                                    self.current_tab
                                });
                            }
                            else {
                                self.current_tab = Tab::DocumentList;
                            }
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
                        let serialized = match toml::to_string(&self.config) {
                            Ok(string) => string,
                            Err(err) => {
                                error!("Error serializing config to toml: {}", err);
                                String::new()
                            }
                        };
                        match fs::write("./config.toml", serialized) {
                            Err(err) => {
                                error!("Error writing to config file: {}", err);
                            },
                            _ => {}
                        };
                        self.settings.set_theme(theme.clone());
                        self.document_list.set_current_theme(theme.clone().into());
                    }
                    settings::Message::ShowConsole(show_console) => {
                        self.config.show_console = show_console;
                        self.config.show_console();
                        let serialized = match toml::to_string(&self.config) {
                            Ok(string) => string,
                            Err(err) => {
                                error!("Error serializing config to toml: {}", err);
                                String::new()
                            }
                        };
                        match fs::write("./config.toml", serialized) {
                            Err(err) => {
                                error!("Error writing to config file: {}", err);
                            },
                            _ => {}
                        };
                        return self.settings.update(settings_message).map(Message::Settings)
                    }
                    settings::Message::Back => {
                        if self.previous_tab.is_some() {
                            if self.current_tab != self.previous_tab.unwrap() && self.current_tab != Tab::DocumentList {
                                self.current_tab = self.previous_tab.unwrap_or_else(|| {
                                    self.current_tab
                                });
                            }
                            else {
                                self.current_tab = Tab::DocumentList;
                            }
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
        self.config.show_console();
        let screen = match &self.current_tab {
            Tab::Home => self.main_menu.view().map(Message::MainMenu),
            Tab::DocumentList => self.document_list.view().map(Message::DocumentList),
            Tab::Settings => self.settings.view().map(Message::Settings)
        };
        Container::new(row![
            Container::new(
                sidebar(self.current_tab)
            ).width(Length::FillPortion(1)),
            container(screen).width(Length::FillPortion(5))
        ].spacing(5)).padding(5).into()
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

    fn window_title(&self) -> String {
        format!("DocManager {}", env!("CARGO_PKG_VERSION"))
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

fn sidebar(selected_tab: Tab) -> Element<'static, Message> {
    Container::new(
        column![
            // button(Text::from("Home").size(18)).on_press(Message::SelectedTab(Tab::Home)).width(Length::Fill).style(move |theme: &Theme, status| 
            //     if selected_tab == Tab::Home {
            //         sidebar_button_selected_style(theme)
            //     }
            //     else {
            //         sidebar_button_style(theme, status)
            //     }
            // ),
            button(Text::from("Document List").size(18)).on_press(Message::SelectedTab(Tab::DocumentList)).width(Length::Fill).style(move |theme: &Theme, status|
                if selected_tab == Tab::DocumentList {
                    sidebar_button_selected_style(theme)
                }
                else {
                    sidebar_button_style(theme, status)
                }
            ),
            button(Text::from("Settings").size(18)).on_press(Message::SelectedTab(Tab::Settings)).width(Length::Fill).style(move |theme: &Theme, status|
                if selected_tab == Tab::Settings {
                    sidebar_button_selected_style(theme)
                }
                else {
                    sidebar_button_style(theme, status)
                }
            )
        ].spacing(5).align_x(Left)
    ).into()
}

fn sidebar_button_style(theme: &Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
    match status {
        button::Status::Active => {
            iced::widget::button::Style {
                text_color: theme.extended_palette().background.weak.text.into(),
                background: Some(Color::TRANSPARENT.into()),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 5.0.into()
                },
                shadow: Default::default(),
                snap: false
            }
        },
        button::Status::Hovered => {
            iced::widget::button::Style {
                text_color: theme.extended_palette().background.weakest.text.into(),
                background: Some(theme.extended_palette().background.weakest.color.into()),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 5.0.into()
                },
                shadow: Default::default(),
                snap: false
            }
        },
        button::Status::Pressed => iced::widget::button::Style {
            text_color: theme.extended_palette().background.weaker.text.into(),
            background: Some(theme.extended_palette().background.weaker.color.into()),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 5.0.into()
            },
            shadow: Default::default(),
            snap: false
        },
        button::Status::Disabled => iced::widget::button::Style {
            text_color: theme.extended_palette().background.strong.text.into(),
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 5.0.into()
            },
            shadow: Default::default(),
            snap: false
        },
    }
}

fn sidebar_button_selected_style(theme: &Theme) -> iced::widget::button::Style {
    iced::widget::button::Style {
        text_color: theme.extended_palette().background.weak.text.into(),
        background: Some(theme.extended_palette().background.weak.color.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 5.0.into()
        },
        shadow: Default::default(),
        snap: false
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
