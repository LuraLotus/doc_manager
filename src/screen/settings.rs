pub(crate) mod settings {
    use iced::{Alignment::Center, Element, Length, Task, Theme, widget::{Container, Grid, PickList, Text, column, pick_list, row}};

    pub(crate) struct Settings {
        current_theme: Option<Theme>
    }

    impl Settings {
        pub(crate) fn new() -> Settings {
            Settings {
                current_theme: Some(Theme::CatppuccinMacchiato)
            }
        }

        pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::ChangeTheme(theme) => {
                    self.current_theme = Some(theme);
                    Task::none()
                }
            }
        }

        pub(crate) fn view(&self) -> Element<Message> {
            Container::new(column![
                row![
                    Text::new("Theme: ").align_y(Center),
                    PickList::new(Settings::available_themes(), self.current_theme.clone(), Message::ChangeTheme)
                ].spacing(5).align_y(Center),
            ].spacing(5)).padding(5).into()
        }

        fn available_themes() -> Vec<Theme> {
            let mut available_themes: Vec<Theme> = Vec::new();

            for theme in Theme::ALL {
                available_themes.push(theme.clone());
            }

            return available_themes;

            // vec![
            //     Theme::Nightfly,
            //     Theme::TokyoNight,
            //     Theme::CatppuccinLatte,
            //     Theme::CatppuccinFrappe,
            //     Theme::CatppuccinMacchiato,
            //     Theme::CatppuccinMocha

            // ]
        }

        pub(crate) fn set_theme(&mut self, theme: Theme) {
            self.current_theme = Some(theme);
        }

        
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        ChangeTheme(Theme)
    }

    impl Default for Settings {
        fn default() -> Self {
            Settings::new()
        }
    }
}