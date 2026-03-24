#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db;
mod screen;
mod document;
mod attachment;
mod attachment_page;

use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime};

use iced::alignment::Horizontal::{self, Left};
use iced::widget::image::Handle;
use iced::{Alignment, Border, Color, Element, Length, Renderer, Subscription, Task, Theme, window};
use iced::widget::{Button, Column, Container, Image, Row, Space, Text, button, column, container, row, rule};
use iced_anim::animated::Mode;
use iced_anim::{Animated, Animation, AnimationBuilder, Easing};
use iced_aw::DropDown;
use iced_aw::core::offset;
use iced_aw::drop_down::{self, Offset};
use iced_aw::sidebar::TabLabel;
use iced_aw::style::{card, sidebar};
use iced_aw::widget::Sidebar;
use include_dir::{Dir, include_dir};
use log::{LevelFilter, error, warn};
use log4rs::append::file::FileAppender;
use log4rs::append::rolling_file::policy::compound::trigger::onstartup::OnStartUpTrigger;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::{RollingFileAppender, policy};
use log4rs::append::rolling_file::policy::Policy;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use screen::main_menu::main_menu;
use screen::document_list::document_list;
use screen::recycle_bin::recycle_bin;
use screen::settings::settings;
use serde::{Deserialize, Serialize};
use time::macros::format_description;
use time::{OffsetDateTime, UtcDateTime, format_description};

use crate::screen::{MainMenu, RecycleBin};
use crate::screen::DocumentList;
use crate::screen::Settings;

const ERROR_FERRIS: &[u8] = include_bytes!("../assets/ferris-error-handling.webp");
const HOME_IMAGE: &[u8] = include_bytes!("../assets/home.jpg");
const ICON: &[u8] = include_bytes!("../assets/icon.png");
#[cfg(target_os = "windows")]
const PDFIUM: &[u8] = include_bytes!("../resources/pdfium.dll");
#[cfg(target_os = "linux")]
const PDFIUM: &[u8] = include_bytes!("../resources/libpdfium.so");
const NAPS2: Dir<'_> = include_dir!("./resources/naps2");


pub fn main() -> iced::Result {
    init_logger();
    init_pdfium();
    init_naps2();

    iced::application(State::new, State::update, State::view)
    .title(State::window_title)
    .theme(State::current_theme)
    .subscription(State::subscription)
    .window(window::Settings {
        position: window::Position::Centered,
        icon: Some(window::icon::from_file_data(ICON, None).unwrap()),
        maximized: true,
        ..window::Settings::default()
    })
    .run()
}

fn init_logger() {
    let datetime = OffsetDateTime::from(SystemTime::now());
    let date_format = format_description!("[year]-[month]-[day]");
    let offset_date = datetime.to_offset(OffsetDateTime::now_local().expect("Error applying date offset").offset()).format(date_format).unwrap();
    let roller = FixedWindowRoller::builder()
        .build(format!("./logs/{}_{{}}.log", offset_date).as_str(), 3)
        .unwrap();
    // let trigger = SizeTrigger::new(10 * 1024 * 1024);
    let trigger = OnStartUpTrigger::new(0);
    let policy = CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {M} {L} [{l}] {m}\n")))
        .build(format!("./logs/current.log"), Box::new(policy))
        .unwrap();

    let log_config = log4rs::Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Warn))
        .unwrap();

    log4rs::init_config(log_config).unwrap();
}

fn init_pdfium() {
    if cfg!(target_os = "windows") {
        if !fs::exists("./pdfium.dll").unwrap() {
            match fs::write("./pdfium.dll", &PDFIUM) {
                Err(err) => {
                    error!("Error writing Pdfium bytes to file: {}", err);
                },
                _ => {}
            }
        }
    }
    else if cfg!(target_os = "linux") {
        if !fs::exists("./libpdfium.so").unwrap() {
            match fs::write("./libpdfium.so", &PDFIUM) {
                Err(err) => {
                    error!("Error writing Pdfium bytes to file: {}", err);
                },
                _ => {}
            }
        }
    }
}

