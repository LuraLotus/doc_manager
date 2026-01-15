pub(crate) mod main_menu {
    use iced::{Alignment::Center, Border, Color, Element, Event, Font, Length, Task, Theme, advanced::Widget, widget::{Container, Image, Text, button, column, container, image::{Handle, Viewer}, row, rule, text}};

    use crate::{HOME_IMAGE, LocalTheme};

    #[derive(Default, Debug, Clone)]
    pub(crate) struct MainMenu;
    
    impl MainMenu {
        pub(crate) fn new() -> MainMenu {
            MainMenu
        }
        pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::DocumentList => Task::none(),
                Message::Settings => Task::none(),
                Message::NewDocument => todo!(),
                Message::KeyEvent(event) => Task::none(),
                Message::None => Task::none(),
            }
        }

        pub(crate) fn view(&self) -> Element<Message> {
            Container::new(
                column![
                    Text::new("Get started").size(28).font(Font {
                        family: iced::font::Family::Serif,
                        weight: iced::font::Weight::Normal,
                        ..Default::default()
                    }),
                    rule::horizontal(2),
                    row![
                        button(Text::new("Document List").size(24).align_x(Center).align_y(Center)).on_press(Message::DocumentList).width(200).height(100).style(|theme, status| button_style(theme, status)),
                        button(Text::new("Settings").size(24).align_x(Center).align_y(Center)).on_press(Message::Settings).width(200).height(100).style(|theme, status| button_style(theme, status))
                    ].spacing(5)
                ].spacing(5).padding(10).width(Length::Fill).height(Length::Fill)
                // Image::new(Handle::from_bytes(HOME_IMAGE))
                //     .expand(true)
                //     .content_fit(iced::ContentFit::Cover)
                //     .border_radius(5.0)
            ).width(Length::Fill).height(Length::Fill).style(container::bordered_box).into()
        }
        
    }
    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        DocumentList,
        Settings,
        NewDocument,
        KeyEvent(Event),
        None,
    }

    fn button_style(theme: &Theme, status: button::Status) -> button::Style {
        match status {
            button::Status::Active => {
                iced::widget::button::Style {
                    text_color: theme.extended_palette().primary.base.text.into(),
                    background: Some(theme.extended_palette().primary.base.color.into()),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 15.0.into()
                    },
                    shadow: Default::default(),
                    snap: false
                }
            },
            button::Status::Hovered => {
                iced::widget::button::Style {
                    text_color: theme.extended_palette().primary.strong.text.into(),
                    background: Some(theme.extended_palette().primary.strong.color.into()),
                    border: Border {
                        color: Color::TRANSPARENT,
                        width: 0.0,
                        radius: 15.0.into()
                    },
                    shadow: Default::default(),
                    snap: false
                }
            },
            button::Status::Pressed => iced::widget::button::Style {
                text_color: theme.extended_palette().primary.weak.text.into(),
                background: Some(theme.extended_palette().primary.weak.color.into()),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 15.0.into()
                },
                shadow: Default::default(),
                snap: false
            },
            button::Status::Disabled => iced::widget::button::Style {
                text_color: theme.extended_palette().primary.weak.text.into(),
                background: Some(theme.extended_palette().primary.weak.color.into()),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 15.0.into()
                },
                shadow: Default::default(),
                snap: false
            },
        }
    }

}
