pub(crate) mod recycle_bin {
    use std::sync::Arc;

    use iced::{Alignment::{self, Center}, Border, Color, Element, Length, Task, Theme, mouse::Interaction, widget::{Container, Id, MouseArea, Space, Text, button, column, container, image::{Handle, Viewer}, mouse_area, row, rule, scrollable, text_input}};
    use iced_aw::{Card, card};
    use log::error;
    use time::{OffsetDateTime, UtcDateTime, macros::format_description};

    use crate::{LocalTheme, attachment::attachment::Attachment, db::db_module::DbConnection, document::document::Document};

    pub(crate) struct RecycleBin {
        deleted_documents: Vec<Arc<Document>>,
        deleted_attachments: Vec<Arc<Attachment>>,
        current_open_document: Option<Arc<Document>>,
        current_document_tab: DocumentTab,
        current_open_attachment: Option<Arc<Attachment>>,
        current_file_bytes: Vec<Vec<u8>>,
        current_file_handles: Vec<Handle>,
        current_page_index: usize,
        search_text: String,
        current_tab: Tab,
        current_theme: LocalTheme
    }

    impl RecycleBin {
        pub(crate) fn new() -> RecycleBin {
            RecycleBin {
                deleted_documents: retreive_deleted_documents(),
                deleted_attachments: retreive_deleted_attachments(),
                current_open_document: None,
                current_document_tab: DocumentTab::Details,
                current_open_attachment: None,
                current_file_bytes: Vec::new(),
                current_file_handles: Vec::new(),
                current_page_index: 0,
                search_text: String::new(),
                current_tab: Tab::Document,
                current_theme: LocalTheme::CatppuccinMacchiato
            }
        }

        pub(crate) fn set_current_theme(&mut self, theme: LocalTheme) {
            self.current_theme = theme.clone();
        }

        pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::SwitchTab(tab) => {
                    self.current_tab = tab;
                    Task::none()
                },
                Message::SwitchDocumentTab(tab) => {
                    self.current_document_tab = tab;
                    Task::none()
                },
                Message::OpenDocument(document) => {
                    self.current_open_document = Some(document);
                    Task::none()
                },
                Message::RestoreDocument(document) => {
                    match DbConnection::new().restore_document(document.get_document_id()) {
                        Err(err) => {
                            error!("Error restoring document: {}", err);
                        },
                        Ok(_) => {
                            self.refresh_data();
                            self.current_open_document = None;
                        }
                    }
                    Task::none()
                },
                Message::CloseDocument => {
                    self.reset_document_state();
                    self.current_open_document = None;
                    Task::none()
                },
                Message::OpenAttachment(attachment) => {
                    self.current_open_attachment = Some(attachment);
                    for page in self.current_open_attachment.as_ref().unwrap().pages() {
                        self.current_file_bytes.push(page.image().to_vec());
                    }
                    self.update_file_handles();
                    Task::none()
                },
                Message::RestoreAttachment(attachment) => {
                    match DbConnection::new().restore_attachment(attachment.get_attachment_id()) {
                        Err(err) => {
                            error!("Error restoring attachment: {}", err);
                        },
                        Ok(_) => {
                            self.refresh_data();
                            self.current_open_attachment = None;
                        }
                    }
                    Task::none()
                },
                Message::CloseAttachment => {
                    self.reset_attachment_state();
                    self.current_open_attachment = None;
                    Task::none()
                },
                Message::PrevPage => {
                    if self.current_page_index > 0 {
                        self.current_page_index -= 1;
                    }
                    Task::none()
                },
                Message::NextPage => {
                    if self.current_page_index < self.current_file_handles.len() - 1 {
                        self.current_page_index += 1;
                    }
                    Task::none()
                },
                Message::SearchTextChange(search_text) => {
                    self.search_text = search_text;
                    Task::none()
                },
                Message::RefreshData => {
                    self.refresh_data();
                    Task::none()
                },
                Message::Back => {
                    Task::none()
                }
            }
        }

        pub(crate) fn view(&self) -> Element<'_, Message> {
            let mut document_cards: Vec<DataCard> = Vec::new();

            for document in &self.deleted_documents {
                document_cards.push(DataCard::new(Some(document.clone()), None, self.current_theme.clone()));
            };

            let mut attachment_cards: Vec<DataCard> = Vec::new();

            for attachment in &self.deleted_attachments {
                attachment_cards.push(DataCard::new(None, Some(attachment.clone()), self.current_theme.clone()));
            }

            let content = match self.current_tab {
                Tab::Document => {
                    match &self.current_open_document {
                        Some(document) => {
                            Container::new(column![
                                match self.current_document_tab {
                                    // Document Details Screen
                                    DocumentTab::Details => {
                                        Container::new(column![
                                            Container::new(row![
                                                button("<").on_press(Message::CloseDocument),
                                                Space::new().width(Length::Fill),
                                                button("Restore").on_press(Message::RestoreDocument(document.clone()))
                                            ].spacing(5).align_y(Center)).width(Length::Fill).padding(5).style(container::bordered_box),
                                            Container::new(column![
                                                row![
                                                    Text::new(format!("Deleted Document - {}", document.get_document_number())).size(20)
                                                ].spacing(5).align_y(Center),
                                                rule::horizontal(2),
                                                row![
                                                    row![
                                                        Text::new("Document Number "),
                                                        Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                                    ].width(Length::FillPortion(1)),
                                                    text_input("", &document.get_document_number()).width(Length::FillPortion(4))
                                                ].spacing(5).align_y(Center),
                                                row![
                                                    Text::new("Document Type").width(Length::FillPortion(1)), 
                                                    text_input("", &document.get_document_type()).width(Length::FillPortion(4))
                                                ].spacing(5).align_y(Center),
                                                row![
                                                    Text::new("Comment").width(Length::FillPortion(1)), 
                                                    text_input("", &document.get_comment()).width(Length::FillPortion(4))
                                                ].spacing(5).align_y(Center),
                                                
                                            ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill),
                                        ].spacing(5)
                                        ).height(Length::Fill).width(Length::Fill)
                                    },
                                    DocumentTab::Attachments => {
                                        let mut attachment_cards: Vec<DataCard> = Vec::new();

                                        for attachment in document.get_attachments().unwrap() {
                                            attachment_cards.push(DataCard::new(None, Some(attachment.clone()), self.current_theme.clone()));
                                        }
                                        match &self.current_open_attachment {
                                            None => {
                                                Container::new(column![
                                                    Container::new(row![
                                                        button("<").on_press(Message::CloseDocument),
                                                    ].spacing(5).align_y(Center)).width(Length::Fill).padding(5).style(container::bordered_box),
                                                    Container::new(column![
                                                        row![
                                                            Text::new(format!("{} - Attachments", self.current_open_document.as_ref().unwrap().get_document_number())).size(20).align_y(Center),
                                                            Space::new().width(Length::Fill),
                                                            text_input("Search", &self.search_text).on_input(Message::SearchTextChange).id(Id::new("search"))
                                                        ],
                                                        rule::horizontal(2),
                                                        scrollable(row(attachment_cards.into_iter().filter(|card| {
                                                            card.get_attachment().get_reference_number().to_string().to_lowercase().contains(&self.search_text) ||
                                                            card.get_attachment().get_comment().to_string().to_lowercase().contains(&self.search_text)
                                                        }).map(|card| {
                                                            card.new_attachment_card().into()
                                                        })).spacing(10).wrap()),
                                                    ].spacing(5)).style(container::bordered_box).padding(10).width(Length::Fill).height(Length::Fill),
                                                ].spacing(5)).width(Length::Fill).height(Length::Fill)
                                            },
                                            // Attachment Details Screen
                                            Some(attachment) => {
                                                Container::new(column![
                                                    Container::new(row![
                                                        button("<").on_press(Message::CloseAttachment)
                                                    ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                                    Container::new(column![
                                                        row![
                                                            Text::new(format!("Deleted Attachment - {}", attachment.get_reference_number())).size(20).align_y(Center)
                                                        ].spacing(5).align_y(Center),
                                                        rule::horizontal(2),
                                                        row![
                                                            Container::new(column![
                                                                row![
                                                                    Text::new("Attachment Number "),
                                                                    Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                                                ].width(Length::FillPortion(1)),
                                                                text_input("", &attachment.get_reference_number()),
                                                                Text::new("Comment"), 
                                                                text_input("", &attachment.get_comment()),
                                                                row![
                                                                    Text::new("Image File "),
                                                                    Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                                                ].width(Length::FillPortion(1)),
                                                                column![
                                                                    row![
                                                                        text_input("", &self.current_file_handles.len().to_string().as_str()),
                                                                    ].spacing(5).width(Length::Fill)
                                                                ].spacing(5),
                                                            ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)).height(Length::Fill),
                                                            rule::vertical(2),
                                                            Container::new(
                                                                column![
                                                                    if self.current_file_handles.is_empty() || self.current_file_handles.len() == 0 {
                                                                        Container::new(Space::new().width(Length::Fill).height(Length::Fill))
                                                                    }
                                                                    else {
                                                                        Container::new(Viewer::new(self.current_file_handles[self.current_page_index].clone()).width(Length::Fill).height(Length::Fill))
                                                                    },
                                                                    rule::horizontal(2),
                                                                    row![
                                                                        Container::new(
                                                                            row![
                                                                                if self.current_page_index > 0 {
                                                                                    button("<").on_press(Message::PrevPage)
                                                                                }
                                                                                else {
                                                                                    button("<")
                                                                                },
                                                                                Text::new(self.current_page_index + 1).center(),
                                                                                if self.current_page_index + 1 < self.current_file_handles.len() {
                                                                                    button(">").on_press(Message::NextPage)
                                                                                }
                                                                                else {
                                                                                    button(">")
                                                                                }
                                                                            ].spacing(10).align_y(Center)
                                                                        ).width(Length::Fill).align_x(Center),
                                                                    ].width(Length::Fill).align_y(Center)
                                                                ].spacing(5).align_x(Center)
                                                            ).padding(5).style(container::bordered_box).width(Length::FillPortion(3)).height(Length::Fill)
                                                        ].spacing(5),
                                                    ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                                ].spacing(5)
                                                ).height(Length::Fill).width(Length::Fill)
                                            }
                                        }
                                    },
                                },
                                tab_bar(self.current_document_tab)
                            ].spacing(5)).into()
                        },
                        None => {
                            Container::new(
                                column![
                                    Container::new(row![
                                            button("<").on_press(Message::Back)
                                    ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                    Container::new(
                                        column![
                                            row![
                                                Text::new("Deleted Documents").align_y(Center).size(20),
                                                Space::new().width(Length::Fill),
                                                text_input("Search", &self.search_text).on_input(Message::SearchTextChange).id(Id::new("search")),
                                            ].spacing(5),
                                            rule::horizontal(2),
                                            scrollable(row(
                                                document_cards.into_iter().filter(|card| {
                                                    card.get_document().get_document_number().to_string().to_lowercase().contains(&self.search_text.to_lowercase()) ||
                                                    card.get_document().get_document_type().to_string().to_lowercase().contains(&self.search_text.to_lowercase()) ||
                                                    card.get_document().get_comment().to_string().to_lowercase().contains(&self.search_text.to_lowercase())
                                                }).map(|card| {
                                                    card.new_document_card().into()
                                                }) 
                                            ).spacing(10).wrap()),
                                        ].spacing(5)
                                    ).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                ].spacing(5)
                            )
                        }
                    }
                },
                Tab::Attachment => {
                    match &self.current_open_attachment {
                        Some(attachment) => {
                            Container::new(column![
                                Container::new(row![
                                    button("<").on_press(Message::CloseAttachment),
                                    Space::new().width(Length::Fill),
                                    button("Restore").on_press(Message::RestoreAttachment(attachment.clone()))
                                ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                Container::new(column![
                                    row![
                                        Text::new(format!("Deleted Attachment - {}", attachment.get_reference_number())).size(20).align_y(Center)
                                    ].spacing(5).align_y(Center),
                                    rule::horizontal(2),
                                    row![
                                        Container::new(column![
                                            row![
                                                Text::new("Attachment Number "),
                                                Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                            ].width(Length::FillPortion(1)),
                                            text_input("", &attachment.get_reference_number()),
                                            Text::new("Comment"), 
                                            text_input("", &attachment.get_comment()),
                                            row![
                                                Text::new("Image File "),
                                                Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                            ].width(Length::FillPortion(1)),
                                            column![
                                                row![
                                                    text_input("", &self.current_file_handles.len().to_string().as_str()),
                                                ].spacing(5).width(Length::Fill)
                                            ].spacing(5),
                                        ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)).height(Length::Fill),
                                        rule::vertical(2),
                                        Container::new(
                                            column![
                                                if self.current_file_handles.is_empty() || self.current_file_handles.len() == 0 {
                                                    Container::new(Space::new().width(Length::Fill).height(Length::Fill))
                                                }
                                                else {
                                                    Container::new(Viewer::new(self.current_file_handles[self.current_page_index].clone()).width(Length::Fill).height(Length::Fill))
                                                },
                                                rule::horizontal(2),
                                                row![
                                                    Container::new(
                                                        row![
                                                            if self.current_page_index > 0 {
                                                                button("<").on_press(Message::PrevPage)
                                                            }
                                                            else {
                                                                button("<")
                                                            },
                                                            Text::new(self.current_page_index + 1).center(),
                                                            if self.current_page_index + 1 < self.current_file_handles.len() {
                                                                button(">").on_press(Message::NextPage)
                                                            }
                                                            else {
                                                                button(">")
                                                            }
                                                        ].spacing(10).align_y(Center)
                                                    ).width(Length::Fill).align_x(Center),
                                                ].width(Length::Fill).align_y(Center)
                                            ].spacing(5).align_x(Center)
                                        ).padding(5).style(container::bordered_box).width(Length::FillPortion(3)).height(Length::Fill)
                                    ].spacing(5),
                                ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                            ].spacing(5)).height(Length::Fill).width(Length::Fill)
                        },
                        None => {
                            Container::new(
                                column![
                                    Container::new(row![
                                            button("<").on_press(Message::Back)
                                    ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                    Container::new(
                                        column![
                                            row![
                                                Text::new("Deleted Attachments").align_y(Center).size(20),
                                                Space::new().width(Length::Fill),
                                                text_input("Search", &self.search_text).on_input(Message::SearchTextChange).id(Id::new("search")),
                                            ].spacing(5),
                                            rule::horizontal(2),
                                            scrollable(row(
                                                attachment_cards.into_iter().filter(|card| {
                                                    card.get_attachment().get_reference_number().to_string().to_lowercase().contains(&self.search_text.to_lowercase()) ||
                                                    card.get_attachment().get_reference_number().to_string().to_lowercase().contains(&self.search_text.to_lowercase()) ||
                                                    card.get_attachment().get_comment().to_string().to_lowercase().contains(&self.search_text.to_lowercase())
                                                }).map(|card| {
                                                    card.new_attachment_card().into()
                                                }) 
                                            ).spacing(10).wrap()),
                                        ].spacing(5)
                                    ).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                ].spacing(5)
                            )
                        },
                    }
                    
                }
            };

            Container::new(
                content
            ).into()
        }

        fn update_file_handles(&mut self) {
            self.current_file_handles.clear();
            if self.current_file_bytes.is_empty() {
                self.current_file_handles.clear();
            }
            else {
                for bytes in &self.current_file_bytes {
                    self.current_file_handles.push(Handle::from_bytes(bytes.to_vec()));
                }
            }
        }

        pub(crate) fn refresh_data(&mut self) {
            self.deleted_documents = retreive_deleted_documents();
            self.deleted_attachments = retreive_deleted_attachments();
        }

        pub(crate) fn reset_state(&mut self) {
            self.current_open_document = None;
            self.current_open_attachment = None;
            self.current_file_bytes.clear();
            self.current_file_handles.clear();
            self.current_page_index = 0;
            self.current_tab = Tab::Document;
            self.current_document_tab = DocumentTab::Details;
        }

        pub(crate) fn reset_document_state(&mut self) {
            self.current_open_document = None;
            self.current_open_attachment = None;
            self.current_file_bytes.clear();
            self.current_file_handles.clear();
            self.current_page_index = 0;
        }

        pub(crate) fn reset_attachment_state(&mut self) {
            self.current_open_attachment = None;
            self.current_file_bytes.clear();
            self.current_file_handles.clear();
            self.current_page_index = 0;
        }
    }

    fn retreive_deleted_documents() -> Vec<Arc<Document>> {
        match DbConnection::new().read_deleted_documents() {
            Ok(documents) => {
                documents
            },
            Err(err) => {
                error!("Error retreiving deleted documents: {}", err);
                panic!("Error retreiving deleted documents: {}", err);
            }
        }
    }

    fn retreive_deleted_attachments() -> Vec<Arc<Attachment>> {
        match DbConnection::new().read_deleted_attachments() {
            Ok(attachments) => {
                attachments
            },
            Err(err) => {
                error!("Error retreiving deleted attachments: {}", err);
                panic!("Error retreiving deleted attachments: {}", err);
            }
        }
    }

    struct DataCard {
        document: Option<Arc<Document>>,
        attachment: Option<Arc<Attachment>>,
        theme: LocalTheme
    }

    impl DataCard {
        fn new(document: Option<Arc<Document>>, attachment: Option<Arc<Attachment>>, theme: LocalTheme) -> DataCard {
            DataCard {
                document: document,
                attachment: attachment,
                theme: theme
            }
        }

        fn new_document_card(&self) -> MouseArea<'static, Message> {
            let datetime_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
            let datetime = match match UtcDateTime::from_unix_timestamp(self.document.as_ref().unwrap().get_date_added()) {
                Ok(datetime) => datetime,
                Err(err) => {
                    error!("Error converting Unix Timestamp to UtcDateTime: {}", err);
                    panic!("Error converting Unix Timestamp to UtcDateTime: {}", err);
                }
            }.to_offset(match OffsetDateTime::now_local() {
                Ok(offset_date) => offset_date.offset(),
                Err(err) => {
                    error!("Error applying offset: {}", err);
                    panic!("Error applying offset: {}", err);
                }
            }).format(datetime_format) {
                Ok(date_string) => date_string,
                Err(err) => {
                    error!("Error formatting datetime: {}", err);
                    panic!("Error formatting datetime: {}", err);
                }
            };

            mouse_area(
                Card::new(Text::new(self.document.as_ref().unwrap().get_document_number().to_string()), column![
                    Text::new(self.document.as_ref().unwrap().get_document_type().to_string()),
                    Text::new(self.document.as_ref().unwrap().get_comment().to_string())
                ]).max_height(500.0).max_width(200.0).foot(Text::new(datetime)).style(|theme: &Theme, _| card_style(theme))
            ).on_press(Message::OpenDocument(self.document.as_ref().unwrap().clone())).interaction(Interaction::Pointer)
        }

        fn new_attachment_card(&self) -> MouseArea<'static, Message> {
            let datetime_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
            let datetime = match match UtcDateTime::from_unix_timestamp(self.attachment.as_ref().unwrap().get_date_added()) {
                Ok(datetime) => datetime,
                Err(err) => {
                    error!("Error converting Unix timestamp to UtcDateTime: {}", err);
                    panic!("Error converting Unix timestamp to UtcDateTime: {}", err);
                }
            }.to_offset(match OffsetDateTime::now_local() {
                Ok(offset_date) => offset_date.offset(),
                Err(err) => {
                    error!("Error applying offset: {}", err);
                    panic!("Error applying offset: {}", err);
                }
            }).format(datetime_format) {
                Ok(date_string) => date_string,
                Err(err) => {
                    error!("Error formatting datetime: {}", err);
                    panic!("Error formatting datetime: {}", err);
                }
            };

            mouse_area(
                Card::new(Text::new(self.attachment.as_ref().unwrap().get_reference_number().to_string()), column![
                    Text::new(self.attachment.as_ref().unwrap().get_comment().to_string())
                ]).max_height(500.0).max_width(200.0).foot(Text::new(datetime)).style(|theme, _| card_style(theme))
            ).on_press(Message::OpenAttachment(self.attachment.as_ref().unwrap().clone())).interaction(Interaction::Pointer)
        }

        pub(crate) fn get_document(&self) -> Arc<Document> {
            return self.document.as_ref().unwrap().clone()
        }

        pub(crate) fn get_attachment(&self) -> Arc<Attachment> {
            return self.attachment.as_ref().unwrap().clone()
        }
    }

    fn card_style(theme: &Theme) -> card::Style {
        card::Style {
            background: theme.extended_palette().background.base.color.into(),
            border_radius: 10.0,
            border_width: 1.0,
            border_color: theme.extended_palette().primary.base.color.into(),
            head_background: theme.extended_palette().primary.base.color.into(),
            head_text_color: theme.extended_palette().primary.base.text.into(),
            body_background: Color::TRANSPARENT.into(),
            body_text_color: theme.extended_palette().background.base.text.into(),
            foot_background: Color::TRANSPARENT.into(),
            foot_text_color: theme.extended_palette().background.base.text.into(),
            close_color: Default::default(),
        }
    }

    fn tab_bar(selected_tab: DocumentTab) -> Element<'static, Message> {
        Container::new(
            row![
                button(Text::new("Details").center().size(20).height(Length::Fill)).on_press(Message::SwitchDocumentTab(DocumentTab::Details)).style(move |theme, status| 
                    if selected_tab == DocumentTab::Details {
                        tab_bar_button_selected_style(theme)
                    }
                    else {
                        tab_bar_button_style(theme, status)
                    }
                ).width(Length::FillPortion(1)).height(Length::Fill),
                button(Text::new("Attachments").center().size(20).height(Length::Fill)).on_press(Message::SwitchDocumentTab(DocumentTab::Attachments)).style(move |theme, status|
                    if selected_tab == DocumentTab::Attachments {
                        tab_bar_button_selected_style(theme)
                    }
                    else {
                        tab_bar_button_style(theme, status)
                    }
                ).width(Length::FillPortion(1)).height(Length::Fill)
            ].spacing(5).align_y(Center).width(Length::Fill).height(Length::Fixed(40.0))
        ).padding(5).width(Length::Fill).style(container::bordered_box).into()
    }

    fn tab_bar_button_style(theme: &Theme, status: iced::widget::button::Status) -> iced::widget::button::Style {
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
                    text_color: theme.extended_palette().background.weaker.text.into(),
                    background: Some(theme.extended_palette().background.weaker.color.into()),
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
                text_color: theme.extended_palette().background.weak.text.into(),
                background: Some(theme.extended_palette().background.weak.color.into()),
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

    fn tab_bar_button_selected_style(theme: &Theme) -> iced::widget::button::Style {
        iced::widget::button::Style {
            text_color: theme.extended_palette().background.strong.text.into(),
            background: Some(theme.extended_palette().background.strong.color.into()),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 5.0.into()
            },
            shadow: Default::default(),
            snap: false
        }
    }

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        SwitchTab(Tab),
        OpenDocument(Arc<Document>),
        SwitchDocumentTab(DocumentTab),
        RestoreDocument(Arc<Document>),
        CloseDocument,
        OpenAttachment(Arc<Attachment>),
        RestoreAttachment(Arc<Attachment>),
        CloseAttachment,
        PrevPage,
        NextPage,
        SearchTextChange(String),
        RefreshData,
        Back
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub(crate) enum Tab {
        Document,
        Attachment
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum DocumentTab {
        Details,
        Attachments
    }
}