fn init_naps2() {
    if !fs::exists("./naps2").unwrap() {
        match fs::create_dir("./naps2") {
            Err(err) => {
                error!("Error creating NAPS2 directory: {}", err);
            },
            _ => {}
        }
        match NAPS2.extract("./naps2") {
            Err(err) => {
                error!("Error extracting NAPS2 files to directory: {}", err);
            },
            _ => {}
        }
    }
}


#[derive(Debug, Clone)]
enum Message {
    SelectedTab(Tab),
    MainMenu(main_menu::Message),
    DocumentList(document_list::Message),
    RecycleBin(recycle_bin::Message),
    Settings(settings::Message),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, Copy)]
pub(crate) enum Tab {
    Home,
    #[default]
    DocumentList,
    RecycleBin,
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
    delete_period: u16,
    show_console: bool
}

impl Config {
    fn new() -> Config {
        let mut toml = String::new();
        match fs::read_to_string("./config.toml") {
            Ok(string) => {
                toml = string;
            },
            Err(err) => {
                error!("Error reading config file: {}", err);
                let serialized = match toml::to_string(&Config::default()) {
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
            }
        }
        
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

    fn change_delete_period(&mut self, period: u16) {
        self.delete_period = period;
    }

    fn current_theme(&self) -> LocalTheme {
        return self.current_theme.clone()
    }

    fn delete_period(&self) -> u16 {
        return self.delete_period
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            current_theme: LocalTheme::from(Theme::CatppuccinMacchiato),
            delete_period: 30,
            show_console: false
        }
    }
}

struct State {
    current_tab: Tab,
    main_menu: MainMenu,
    document_list: DocumentList,
    recycle_bin: RecycleBin,
    recycle_bin_tab: recycle_bin::Tab,
    settings: Settings,
    config: Config,
    previous_tab: Option<Tab>,
    show_sidebar: bool,
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
        settings.set_delete_period(config.delete_period());
        let mut state = State {
            current_tab: Tab::default(),
            main_menu: MainMenu::new(),
            document_list: DocumentList::new(),
            recycle_bin: RecycleBin::new(),
            recycle_bin_tab: recycle_bin::Tab::Document,
            settings,
            config,
            previous_tab: None,
            show_sidebar: true,
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
                        self.document_list.reset_state();
                        self.document_list.refresh_data();
                        self.document_list.reset_attachment_state();
                        self.current_tab = tab;
                    },
                    Tab::RecycleBin => {
                        if tab != self.current_tab {
                            self.previous_tab = Some(self.current_tab);
                        }
                        self.recycle_bin.set_current_theme(self.config.current_theme().into());
                        self.recycle_bin.reset_state();
                        self.recycle_bin.refresh_data();
                        self.recycle_bin_tab = recycle_bin::Tab::Document;
                        self.current_tab = tab;
                    }
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
                    },
                    document_list::Message::OpenFullImageViewer => {
                        self.show_sidebar = false;
                        return self.document_list.update(document_list_message).map(Message::DocumentList)
                    },
                    document_list::Message::CloseFullImageViewer => {
                        self.show_sidebar = true;
                        return self.document_list.update(document_list_message).map(Message::DocumentList)
                    },
                    _ => {
                        return self.document_list.update(document_list_message).map(Message::DocumentList)
                    }
                }
            },
            Message::RecycleBin(recycle_bin_message) => {
                match recycle_bin_message {
                    recycle_bin::Message::Back => {
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
                    },
                    recycle_bin::Message::SwitchTab(tab) => {
                        self.recycle_bin_tab = tab;
                        return self.recycle_bin.update(recycle_bin_message).map(Message::RecycleBin)
                    },
                    _ => {
                        return self.recycle_bin.update(recycle_bin_message).map(Message::RecycleBin)
                    }
                }
            },
            Message::Settings(settings_message) => {
                match &settings_message {
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
                    },
                    settings::Message::ChangeDeletePeriod(period) => {
                        let period_string: String = period.chars().filter(|c| c.is_numeric()).collect();
                        match period_string.parse() {
                            Ok(period) => {
                                self.config.change_delete_period(period);
                            }
                            Err(err) => {
                                error!("Error parsing delete period: {}", err);
                            }
                        }

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
        let screen = match &self.current_tab {
            Tab::Home => self.main_menu.view().map(Message::MainMenu),
            Tab::DocumentList => self.document_list.view().map(Message::DocumentList),
            Tab::RecycleBin => self.recycle_bin.view().map(Message::RecycleBin),
            Tab::Settings => self.settings.view().map(Message::Settings)
        };
        let mut content = Row::new().spacing(5);
        if self.show_sidebar {
            content = content.push(Container::new(
                    sidebar(self, self.current_tab)
                ).width(Length::FillPortion(1))
            );
        }
        content = content.push(container(screen).width(Length::FillPortion(5)));
        Container::new(row![
            content
        ].spacing(5)).padding(5).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.current_tab {
            Tab::DocumentList => {
                self.document_list.subscription().map(Message::DocumentList)
            },
            Tab::RecycleBin => {
                Subscription::none()
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

fn sidebar(state: &State, selected_tab: Tab) -> Element<'static, Message> {
    let recycle_bin_tab = state.recycle_bin_tab;
    let mut column: Column<'_, Message, Theme, Renderer> = Column::new().spacing(5).align_x(Horizontal::Left);
    column = column.push(
        button(Text::from("Document List").size(18)).on_press(Message::SelectedTab(Tab::DocumentList)).width(Length::Fill).style(move |theme: &Theme, status|
            if selected_tab == Tab::DocumentList {
                sidebar_button_selected_style(theme)
            }
            else {
                sidebar_button_style(theme, status)
            }
        )
    );
    column = column.push(
        button(row![
            Text::from("Recycle Bin").size(18),
            Space::new().width(Length::Fill),
            Text::from("▼").size(8).height(Length::Fill).center()
        ]).on_press(Message::SelectedTab(Tab::RecycleBin)).width(Length::Fill).height(Length::Shrink).style(move |theme: &Theme, status|
            if selected_tab == Tab::RecycleBin {
                sidebar_button_selected_style(theme)
            }
            else {
                sidebar_button_style(theme, status)
            }
        )
    );
    if state.current_tab == Tab::RecycleBin {
        column = column.push(
            Container::new(
                column![
                    button(Text::from("     Documents").size(16)).width(Length::Fill).on_press(Message::RecycleBin(recycle_bin::Message::SwitchTab(recycle_bin::Tab::Document))).style(move |theme: &Theme, status|
                        if recycle_bin_tab == recycle_bin::Tab::Document {
                            sidebar_button_selected_style(theme)
                        }
                        else {
                            sidebar_button_style(theme, status)
                        }
                    ),
                    button(Text::from("     Attachments").size(16)).width(Length::Fill).on_press(Message::RecycleBin(recycle_bin::Message::SwitchTab(recycle_bin::Tab::Attachment))).style(move |theme: &Theme, status|
                        if recycle_bin_tab == recycle_bin::Tab::Attachment {
                            sidebar_button_selected_style(theme)
                        }
                        else {
                            sidebar_button_style(theme, status)
                        }
                    )
                ].spacing(5)
            )
        );
    }
    column = column.push(
        button(Text::from("Settings").size(18)).on_press(Message::SelectedTab(Tab::Settings)).width(Length::Fill).style(move |theme: &Theme, status|
            if selected_tab == Tab::Settings {
                sidebar_button_selected_style(theme)
            }
            else {
                sidebar_button_style(theme, status)
            }
        )
    );

    Container::new(
        column
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
