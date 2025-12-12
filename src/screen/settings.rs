pub(crate) mod settings {
    use iced::{Alignment::Center, Element, Length, Task, Theme, widget::{Container, Grid, PickList, Text, button, column, container, pick_list, row, rule}};
    use iced_aw::Card;

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
                ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill),
            ].spacing(5)).padding(5).into()
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

        
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        ChangeTheme(Theme),
        Back
    }

    impl Default for Settings {
        fn default() -> Self {
            Settings::new()
        }
    }
}