pub(crate) mod settings {
    use iced::{Alignment::Center, Color, Element, Length, Task, Theme, alignment::Horizontal::Left, widget::{Container, Grid, PickList, Space, Text, TextInput, Toggler, button, column, container, pick_list, rich_text, row, rule, span, text::Rich, toggler}};
    use iced_aw::Card;
    use log::error;

    pub(crate) struct Settings {
        current_theme: Option<Theme>,
        delete_period: u16,
        show_console: bool
    }

    impl Settings {
        pub(crate) fn new() -> Settings {
            Settings {
                current_theme: Some(Theme::CatppuccinMacchiato),
                delete_period: 30,
                show_console: false
            }
        }

        pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::ChangeTheme(theme) => {
                    self.current_theme = Some(theme);
                    Task::none()
                },
                Message::ChangeDeletePeriod(period) => {
                    let period_string: String = period.chars().filter(|c| c.is_numeric()).collect();
                    match period_string.parse() {
                        Ok(period) => {
                            self.delete_period = period;
                        },
                        Err(err) => {
                            if period_string.is_empty() {
                                self.delete_period = 0;
                            }
                            error!("Error parsing delete period: {}", err);
                        }
                    }
                    
                    Task::none()
                },
                Message::ShowConsole(show_console) => {
                    self.show_console = show_console;
                    println!("{}", show_console);
                    Task::none()
                },
                Message::OpenLink(link) => {
                    match opener::open(link) {
                        Err(err) => {
                            error!("Error opening link: {}", err);
                        },
                        _ => {}
                    };
                    Task::none()
                },
                Message::Back => Task::none()
            }
        }

        pub(crate) fn view(&self) -> Element<Message> {
            Container::new(column![
                Container::new(row![
                    button("<").on_press(Message::Back)
                ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill),
                Container::new(column![
                    row![
                        Text::new("Settings").size(20)
                    ].spacing(5).align_y(Center),
                    rule::horizontal(2),
                    row![
                        Text::new("Theme: ").align_y(Center),
                        PickList::new(Settings::available_themes(), self.current_theme.clone(), Message::ChangeTheme)
                    ].spacing(5).align_y(Center),
                    row![
                        Text::new("Permanent Delete Period: "),
                        TextInput::new("", self.delete_period.to_string().as_str()).on_input(Message::ChangeDeletePeriod).width(50)
                    ].spacing(5).align_y(Center),
                    Space::new().height(Length::Fill),
                    row![
                        Text::new("Scanning powered by "),
                        Rich::with_spans([
                            span("NAPS2").color(Color::from_rgb(0.2, 0.5, 1.0)).link("http://www.naps2.com")
                        ]).on_link_click(|link: String| Message::OpenLink(link))
                    ]
                    
                    // row![
                    //     Text::new("Show Console: "),
                    //     Toggler::new(self.show_console).on_toggle(Message::ShowConsole).size(18)
                    // ].spacing(5).align_y(Center)
                ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill),
            ].spacing(5)).into()
        }

        fn available_themes() -> Vec<Theme> {
            let mut available_themes: Vec<Theme> = Vec::new();

            for theme in Theme::ALL {
                available_themes.push(theme.clone());
            }

            return available_themes;
        }

        pub(crate) fn set_theme(&mut self, theme: Theme) {
            self.current_theme = Some(theme);
        }

        pub(crate) fn set_delete_period(&mut self, period: u16) {
            self.delete_period = period;
        }

        
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        ChangeTheme(Theme),
        ChangeDeletePeriod(String),
        ShowConsole(bool),
        OpenLink(String),
        Back
    }

    impl Default for Settings {
        fn default() -> Self {
            Settings::new()
        }
    }
}