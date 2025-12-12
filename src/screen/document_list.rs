pub(crate) mod document_list {
    use std::{env::{current_dir, current_exe}, fs, io::Cursor, path::PathBuf, process::Stdio, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

    use caesium::{compress_in_memory, convert_in_memory, parameters::{CSParameters, PngParameters}};
    use file_format::FileFormat;
    use iced::{Alignment::Center, Background, Border, Color, Element, Event, Gradient, Length, Shadow, Subscription, Task, Theme, advanced::graphics::futures::subscription, gradient::{ColorStop, Linear}, keyboard::{self, Key, key}, mouse::Interaction, theme::Palette, widget::{Container, Id, MouseArea, ProgressBar, Space, Text, button, column, container::{self, Style}, image::{Handle, Viewer}, mouse_area, operation::focus_next, progress_bar, row, rule, scrollable}, window::events};
    use iced::widget::text_input;
    use iced_aw::{Card, TabBarPosition, TabLabel, Tabs, card::Status, style::card};
    use rfd::FileDialog;
    use time::{Duration, OffsetDateTime, UtcDateTime, macros::format_description};

    use crate::{LocalTheme, State, db::db_module::DbConnection, screen::{Attachment, Document}};

    #[derive(Debug, Clone, Default)]
    pub(crate) struct DocumentList {
        documents: Vec<Arc<Document>>,
        search_text: String,
        current_open_document: Option<Arc<Document>>,
        current_document_tab: Tab,
        current_document_number: String,
        current_document_type: String,
        current_comment: String,
        current_open_attachment: Option<Arc<Attachment>>,
        current_attachment_reference_number: String,
        current_attachment_comment: String,
        data_changed: bool,
        create_new_document: bool,
        create_new_attachment: bool,
        current_file_path: Option<String>,
        current_file: Option<Handle>,
        current_file_bytes: Option<Vec<u8>>,
        file_scanned: bool,
        file_path_changed: bool,
        input1_id: Option<Id>,
        input2_id: Option<Id>,
        input3_id: Option<Id>,
        scanning: bool,
        scan_progress: f32,
        current_theme: Option<LocalTheme>
    }

    impl DocumentList {
        pub(crate) fn new() -> DocumentList {
            DocumentList {
                documents: Result::expect(DbConnection::new().read_document_table(), "Error retrieving data from database."),
                search_text: String::from(""),
                current_open_document: None,
                current_document_tab: Tab::default(),
                current_document_number: String::default(),
                current_document_type: String::default(),
                current_comment: String::default(),
                current_open_attachment: None,
                current_attachment_reference_number: String::default(),
                current_attachment_comment: String::default(),
                data_changed: false,
                create_new_document: false,
                create_new_attachment: false,
                current_file_path: None,
                current_file: None,
                current_file_bytes: None,
                file_scanned: false,
                file_path_changed: false,
                input1_id: Some(Id::new("1")),
                input2_id: Some(Id::new("2")),
                input3_id: Some(Id::new("3")),
                scanning: false,
                scan_progress: f32::default(),
                current_theme: None
            }
        }

        pub(crate) fn set_current_theme(&mut self, theme: LocalTheme) {
            self.current_theme = Some(theme);
        }

        pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::NewDocument => {
                    self.create_new_document = true;
                    self.current_document_number.clear();
                    self.current_document_type.clear();
                    self.current_comment.clear();
                    Task::none()
                },
                Message::SaveNewDocument => {
                    let mut conn = DbConnection::new();
                    conn.new_document(
                        self.current_document_number.clone(),
                        self.current_document_type.clone(), 
                        self.current_comment.clone()
                    ).unwrap_or_else(|err| {
                        println!("Error adding new document: {}", err);
                        0
                    });

                    self.create_new_document = false;
                    self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                    self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == conn.get_last_rowid().unwrap() as u32).cloned();
                    let file_path = format!("./data/{}", self.current_open_document.as_ref().unwrap().get_document_id());
                    fs::create_dir(file_path).unwrap_or_else(|err| {
                        println!("Error creating document's attachment folder: {}", err);
                    });
                    self.data_changed = false;
                    Task::none()
                },
                Message::OpenDocument(document) => {
                    self.current_open_document = Some(document.clone());
                    self.current_document_number = document.clone().get_document_number().to_string();
                    self.current_document_type = document.clone().get_document_type().to_string();
                    self.current_comment = document.clone().get_comment().to_string();
                    Task::none()
                }
                Message::SaveCurrentDocument => {
                    let mut conn=  DbConnection::new();
                    let current_document_id = self.current_open_document.as_ref().unwrap().get_document_id();
                    conn.edit_document_details(
                        current_document_id,
                        self.current_document_number.clone(),
                        self.current_document_type.clone(),
                        self.current_comment.clone()
                    ).unwrap_or_else(|err| {
                        println!("Error editing document: {}", err);
                        0
                    });

                    self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                    self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                    self.data_changed = false;
                    Task::none()
                }
                Message::SwitchTab(tab) => {
                    self.current_document_tab = tab;
                    Task::none()
                },
                Message::CloseDocument => {
                    self.current_open_document = None;
                    self.current_document_number.clear();
                    self.current_document_type.clear();
                    self.current_comment.clear();
                    self.data_changed = false;
                    self.create_new_document = false;
                    self.create_new_attachment = false;
                    self.current_document_tab = Tab::default();
                    String::clear(&mut self.search_text);
                    Task::none()
                }
                Message::SearchTextChange(input) => {
                    self.search_text = input;
                    Task::none()
                },
                Message::Back => { Task::none() },
                Message::None => { Task::none() },
                Message::CurrentDocumentNumberChange(input) => {
                    self.current_document_number = input;
                    self.data_changed = true;
                    Task::none()
                },
                Message::CurrentDocumentTypeChange(input) => {
                    self.current_document_type = input;
                    self.data_changed = true;
                    Task::none()
                },
                Message::CurrentCommentChange(input) => {
                    self.current_comment = input;
                    self.data_changed = true;
                    Task::none()
                },
                Message::NewAttachment => {
                    self.create_new_attachment = true;
                    Task::none()
                },
                Message::OpenFileDialog => {
                    let previous_path = self.current_file_path.clone();
                    self.current_file_path = Some(
                        FileDialog::new().set_title("Select Document")
                        .add_filter("Image (.png, .jpg, .jpeg, .webp)", &["png", "jpg", "jpeg", "webp"])
                        .pick_file().unwrap_or_else(|| {
                            println!("No file was selected");
                            if self.current_file_path.is_none() {
                                PathBuf::new()
                            }
                            else {
                                PathBuf::from(self.current_file_path.as_ref().unwrap())
                            }
                    }).to_str().unwrap().to_string());
                    if previous_path != self.current_file_path {
                        println!("true");
                        self.file_path_changed = true;
                        self.current_file = Some(Handle::from_bytes(fs::read(self.current_file_path.clone().unwrap_or_else(|| {
                            println!("No file was selected");
                            String::new()
                        })).unwrap_or_else(|err| {
                            println!("No file was selected: {}", err);
                            Vec::new()
                        })))
                    }
                    println!("{:?}", &self.current_file_path);
                    Task::none()
                },
                Message::SaveNewAttachment => {
                    let mut conn = DbConnection::new();
                    let current_document_id = self.current_open_document.clone().unwrap().get_document_id();
                    
                    conn.new_attachment(String::new(), self.current_attachment_reference_number.clone(), self.current_attachment_comment.clone(), current_document_id).unwrap_or_else(|err| {
                        println!("Error creating new attachment: {}", err);
                        0
                    });

                    let file_name = format!("{}_{}.png", current_document_id, conn.get_last_rowid().unwrap());

                    let file_path = format!("./data/{}/{}", current_document_id, file_name);

                    if self.file_scanned {
                        let mut bytes = self.current_file_bytes.clone().unwrap_or_else(|| {
                            println!("Current file bytes is none");
                            Vec::new()
                        });

                        if FileFormat::from_bytes(&bytes).extension() != "png" {
                            let img = image::load_from_memory(&bytes);
                            match img.unwrap().write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png) {
                                Err(err) => println!("Error converting image format: {}", err),
                                _ => {}
                            }
                        }
                        let parameters = CSParameters::new();
                        let compressed_bytes = compress_in_memory(bytes, &parameters).unwrap_or_else(|err| {
                            println!("Error compressing image: {}", err);
                            Vec::new()
                        });

                        fs::write(file_path.clone(), compressed_bytes).unwrap_or_else(|err| {
                            println!("Error writing bytes to file: {}", err);
                        });
                    }
                    else {
                        let mut bytes = fs::read(self.current_file_path.clone().unwrap_or_else(|| {
                            println!("Error reading file from path");
                            String::new()
                        })).unwrap_or_else(|err| {
                            println!("Error reading file from path: {}", err);
                            Vec::new()
                        });

                        if FileFormat::from_bytes(&bytes).extension() != "png" {
                            let img = image::load_from_memory(&bytes);
                            match img.unwrap().write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png) {
                                Err(err) => println!("Error converting image format: {}", err),
                                _ => {}
                            }
                        }
                        let mut parameters = CSParameters::new();
                        parameters.png.quality = 100;
                        parameters.png.optimize = true;
                        parameters.png.optimization_level = 6;
                        let compressed_bytes = compress_in_memory(bytes, &parameters).unwrap_or_else(|err| {
                            println!("Error compressing image: {}", err);
                            Vec::new()
                        });

                        fs::write(&file_path, compressed_bytes).unwrap_or_else(|err| {
                            println!("Error writing file to data folder: {}", err);
                        });
                    }

                    conn.edit_attachment_file_path(conn.get_last_rowid().unwrap() as u32, file_path).unwrap_or_else(|err| {
                        println!("Error editing attachment file path: {}", err);
                        0
                    });
 
                    self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                    self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                    self.current_open_attachment = self.current_open_document.as_ref().unwrap().get_attachments().unwrap().iter().find(|attachment| attachment.get_attachment_id() == conn.get_last_rowid().unwrap() as u32).cloned();
                    self.current_attachment_reference_number = self.current_open_attachment.as_ref().unwrap().get_reference_number().to_string();
                    self.current_attachment_comment = self.current_open_attachment.as_ref().unwrap().get_comment().to_string();
                    self.current_file_path = Some(self.current_open_attachment.as_ref().unwrap().get_file_path().to_string());
                    self.file_scanned = false;
                    self.current_file_bytes = None;
                    self.create_new_attachment = false;
                    self.data_changed = false;
                    self.file_path_changed = false;
                    Task::none()
                    
                },
                Message::OpenAttachment(attachment) => {
                    self.current_open_attachment = Some(attachment.clone());
                    self.current_attachment_reference_number = attachment.clone().get_reference_number().to_string();
                    self.current_attachment_comment = attachment.clone().get_comment().to_string();
                    self.current_file_path = Some(attachment.as_ref().get_file_path().to_string());
                    if let Some(file_path) = &self.current_file_path {
                        match fs::read(file_path) {
                            Ok(bytes) => {
                                self.current_file = Some(Handle::from_bytes(bytes));
                            }
                            Err(err) => {
                                println!("Error reading file: {}", err);
                                self.current_file = Some(Handle::from_path("src/ferris-error-handling.webp"));
                            }
                        }
                    }
                    Task::none()
                },
                Message::SaveCurrentAttachment => {
                    let mut conn = DbConnection::new();
                    let current_document_id = self.current_open_document.as_ref().unwrap().get_document_id();
                    let current_attachment_id = self.current_open_attachment.as_ref().unwrap().get_attachment_id();
                    
                    conn.edit_attachment_details(
                        current_attachment_id,
                        self.current_attachment_reference_number.clone(),
                        self.current_attachment_comment.clone()
                    ).unwrap_or_else(|err| {
                        println!("Error editing attachment: {}", err);
                        0
                    });

                    if self.file_path_changed {
                        let file_format = FileFormat::from_file(self.current_file_path.as_ref().unwrap()).unwrap_or_else(|err| {
                            println!("Error checking file format: {}", err);
                            FileFormat::Empty
                        });


                        let file_name = format!("{}_{}.png", current_document_id, current_attachment_id);

                        let file_path = format!("./data/{}/{}", current_document_id, file_name);

                        fs::remove_file(self.current_open_attachment.as_ref().unwrap().clone().get_file_path().to_string()).unwrap_or_else(|err| {
                            println!("Error deleting old file: {}", err);
                        });

                        if self.file_scanned {
                            let mut bytes = self.current_file_bytes.clone().unwrap_or_else(|| {
                                println!("Current file bytes is none");
                                Vec::new()
                            });

                            if FileFormat::from_bytes(&bytes).extension() != "png" {
                                let img = image::load_from_memory(&bytes);
                                match img.unwrap().write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png) {
                                    Err(err) => println!("Error converting image format: {}", err),
                                    _ => {}
                                }
                            }
                            let parameters = CSParameters::new();
                            let compressed_bytes = compress_in_memory(bytes, &parameters).unwrap_or_else(|err| {
                                println!("Error compressing image: {}", err);
                                Vec::new()
                            });

                            fs::write(file_path.clone(), compressed_bytes).unwrap_or_else(|err| {
                                println!("Error writing bytes to file: {}", err);
                            });
                        }
                        else {
                            let mut bytes = fs::read(self.current_file_path.clone().unwrap_or_else(|| {
                                println!("Error reading file from path");
                                String::new()
                            })).unwrap_or_else(|err| {
                                println!("Error reading file from path: {}", err);
                                Vec::new()
                            });

                            if FileFormat::from_bytes(&bytes).extension() != "png" {
                                let img = image::load_from_memory(&bytes);
                                match img.unwrap().write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png) {
                                    Err(err) => println!("Error converting image format: {}", err),
                                    _ => {}
                                }
                            }
                            let parameters = CSParameters::new();
                            let compressed_bytes = compress_in_memory(bytes, &parameters).unwrap_or_else(|err| {
                                println!("Error compressing image: {}", err);
                                Vec::new()
                            });

                            fs::write(&file_path, compressed_bytes).unwrap_or_else(|err| {
                                println!("Error writing file to data folder: {}", err);
                            });
                        }

                        conn.edit_attachment_file_path(current_attachment_id, file_path).unwrap_or_else(|err| {
                            println!("Error editing attachment file path: {}", err);
                            0
                        });

                        self.file_path_changed = false;
                    }

                    self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                    self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                    self.current_open_attachment = self.current_open_document.as_ref().unwrap().get_attachments().unwrap().iter().find(|attachment| attachment.get_attachment_id() == current_attachment_id).cloned();
                    self.current_file_path = Some(self.current_open_attachment.as_ref().unwrap().get_file_path().to_string());
                    self.file_scanned = false;
                    self.current_file_bytes = None;
                    self.data_changed = false;
                    Task::none()
                },
                Message::CurrentAttachmentReferenceNumberChange(input) => {
                    self.current_attachment_reference_number = input;
                    self.data_changed = true;
                    Task::none()
                },
                Message::CurrentAttachmentCommentChange(input) => {
                    self.current_attachment_comment = input;
                    self.data_changed = true;
                    Task::none()
                },
                Message::CloseAttachment => {
                    self.current_open_attachment = None;
                    self.current_attachment_reference_number.clear(); 
                    self.current_attachment_comment.clear();
                    self.data_changed = false;
                    self.file_path_changed = false;
                    self.current_file_path = None;
                    self.current_file = None;
                    self.current_file_bytes = None;
                    self.file_scanned = false;
                    Task::none()
                },
                Message::KeyEvent(key) => {
                    match key {
                        keyboard::Key::Named(key::Named::Tab) => {
                            focus_next()
                        }

                        _ => Task::none()
                    }
                },
                Message::Scan => {
                    self.scanning = true;
                    self.scan_progress = 0.0;
                    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    let temp_path = std::env::temp_dir().join("temp").join(format!("scan_{}.png", time));
                    if let Some(parent) = temp_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }

                    return Task::perform(
                        async move {
                            let script = format!(r#"
                                $out = '{}';
                                $d = New-Object -ComObject WIA.CommonDialog;
                                $device = $d.ShowSelectDevice();
                                if ($device -ne $null) {{
                                    try {{
                                        $img = $d.ShowAcquireImage($device.DeviceID);
                                    }} catch {{
                                        $img = $d.ShowAcquireImage();
                                    }}
                                }}
                                if ($img -ne $null) {{ 
                                    $img.SaveFile($out); 
                                    Write-Output $out;
                                    exit 0
                                }}
                                else {{ 
                                    exit 1
                                }}
                                "#,
                                temp_path.to_string_lossy()
                            );

                            let output = std::process::Command::new("powershell.exe")
                                .arg("-NoProfile")
                                .arg("-ExecutionPolicy")
                                .arg("Bypass")
                                .arg("-Command")
                                .arg(script)
                                .stdin(Stdio::null())
                                .output();

                            match output {
                                Ok(out) if out.status.success() => {
                                    println!("Success");
                                    String::from_utf8_lossy(&out.stdout).trim().to_string()
                                }
                                Ok(out) => {
                                    eprintln!("Scan failed, stderr: {}", String::from_utf8_lossy(&out.stderr));
                                    String::new()
                                }
                                Err(err) => {
                                    println!("Error running powershell command: {}", err);
                                    String::new()
                                }
                                _ => String::new()
                            }
                        },

                        move |path| {
                            if path.is_empty() {
                                Message::None
                            }
                            else {
                                Message::Scanned(path.into())
                            }
                        }
                    )
                }
                Message::Scanned(temp_path) => {
                    self.scanning = false;
                    self.scan_progress = 1.0;
                    match fs::read(&temp_path) {
                        Ok(bytes) => {
                            self.file_scanned = true;
                            self.file_path_changed = true;
                            self.data_changed = true;
                            self.current_file_bytes = Some(bytes.clone());
                            self.current_file = Some(Handle::from_bytes(bytes.clone()));
                            self.current_file_path = Some(temp_path.to_string_lossy().to_string());
                        },
                        Err(err) => {
                            println!("Error reading scanned image: {}", err);
                            self.current_file = Some(Handle::from_path("src/ferris-error-handling.webp"));
                            self.current_file_path = None;
                        }
                    }
                    Task::none()
                }
                Message::ScanFail => {
                    self.scanning = false;
                    self.scan_progress = 0.0;
                    println!("Scan failed or was cancelled.");
                    Task::none()
                }
                Message::ScanTick => {
                    if self.scanning {
                        self.scan_progress += 0.02;
                        if self.scan_progress > 1.0 {
                            self.scan_progress = 0.0;
                        }
                    }
                    Task::none()
                }
            }
        }


        pub(crate) fn view(&self) -> Element<Message> {
            let test_linear = Linear {
                stops: [
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(1.0, 0.0, 0.0)}),
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(1.0, 0.0, 0.0)}),
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(1.0, 0.0, 0.0)}),
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(1.0, 0.0, 0.0)}),
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(0.0, 0.0, 1.0)}),
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(0.0, 0.0, 1.0)}),
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(0.0, 0.0, 1.0)}),
                    Some(ColorStop { offset: 0.0, color: Color::from_rgb(0.0, 0.0, 1.0)}),
                ],
                angle: 30.into(),
            };

            let test_gradient = Gradient::Linear(test_linear);
            let test_background = Background::Gradient(test_gradient);

            //let mut document_cards: Vec<MouseArea<'static, Message>> = Vec::new();
            let mut document_cards: Vec<DataCard> = Vec::new();

            for document in &self.documents {
                document_cards.push(DataCard::new(Some(document.clone()), None, self.current_theme.clone().unwrap()));
            }

            for card in document_cards.iter() {
                
            }
            
            match &self.current_open_document {
                None => {
                    match self.create_new_document {
                        // New Document Screen
                        true => {
                            Container::new(column![
                                Container::new(row![
                                    button("<").on_press(Message::CloseDocument),
                                    button("Save").on_press(Message::SaveNewDocument)
                                ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                Container::new(column![
                                    row![
                                        Text::new("New Document").size(20).align_y(Center)
                                    ].spacing(5).align_y(Center),
                                    rule::horizontal(2),
                                    row![
                                        Text::new("Document Number: ").width(Length::FillPortion(1)), 
                                        text_input("", &self.current_document_number).on_input(Message::CurrentDocumentNumberChange).id(self.input1_id.as_ref().unwrap().clone()).width(Length::FillPortion(4))
                                    ].spacing(5).align_y(Center),
                                    row![
                                        Text::new("Document Type: ").width(Length::FillPortion(1)), 
                                        text_input("", &self.current_document_type).on_input(Message::CurrentDocumentTypeChange).id(self.input2_id.as_ref().unwrap().clone()).width(Length::FillPortion(4))
                                    ].spacing(5).align_y(Center),
                                    row![
                                        Text::new("Comment: ").width(Length::FillPortion(1)), 
                                        text_input("", &self.current_comment).on_input(Message::CurrentCommentChange).id(self.input3_id.as_ref().unwrap().clone()).width(Length::FillPortion(4))
                                    ].spacing(5).align_y(Center)
                                ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                
                            ].spacing(5)
                            ).height(Length::Fill).width(Length::Fill).padding(5).into()
                        }
                        // Main Document List Screen
                        false => {
                            Container::new(column![
                                Container::new(row![
                                        button("<").on_press(Message::Back),
                                        button("New").on_press(Message::NewDocument),
                                        button("Delete")
                                ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                Container::new(column![
                                    row![
                                        Text::new("Documents").align_y(Center).size(20),
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
                                    ).spacing(10).wrap())
                                ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                            ].spacing(5)
                            ).width(Length::Fill).height(Length::Fill).padding(5).into()
                        }
                    }
                }

                Some(document) => {
                    Container::new(column![
                        match self.current_document_tab {
                            // Document Details Screen
                            Tab::Details => {
                                Container::new(column![
                                    Container::new(row![
                                        button("<").on_press(Message::CloseDocument),
                                        if self.data_changed {
                                            button("Save").on_press(Message::SaveCurrentDocument)
                                        }
                                        else {
                                            button("Save")
                                        }
                                    ].spacing(5).align_y(Center)).width(Length::Fill).padding(5).style(container::bordered_box),
                                    Container::new(column![
                                        row![
                                            Text::new(format!("Document - {}", self.current_document_number),).size(20)
                                        ].spacing(5).align_y(Center),
                                        rule::horizontal(2),
                                        row![
                                            Text::new("Document Number: ").width(Length::FillPortion(1)), 
                                            text_input(&document.get_document_number().to_string(), &self.current_document_number).on_input(Message::CurrentDocumentNumberChange).width(Length::FillPortion(4)).id(self.input1_id.as_ref().unwrap().clone())
                                        ].spacing(5).align_y(Center),
                                        row![
                                            Text::new("Document Type: ").width(Length::FillPortion(1)), 
                                            text_input(&document.get_document_type().to_string(), &self.current_document_type).on_input(Message::CurrentDocumentTypeChange).width(Length::FillPortion(4)).id(self.input2_id.as_ref().unwrap().clone())
                                        ].spacing(5).align_y(Center),
                                        row![
                                            Text::new("Comment: ").width(Length::FillPortion(1)), 
                                            text_input(&document.get_comment().to_string(), &self.current_comment).on_input(Message::CurrentCommentChange).width(Length::FillPortion(4)).id(self.input3_id.as_ref().unwrap().clone())
                                        ].spacing(5).align_y(Center)
                                    ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                ].spacing(5)
                                ).height(Length::Fill).width(Length::Fill)
                            },
                            Tab::Attachments => {
                                let mut attachment_cards: Vec<DataCard> = Vec::new();

                                for attachment in &self.current_open_document.as_ref().unwrap().get_attachments().unwrap() {
                                    attachment_cards.push(DataCard::new(None, Some(attachment.clone()), self.current_theme.clone().unwrap()));
                                }
                                match &self.current_open_attachment {
                                    None => {
                                        match self.create_new_attachment {
                                            // New Attachment Screen
                                            true => {
                                                Container::new(column![
                                                    Container::new(row![
                                                        button("<").on_press(Message::CloseDocument),
                                                        button("Save").on_press(Message::SaveNewAttachment)
                                                    ].spacing(5).align_y(Center)).width(Length::Fill).padding(5).style(container::bordered_box),
                                                    Container::new(column![
                                                        row![
                                                            Text::new("New Attachment").size(20).align_y(Center)
                                                        ].spacing(5).align_y(Center),
                                                        rule::horizontal(2),
                                                        row![
                                                            Container::new(column![
                                                                Text::new("Attachment Number: "), 
                                                                text_input("", &self.current_attachment_reference_number).on_input(Message::CurrentAttachmentReferenceNumberChange).id(self.input1_id.as_ref().unwrap().clone()),
                                                                Text::new("Comment: "), 
                                                                text_input("", &self.current_attachment_comment).on_input(Message::CurrentAttachmentCommentChange).id(self.input2_id.as_ref().unwrap().clone()),
                                                                Text::new("File: "),
                                                                row![
                                                                    text_input("", &self.current_file_path.clone().unwrap_or_else(|| {
                                                                        println!("No currently selected file.");
                                                                        String::new()
                                                                    })),
                                                                    button("Select").on_press(Message::OpenFileDialog),
                                                                    button("Scan").on_press(Message::Scan)
                                                                ].spacing(5).width(Length::FillPortion(4))
                                                            ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)),
                                                            rule::vertical(2),
                                                            Container::new(
                                                                Viewer::new(self.current_file.clone().unwrap_or_else(|| {
                                                                    println!("No selected file yet.");
                                                                    Handle::from_path("src/ferris-error-handling.webp")
                                                                })).width(Length::Fill).height(Length::Fill)
                                                            ).padding(5).style(container::bordered_box).width(Length::FillPortion(3)).height(Length::Fill)
                                                            
                                                        ].spacing(5),
                                                    ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                                ].spacing(5)).width(Length::Fill).height(Length::Fill)
                                            },
                                            // Attachment List Screen
                                            false => {
                                                Container::new(column![
                                                    Container::new(row![
                                                        button("<").on_press(Message::CloseDocument),
                                                        button("New").on_press(Message::NewAttachment)
                                                    ].spacing(5).align_y(Center)).width(Length::Fill).padding(5).style(container::bordered_box),
                                                    Container::new(column![
                                                        row![
                                                            Text::new("Attachments").size(20).align_y(Center),
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
                                                    ].spacing(5)).style(container::bordered_box).padding(5).width(Length::Fill).height(Length::Fill),
                                                ].spacing(5)
                                                ).width(Length::Fill).height(Length::Fill)
                                            }
                                        }
                                    },
                                    // Attachment Details Screen
                                    Some(attachment) => {
                                        Container::new(column![
                                            Container::new(row![
                                                button("<").on_press(Message::CloseAttachment),
                                                if self.data_changed || self.file_path_changed {
                                                    button("Save").on_press(Message::SaveCurrentAttachment)
                                                }
                                                else {
                                                    button("Save")
                                                }
                                            ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                            Container::new(column![
                                                row![
                                                    Text::new(format!("Attachment - {}", self.current_attachment_reference_number)).size(20).align_y(Center)
                                                ].spacing(5).align_y(Center),
                                                rule::horizontal(2),
                                                row![
                                                    Container::new(column![
                                                        Text::new("Attachment Number: "), 
                                                        text_input(&attachment.get_reference_number().to_string(), &self.current_attachment_reference_number).on_input(Message::CurrentAttachmentReferenceNumberChange).id(self.input1_id.as_ref().unwrap().clone()),
                                                        Text::new("Comment: "), 
                                                        text_input(&attachment.get_comment().to_string(), &self.current_attachment_comment).on_input(Message::CurrentAttachmentCommentChange).id(self.input2_id.as_ref().unwrap().clone()),
                                                        Text::new("File: "),
                                                        row![
                                                            text_input("", &self.current_file_path.clone().unwrap_or_else(|| {
                                                                println!("No currently selected file.");
                                                                String::new()
                                                            })),
                                                            button("Select").on_press(Message::OpenFileDialog),
                                                            button("Scan").on_press(Message::Scan)
                                                        ].spacing(5).width(Length::FillPortion(4)),
                                                        ProgressBar::new(0.0..=1.0, self.scan_progress)
                                                    ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)).height(Length::Fill),
                                                    rule::vertical(2),
                                                    Container::new(
                                                        Viewer::new(self.current_file.clone().unwrap_or_else(|| {
                                                            println!("Error displaying image.");
                                                            Handle::from_path("src/ferris-error-handling.webp")
                                                        })).width(Length::Fill).height(Length::Fill)
                                                    ).padding(5).style(container::bordered_box).width(Length::FillPortion(3)).height(Length::Fill)
                                                ].spacing(5),
                                            ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                        ].spacing(5)
                                        ).height(Length::Fill).width(Length::Fill)
                                    }
                                }
                            },
                        },
                        Container::new(
                            // Navigation Tabs between Document details and attachments
                            Tabs::new(Message::SwitchTab)
                                .push(Tab::Details, TabLabel::Text(String::from("Details")), Space::new())
                                .push(Tab::Attachments, TabLabel::Text(String::from("Attachments")), Space::new())
                                .tab_bar_position(TabBarPosition::Bottom)
                                .set_active_tab(&self.current_document_tab)
                                .height(Length::Shrink)
                        )
                    ].spacing(5)).padding(5).into()
                }
            }
        }

        pub(crate) fn subscription(&self) -> Subscription<Message> {
            let kb_event = keyboard::listen().map(|event| {
                match event {
                    keyboard::Event::KeyPressed { key, ..} => {
                        Message::KeyEvent(key)
                    }
                    _ => Message::None
                }
            });

            let tick_event = if self.scanning {
                iced::time::every(iced::time::Duration::from_millis(100)).map(|_| Message::ScanTick)
            }
            else {
                Subscription::none()
            };

            Subscription::batch(vec![kb_event, tick_event])
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
            let datetime = Result::expect (
                Result::expect (
                    UtcDateTime::from_unix_timestamp(self.document.as_ref().unwrap().get_date_added()),
                    "Error retrieving data from Document.").to_offset(OffsetDateTime::now_local().expect("Failed to acquire local offset.").offset())
            .format(datetime_format),
            "Error converting unix epoch to UtcDateTime");

            mouse_area(
                Card::new(Text::new(self.document.as_ref().unwrap().get_document_number().to_string()), column![
                    Text::new(self.document.as_ref().unwrap().get_document_type().to_string()),
                    Text::new(self.document.as_ref().unwrap().get_comment().to_string())
                ]).max_height(500.0).max_width(200.0).foot(Text::new(datetime)).style(|theme: &Theme, _| card_style(theme))
            ).on_press(Message::OpenDocument(self.document.as_ref().unwrap().clone())).interaction(Interaction::Pointer)
        }

        fn new_attachment_card(&self) -> MouseArea<'static, Message> {
            let datetime_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
            let datetime = Result::expect (
                Result::expect (
                    UtcDateTime::from_unix_timestamp(self.attachment.as_ref().unwrap().get_date_added()),
                    "Error retrieving data from Attachment.").to_offset(OffsetDateTime::now_local().expect("Failed to acquire local offset.").offset())
            .format(datetime_format),
            "Error converting unix epoch to UtcDateTime");

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

    #[derive(Debug, Clone)]
    pub(crate) enum Message {
        NewDocument,
        OpenDocument(Arc<Document>),
        CurrentDocumentNumberChange(String),
        CurrentDocumentTypeChange(String),
        CurrentCommentChange(String),
        SaveCurrentDocument,
        SaveNewDocument,
        SwitchTab(Tab),
        CloseDocument,
        NewAttachment,
        SaveNewAttachment,
        OpenAttachment(Arc<Attachment>),
        SaveCurrentAttachment,
        CurrentAttachmentReferenceNumberChange(String),
        CurrentAttachmentCommentChange(String),
        CloseAttachment,
        SearchTextChange(String),
        OpenFileDialog,
        Back,
        KeyEvent(Key),
        Scan,
        Scanned(PathBuf),
        ScanFail,
        ScanTick,
        None,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Default)]
    pub(crate) enum Tab {
        #[default]
        Details,
        Attachments,
    }

    pub(crate) enum ActiveInput {
        Input1,
        Input2,
        Input3
    }
}