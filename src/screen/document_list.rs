pub(crate) mod document_list {
    use std::{env::{current_dir, current_exe}, fs, io::Cursor, path::PathBuf, process::Stdio, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

    use caesium::{compress_in_memory, convert_in_memory, parameters::{CSParameters, PngParameters}};
    use file_format::FileFormat;
    use iced::{Alignment::Center, Background, Border, Color, Element, Event, Gradient, Length, Renderer, Shadow, Subscription, Task, Theme, advanced::graphics::futures::subscription, border::Radius, gradient::{ColorStop, Linear}, keyboard::{self, Key, key}, mouse::Interaction, theme::Palette, wgpu::rwh, widget::{Container, Id, MouseArea, ProgressBar, Space, Stack, Text, TextInput, button, center, column, container::{self, Style}, image::{Handle, Viewer}, mouse_area, operation::focus_next, progress_bar, row, rule, scrollable}, window::events};
    use iced::widget::text_input;
    use iced_aw::{Card, TabBarPosition, TabLabel, Tabs, card::Status, style::card};
    use iced_dialog::dialog;
    use image::{DynamicImage, ImageBuffer};
    use pdfium_render::prelude::{PdfBitmap, PdfBitmapFormat, PdfPageImageObject, PdfPageObjectsCommon, PdfPageOrientation, PdfPagePaperSize, PdfPageRenderRotation, PdfPoints, PdfRenderConfig, Pdfium, PdfiumError, PdfiumLibraryBindings};
    use rfd::FileDialog;
    use rusqlite::ffi::SQLITE_LIMIT_FUNCTION_ARG;
    use time::{Duration, OffsetDateTime, UtcDateTime, macros::format_description};

    use crate::{ERROR_FERRIS, LocalTheme, State, attachment::attachment::Attachment, attachment_page::attachment_page::AttachmentPage, db::db_module::DbConnection, document::document::Document};

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
        selected_file_paths: Option<Vec<PathBuf>>,
        selected_file_bytes: Option<Vec<Vec<u8>>>,
        scanned_file_bytes: Option<Vec<Vec<u8>>>,
        current_file_bytes: Option<Vec<Vec<u8>>>,
        current_file_handles: Option<Vec<Handle>>,
        current_file: Option<Handle>,
        current_page_index: usize,
        file_scanned: bool,
        files_changed: bool,
        input1_id: Option<Id>,
        input2_id: Option<Id>,
        input3_id: Option<Id>,
        scanning: bool,
        scan_progress: f32,
        current_theme: Option<LocalTheme>,
        show_confirm_delete: bool,
        show_empty_field_warning: bool
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
                selected_file_paths: None,
                selected_file_bytes: None,
                scanned_file_bytes: None,
                current_file_handles: None,
                current_file: None,
                current_file_bytes: None,
                current_page_index: 0,
                file_scanned: false,
                files_changed: false,
                input1_id: Some(Id::new("1")),
                input2_id: Some(Id::new("2")),
                input3_id: Some(Id::new("3")),
                scanning: false,
                scan_progress: f32::default(),
                current_theme: None,
                show_confirm_delete: false,
                show_empty_field_warning: false
            }
        }

        pub(crate) fn set_current_theme(&mut self, theme: LocalTheme) {
            self.current_theme = Some(theme);
        }

        pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::NewDocument => {
                    self.reset_state();
                    self.create_new_document = true;
                    Task::none()
                },
                Message::SaveNewDocument => {
                    if self.current_document_number.is_empty() {
                        self.show_empty_field_warning = true;
                    }
                    else {
                        let mut conn = DbConnection::new();
                        conn.new_document(
                            self.current_document_number.clone(),
                            self.current_document_type.clone(), 
                            self.current_comment.clone()
                        ).unwrap_or_else(|err| {
                            println!("Error adding new document: {}", err);
                            0
                        });

                        self.reset_state();
                        self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                        self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == conn.last_rowid().unwrap() as u32).cloned();
                        self.current_document_number = self.current_open_document.as_ref().unwrap().get_document_number().to_string();
                        self.current_document_type = self.current_open_document.as_ref().unwrap().get_document_type().to_string();
                        self.current_comment = self.current_open_document.as_ref().unwrap().get_comment().to_string();
                        let file_path = format!("./data/{}", self.current_open_document.as_ref().unwrap().get_document_number());
                        fs::create_dir(file_path).unwrap_or_else(|err| {
                            println!("Error creating document's attachment folder: {}", err);
                        });
                    }
                    
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
                    if self.current_document_number.is_empty() {
                        self.show_empty_field_warning = true;
                    }
                    else {
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

                        self.reset_state();
                        self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                        self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                    }
                    
                    Task::none()
                }
                Message::SwitchTab(tab) => {
                    self.current_document_tab = tab;
                    Task::none()
                },
                Message::CloseDocument => {
                    self.reset_state();
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
                    self.reset_attachment_state();
                    self.create_new_attachment = true;
                    self.current_file_handles = Some(Vec::new());
                    Task::none()
                },
                Message::OpenFileDialog => {
                    let previous_file_paths = self.selected_file_paths.clone();
                    self.selected_file_paths = FileDialog::new().set_title("Select Document")
                        .add_filter("Image (.png, .jpg, .jpeg, .webp)", &["png", "jpg", "jpeg", "webp"])
                        .add_filter("PDF (.pdf)", &["pdf"])
                        .pick_files().and_then(|paths| {
                            self.files_changed = true;
                            Some(paths)
                        }
                    );
                    if previous_file_paths != self.selected_file_paths && self.selected_file_paths.is_some() {
                        self.files_changed = true;
                        let mut selected_file_bytes: Vec<Vec<u8>> = Vec::new();
                        for path in self.selected_file_paths.as_ref().unwrap() {
                            match fs::read(&path) {
                                Ok(bytes) => {
                                    if file_format::FileFormat::from_bytes(&bytes) == FileFormat::PortableDocumentFormat {
                                        for image in pdf_to_png(bytes) {
                                            selected_file_bytes.push(image);
                                        }
                                    }
                                    else {
                                        selected_file_bytes.push(bytes);
                                    }
                                }
                                Err(err) => {
                                    println!("Error reading files from paths: {}", err);
                                }
                            }
                        }

                        for bytes in selected_file_bytes {
                            self.add_file_bytes(bytes);
                        }

                        self.update_file_handles();
                        self.current_page_index = 0;
                    }
                    Task::none()
                },
                Message::SaveNewAttachment => {
                    if self.current_attachment_reference_number.is_empty() || self.current_file_bytes.is_none() {
                        self.show_empty_field_warning = true;
                    }
                    else {
                        let file_path = format!("./data/{}/{}", self.current_open_document.as_ref().unwrap().get_document_number(), self.current_attachment_reference_number);
                        fs::create_dir(&file_path).unwrap_or_else(|err| {
                            println!("Error creating document's attachment folder: {}", err);
                        });
                        let mut conn = DbConnection::new();
                        let current_document_id = self.current_open_document.clone().unwrap().get_document_id();
                        let current_document_number = &self.current_document_number;

                        let mut data_file_paths: Vec<PathBuf> = Vec::new();
                        for (index, _) in self.current_file_handles.as_ref().unwrap().iter().enumerate() {
                            data_file_paths.push(format!("{}/{}_{}_{}.png", &file_path, current_document_number, self.current_attachment_reference_number, index + 1).into())
                        }
                        
                        conn.new_attachment(data_file_paths, self.current_attachment_reference_number.clone(), self.current_attachment_comment.clone(), current_document_id).unwrap_or_else(|err| {
                            println!("Error creating new attachment: {}", err);
                        });

                        for (index, bytes) in self.current_file_bytes.as_mut().unwrap().iter_mut().enumerate() {
                            let file_name = format!("{}_{}_{}.png", current_document_number, self.current_attachment_reference_number, index + 1);
                            let file_path = format!("./data/{}/{}/{}", current_document_number, self.current_attachment_reference_number, file_name);
                            if FileFormat::from_bytes(&bytes) != FileFormat::PortableNetworkGraphics {
                                let img = image::load_from_memory(&bytes);
                                match img.unwrap().write_to(&mut Cursor::new(&mut *bytes), image::ImageFormat::Png) {
                                    Err(err) => println!("Error converting image format: {}", err),
                                    _ => {}
                                }
                            }

                            let compressed_bytes = compress_image(bytes.to_vec());

                            fs::write(&file_path, compressed_bytes).unwrap_or_else(|err| {
                                println!("Error writing file to data folder: {}", err);
                            });
                        }

                        self.reset_attachment_state();
                        self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                        self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                        self.current_open_attachment = self.current_open_document.as_ref().unwrap().get_attachments().unwrap().iter().find(|attachment| attachment.get_attachment_id() == conn.last_rowid().unwrap() as u32).cloned();
                        self.current_attachment_reference_number = self.current_open_attachment.as_ref().unwrap().get_reference_number().to_string();
                        self.current_attachment_comment = self.current_open_attachment.as_ref().unwrap().get_comment().to_string();
                        self.current_file_bytes = Some(Vec::new());
                        for page in self.current_open_attachment.as_ref().unwrap().pages() {
                            self.current_file_bytes.as_mut().unwrap().push(page.image().to_vec());
                        }
                        self.update_file_handles();
                    }
                    Task::none()
                },
                Message::OpenAttachment(attachment) => {
                    self.current_open_attachment = Some(attachment.clone());
                    self.current_attachment_reference_number = attachment.clone().get_reference_number().to_string();
                    self.current_attachment_comment = attachment.clone().get_comment().to_string();
                    self.current_file_bytes = Some(Vec::new());
                    self.current_file_handles = Some(Vec::new());
                    for page in self.current_open_attachment.as_ref().unwrap().pages() {
                        self.current_file_bytes.as_mut().unwrap().push(page.image().to_vec());
                    }
                    self.update_file_handles();

                    Task::none()
                },
                Message::SaveCurrentAttachment => {
                    if self.current_attachment_reference_number.is_empty() {
                        self.show_empty_field_warning = true;
                    }
                    else {
                        let mut conn = DbConnection::new();
                        let current_document_id = self.current_open_document.as_ref().unwrap().get_document_id();
                        let current_document_number = self.current_open_document.as_ref().unwrap().get_document_number();
                        let current_attachment_id = self.current_open_attachment.as_ref().unwrap().get_attachment_id();
                        let old_attachment_reference_number = self.current_open_attachment.as_ref().unwrap().get_reference_number();
                        let mut old_file_paths: Vec<String> = Vec::new();
                        for page in self.current_open_attachment.as_ref().unwrap().pages() {
                            old_file_paths.push(page.file_path().to_string());
                        }
                        
                        conn.edit_attachment_details(
                            current_attachment_id,
                            self.current_attachment_reference_number.clone(),
                            self.current_attachment_comment.clone()
                        ).unwrap_or_else(|err| {
                            println!("Error editing attachment: {}", err);
                            0
                        });

                        if self.files_changed {
                            let path = format!("./data/{}/{}", current_document_number, self.current_open_attachment.as_ref().unwrap().get_reference_number());
                            fs::remove_dir_all(&path).expect("Error deleting old directory");
                            let mut file_paths: Vec<PathBuf> = Vec::new();
                            let file_dir = format!("./data/{}/{}", current_document_number, self.current_attachment_reference_number);
                            fs::create_dir(&file_dir).expect("Error creating new directory");

                            for (index, bytes) in self.current_file_bytes.as_mut().unwrap().iter_mut().enumerate() {
                                let file_name = format!("{}_{}_{}.png", current_document_number, self.current_attachment_reference_number, index + 1);
                                let file_path = format!("{}/{}", file_dir, file_name);
                                
                                if FileFormat::from_bytes(&bytes) != FileFormat::PortableNetworkGraphics {
                                    let img = image::load_from_memory(&bytes);
                                    match img.unwrap().write_to(&mut Cursor::new(&mut *bytes), image::ImageFormat::Png) {
                                        Err(err) => println!("Error converting image format: {}", err),
                                        _ => {}
                                    }
                                }

                                let compressed_bytes = compress_image(bytes.to_vec());

                                fs::write(&file_path, compressed_bytes).unwrap_or_else(|err| {
                                    println!("Error writing file to data folder: {}", err);
                                });

                                file_paths.push(file_path.into());
                            }

                            conn.edit_attachment_pages(self.current_open_attachment.as_ref().unwrap().get_attachment_id(), file_paths).expect("Error editing attachment pages");

                        }
                        else {
                            let mut new_file_paths: Vec<PathBuf> = Vec::new();
                            let old_file_dir = format!("./data/{}/{}", current_document_number, old_attachment_reference_number);
                            let new_file_dir = format!("./data/{}/{}", current_document_number, self.current_attachment_reference_number);
                            
                            for (index, path) in old_file_paths.into_iter().enumerate() {
                                let file_name = format!("{}_{}_{}.png", current_document_number, self.current_attachment_reference_number, index + 1);
                                let old_file_path = format!("{}/{}", old_file_dir, file_name);

                                fs::rename(path, &old_file_path).expect("Error renaming file");
                                let new_file_path = format!("{}/{}", new_file_dir, file_name);
                                new_file_paths.push(new_file_path.into());
                            }
                            fs::rename(old_file_dir, &new_file_dir).expect("Error renaming directory");

                            conn.edit_attachment_pages(self.current_open_attachment.as_ref().unwrap().get_attachment_id(), new_file_paths).expect("Error editing attachment pages");
                        }

                        self.reset_attachment_state();
                        self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                        self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                        self.current_open_attachment = self.current_open_document.as_ref().unwrap().get_attachments().unwrap().iter().find(|attachment| attachment.get_attachment_id() == current_attachment_id).cloned();
                        self.current_attachment_reference_number = self.current_open_attachment.as_ref().unwrap().get_reference_number().to_string();
                        self.current_attachment_comment = self.current_open_attachment.as_ref().unwrap().get_comment().to_string();
                        self.current_file_bytes = Some(Vec::new());
                        self.current_file_handles = Some(Vec::new());
                        for page in self.current_open_attachment.as_ref().unwrap().pages() {
                            self.current_file_bytes.as_mut().unwrap().push(page.image().to_vec());
                        }
                        self.update_file_handles();
                    }

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
                    self.reset_attachment_state();
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
                },
                Message::Scanned(temp_path) => {
                    self.scanning = false;
                    self.scan_progress = 1.0;
                    match fs::read(&temp_path) {
                        Ok(bytes) => {
                            self.file_scanned = true;
                            self.files_changed = true;
                            self.data_changed = true;
                            self.add_file_bytes(bytes);
                            self.update_file_handles();
                        },
                        Err(err) => {
                            println!("Error reading scanned image: {}", err);
                            self.current_file = Some(Handle::from_bytes(ERROR_FERRIS));
                            self.current_file_path = None;
                        }
                    }
                    Task::none()
                },
                Message::ScanFail => {
                    self.scanning = false;
                    self.scan_progress = 0.0;
                    println!("Scan failed or was cancelled.");
                    Task::none()
                },
                Message::ScanTick => {
                    if self.scanning {
                        self.scan_progress += 0.02;
                        if self.scan_progress > 1.0 {
                            self.scan_progress = 0.0;
                        }
                    }
                    Task::none()
                },
                Message::FinalizeScan => {
                    for bytes in self.scanned_file_bytes.as_ref().unwrap() {
                        self.current_file_handles.as_mut().unwrap().push(Handle::from_bytes(bytes.to_vec()));
                    }
                    Task::none()
                },
                Message::ClearImageFiles => {
                    self.current_file_bytes = None;
                    self.update_file_handles();
                    Task::none()
                }
                Message::DeleteDocument => {
                    let mut conn = DbConnection::new();
                    conn.delete_document(self.current_open_document.as_ref().unwrap().get_document_id());
                    match fs::remove_dir_all(format!("./data/{}", self.current_open_document.as_ref().unwrap().get_document_number())) {
                        Err(err) => println!("Error deleting data directory: {}", err),
                        Ok(_) => {}
                    }

                    self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                    self.reset_state();
                    
                    Task::none()
                },
                Message::DeleteAttachment => {
                    let mut conn = DbConnection::new();
                    match conn.delete_attachment(self.current_open_attachment.as_ref().unwrap().get_attachment_id()) {
                        Ok(_) => {},
                        Err(err) => println!("Error deleting attachment: {}", err)
                    }
                    match fs::remove_dir_all(format!("./data/{}/{}", self.current_open_document.as_ref().unwrap().get_document_number(), self.current_open_attachment.as_ref().unwrap().get_reference_number())) {
                        Err(err) => println!("Error deleting file: {}", err),
                        Ok(_) => {}
                    }

                    let current_document_id = self.current_open_document.as_ref().unwrap().get_document_id();
                    self.documents = Result::expect(conn.read_document_table(), "Error retrieving data from database");
                    self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                    self.reset_attachment_state();
                    
                    Task::none()
                },
                Message::ShowConfirmDelete => {
                    self.show_confirm_delete = true;
                    Task::none()
                },
                Message::ExportToPdf => {
                    let file_name = format!("{}_{}.pdf", self.current_open_document.as_ref().unwrap().get_document_number(), self.current_open_attachment.as_ref().unwrap().get_reference_number());
                    let path = format!("./data/{}/{}/{}", self.current_open_document.as_ref().unwrap().get_document_number(), self.current_open_attachment.as_ref().unwrap().get_reference_number(), file_name);
                    export_to_pdf(self.current_file_bytes.as_ref().unwrap().to_vec(), path.into());
                    Task::none()
                },
                Message::PrevPage => {
                    if self.current_page_index > 0 {
                        self.current_page_index -= 1;
                    }
                    Task::none()
                },
                Message::NextPage => {
                    if self.current_page_index < self.current_file_handles.as_ref().unwrap().len() - 1 {
                        self.current_page_index += 1;
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

            match &self.current_open_document {
                None => {
                    match self.create_new_document {
                        // New Document Screen
                        true => {
                            Container::new(column![
                                Container::new(row![
                                    button("<").on_press(Message::CloseDocument),
                                    button("Save").on_press(Message::SaveNewDocument),
                                ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                Container::new(column![
                                    row![
                                        Text::new("New Document").size(20).align_y(Center)
                                    ].spacing(5).align_y(Center),
                                    rule::horizontal(2),
                                    row![
                                        row![
                                            Text::new("Document Number "),
                                            Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                        ].width(Length::FillPortion(1)),
                                        if self.show_empty_field_warning && self.current_document_number.is_empty() {
                                            text_input("", &self.current_document_number).on_input(Message::CurrentDocumentNumberChange).id(self.input1_id.as_ref().unwrap().clone()).width(Length::FillPortion(4)).style(|theme, _| empty_text_input_warning(theme))
                                        }
                                        else {
                                            text_input("", &self.current_document_number).on_input(Message::CurrentDocumentNumberChange).id(self.input1_id.as_ref().unwrap().clone()).width(Length::FillPortion(4))
                                        }
                                    ].spacing(5).align_y(Center),
                                    row![
                                        Text::new("Document Type").width(Length::FillPortion(1)), 
                                        text_input("", &self.current_document_type).on_input(Message::CurrentDocumentTypeChange).id(self.input2_id.as_ref().unwrap().clone()).width(Length::FillPortion(4))
                                    ].spacing(5).align_y(Center),
                                    row![
                                        Text::new("Comment").width(Length::FillPortion(1)), 
                                        text_input("", &self.current_comment).on_input(Message::CurrentCommentChange).id(self.input3_id.as_ref().unwrap().clone()).width(Length::FillPortion(4))
                                    ].spacing(5).align_y(Center)
                                ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                
                            ].spacing(5)
                            ).height(Length::Fill).width(Length::Fill).into()
                        }
                        // Main Document List Screen
                        false => {
                            Container::new(column![
                                Container::new(row![
                                        button("<").on_press(Message::Back),
                                        button("New").on_press(Message::NewDocument)
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
                                    ).spacing(10).wrap()),
                                ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                            ].spacing(5)
                            ).width(Length::Fill).height(Length::Fill).into()
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
                                        },
                                        button("New").on_press(Message::NewDocument),
                                        Space::new().width(Length::Fill),
                                        if self.show_confirm_delete {
                                            row![
                                                Text::from("Confirm deletion: "),
                                                button("Confirm").on_press(Message::DeleteDocument),
                                                button("Cancel")
                                            ].spacing(5).align_y(Center)
                                        }
                                        else {
                                            row![button("Delete").on_press(Message::ShowConfirmDelete)]
                                        }
                                    ].spacing(5).align_y(Center)).width(Length::Fill).padding(5).style(container::bordered_box),
                                    Container::new(column![
                                        row![
                                            Text::new(format!("Document - {}", self.current_document_number),).size(20)
                                        ].spacing(5).align_y(Center),
                                        rule::horizontal(2),
                                        row![
                                            row![
                                                Text::new("Document Number "),
                                                Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                            ].width(Length::FillPortion(1)),
                                            if self.show_empty_field_warning && self.current_document_number.is_empty() {
                                                text_input(&document.get_document_number().to_string(), &self.current_document_number).on_input(Message::CurrentDocumentNumberChange).width(Length::FillPortion(4)).id(self.input1_id.as_ref().unwrap().clone()).style(|theme, _| empty_text_input_warning(theme))
                                            }
                                            else {
                                                text_input(&document.get_document_number().to_string(), &self.current_document_number).on_input(Message::CurrentDocumentNumberChange).width(Length::FillPortion(4)).id(self.input1_id.as_ref().unwrap().clone())
                                            }
                                        ].spacing(5).align_y(Center),
                                        row![
                                            Text::new("Document Type").width(Length::FillPortion(1)), 
                                            text_input(&document.get_document_type().to_string(), &self.current_document_type).on_input(Message::CurrentDocumentTypeChange).width(Length::FillPortion(4)).id(self.input2_id.as_ref().unwrap().clone())
                                        ].spacing(5).align_y(Center),
                                        row![
                                            Text::new("Comment").width(Length::FillPortion(1)), 
                                            text_input(&document.get_comment().to_string(), &self.current_comment).on_input(Message::CurrentCommentChange).width(Length::FillPortion(4)).id(self.input3_id.as_ref().unwrap().clone())
                                        ].spacing(5).align_y(Center),
                                        
                                    ].spacing(5)).padding(5).style(container::bordered_box).width(Length::Fill).height(Length::Fill),
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
                                                        button("<").on_press(Message::CloseAttachment),
                                                        button("Save").on_press(Message::SaveNewAttachment)
                                                    ].spacing(5).align_y(Center)).width(Length::Fill).padding(5).style(container::bordered_box),
                                                    Container::new(column![
                                                        row![
                                                            Text::new("New Attachment").size(20).align_y(Center)
                                                        ].spacing(5).align_y(Center),
                                                        rule::horizontal(2),
                                                        row![
                                                            Container::new(column![
                                                                row![
                                                                        Text::new("Attachment Number "),
                                                                        Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                                                ].width(Length::FillPortion(1)),
                                                                if self.show_empty_field_warning && self.current_attachment_reference_number.is_empty() {
                                                                    text_input("", &self.current_attachment_reference_number).on_input(Message::CurrentAttachmentReferenceNumberChange).id(self.input1_id.as_ref().unwrap().clone()).style(|theme, _| empty_text_input_warning(theme))
                                                                }
                                                                else {
                                                                    text_input("", &self.current_attachment_reference_number).on_input(Message::CurrentAttachmentReferenceNumberChange).id(self.input1_id.as_ref().unwrap().clone())
                                                                },
                                                                Text::new("Comment"), 
                                                                text_input("", &self.current_attachment_comment).on_input(Message::CurrentAttachmentCommentChange).id(self.input2_id.as_ref().unwrap().clone()),
                                                                row![
                                                                    Text::new("Image Files "),
                                                                    Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                                                ].width(Length::FillPortion(1)),
                                                                row![
                                                                    if self.show_empty_field_warning && self.current_file_bytes.is_none() {
                                                                        text_input("", &self.current_file_handles.as_ref().unwrap().len().to_string()).style(|theme, _| empty_text_input_warning(theme))
                                                                    }
                                                                    else {
                                                                        if self.current_file_handles.is_none() {
                                                                            text_input("", "0")
                                                                        }
                                                                        else {
                                                                            text_input("", &self.current_file_handles.as_ref().unwrap().len().to_string())
                                                                        }
                                                                    },
                                                                    button("Select").on_press(Message::OpenFileDialog),
                                                                    button("Scan").on_press(Message::Scan)
                                                                ].spacing(5).width(Length::FillPortion(4)),
                                                                button(Text::new("Clear").align_x(Center).width(Length::Fill)).on_press(Message::ClearImageFiles).width(Length::Fill)
                                                            ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)),
                                                            rule::vertical(2),
                                                            Container::new(
                                                                column![
                                                                    if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                        Viewer::new(Handle::from_bytes(ERROR_FERRIS)).width(Length::Fill).height(Length::Fill)
                                                                    }
                                                                    else {
                                                                        Viewer::new(self.current_file_handles.as_ref().unwrap()[self.current_page_index].clone()).width(Length::Fill).height(Length::Fill)
                                                                    },
                                                                    rule::horizontal(2),
                                                                    row![
                                                                        if self.current_page_index > 0 {
                                                                            button("<").on_press(Message::PrevPage)
                                                                        }
                                                                        else {
                                                                            button("<")
                                                                        },
                                                                        Text::new(self.current_page_index + 1),
                                                                        if self.current_page_index + 1 < self.current_file_handles.as_ref().unwrap().len() {
                                                                            button(">").on_press(Message::NextPage)
                                                                        }
                                                                        else {
                                                                            button(">")
                                                                        }
                                                                    ].spacing(10).align_y(Center)
                                                                ].spacing(5).align_x(Center).width(Length::Fill).height(Length::Fill)
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
                                                if self.data_changed || self.files_changed {
                                                    button("Save").on_press(Message::SaveCurrentAttachment)
                                                }
                                                else {
                                                    button("Save")
                                                },
                                                button("New").on_press(Message::NewAttachment),
                                                Space::new().width(Length::Fill),
                                                if self.show_confirm_delete {
                                                    row![
                                                        Text::from("Confirm deletion: "),
                                                        button("Confirm").on_press(Message::DeleteAttachment),
                                                        button("Cancel")
                                                    ].spacing(5).align_y(Center)
                                                }
                                                else {
                                                    row![button("Delete").on_press(Message::ShowConfirmDelete)]
                                                }
                                                
                                            ].spacing(5)).width(Length::Fill).padding(5).style(container::bordered_box),
                                            Container::new(column![
                                                row![
                                                    Text::new(format!("Attachment - {}", self.current_attachment_reference_number)).size(20).align_y(Center)
                                                ].spacing(5).align_y(Center),
                                                rule::horizontal(2),
                                                row![
                                                    Container::new(column![
                                                        row![
                                                            Text::new("Attachment Number "),
                                                            Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                                        ].width(Length::FillPortion(1)),
                                                        if self.show_empty_field_warning && self.current_attachment_reference_number.is_empty() {
                                                            text_input(&attachment.get_reference_number().to_string(), &self.current_attachment_reference_number).on_input(Message::CurrentAttachmentReferenceNumberChange).id(self.input1_id.as_ref().unwrap().clone()).style(|theme, _| empty_text_input_warning(theme))
                                                        }
                                                        else {
                                                            text_input(&attachment.get_reference_number().to_string(), &self.current_attachment_reference_number).on_input(Message::CurrentAttachmentReferenceNumberChange).id(self.input1_id.as_ref().unwrap().clone())
                                                        },
                                                        Text::new("Comment"), 
                                                        text_input(&attachment.get_comment().to_string(), &self.current_attachment_comment).on_input(Message::CurrentAttachmentCommentChange).id(self.input2_id.as_ref().unwrap().clone()),
                                                        row![
                                                            Text::new("Image File "),
                                                            Text::new("*").color(Color::from_rgb(1.0, 0.0, 0.0))
                                                        ].width(Length::FillPortion(1)),
                                                        column![
                                                            row![
                                                                if self.show_empty_field_warning && self.current_file_path.is_none() {
                                                                    text_input("", &self.current_file_handles.as_ref().unwrap().len().to_string().as_str()).style(|theme, _| empty_text_input_warning(theme))
                                                                }
                                                                else {
                                                                    text_input("", &self.current_file_handles.as_ref().unwrap().len().to_string().as_str())
                                                                },
                                                                button("Select").on_press(Message::OpenFileDialog),
                                                                button("Scan").on_press(Message::Scan),
                                                            ].spacing(5).width(Length::Fill),
                                                            row![
                                                                button(Text::new("Export").center()).on_press(Message::ExportToPdf).width(Length::FillPortion(1)),
                                                                button(Text::new("Clear").center()).on_press(Message::ClearImageFiles).width(Length::FillPortion(1))
                                                            ].spacing(5)
                                                        ].spacing(5),
                                                        ProgressBar::new(0.0..=1.0, self.scan_progress)
                                                    ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)).height(Length::Fill),
                                                    rule::vertical(2),
                                                    Container::new(
                                                        column![
                                                            if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                Viewer::new(Handle::from_bytes(ERROR_FERRIS)).width(Length::Fill).height(Length::Fill)
                                                            }
                                                            else {
                                                                Viewer::new(self.current_file_handles.as_ref().unwrap()[self.current_page_index].clone()).width(Length::Fill).height(Length::Fill)
                                                            },
                                                            rule::horizontal(2),
                                                            row![
                                                                if self.current_page_index > 0 {
                                                                    button("<").on_press(Message::PrevPage)
                                                                }
                                                                else {
                                                                    button("<")
                                                                },
                                                                Text::new(self.current_page_index + 1),
                                                                if self.current_page_index + 1 < self.current_file_handles.as_ref().unwrap().len() {
                                                                    button(">").on_press(Message::NextPage)
                                                                }
                                                                else {
                                                                    button(">")
                                                                }
                                                            ].spacing(10).align_y(Center)
                                                        ].spacing(5).align_x(Center)
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
                    ].spacing(5)).into()
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

        fn add_file_bytes(&mut self, bytes: Vec<u8>) {
            if self.current_file_bytes.is_none() {
                self.current_file_bytes = Some(Vec::new());
            }
            self.current_file_bytes.as_mut().unwrap().push(bytes);
        }

        fn update_file_handles(&mut self) {
            if self.current_file_handles.is_none() {
                self.current_file_handles = Some(Vec::new());
            }
            self.current_file_handles.as_mut().unwrap().clear();
            if self.current_file_bytes.is_none() {
                self.current_file_handles.as_mut().unwrap().clear();
            }
            else {
                for bytes in self.current_file_bytes.as_ref().unwrap() {
                    self.current_file_handles.as_mut().unwrap().push(Handle::from_bytes(bytes.to_vec()));
                }
            }
            
        }

        fn reset_state(&mut self) {
            self.current_open_document = None;
            self.current_document_number.clear();
            self.current_document_type.clear();
            self.current_comment.clear();
            self.create_new_document = false;
            self.reset_attachment_state();
        }

        fn reset_attachment_state(&mut self) {
            self.current_open_attachment = None;
            self.current_attachment_reference_number.clear();
            self.current_attachment_comment.clear();
            self.current_file = None;
            self.current_file_bytes = None;
            self.current_file_path = None;
            self.data_changed = false;
            self.files_changed = false;
            self.file_scanned = false;
            self.scanning = false;
            self.scan_progress = 0.0;
            self.create_new_attachment = false;
            self.show_confirm_delete = false;
            self.show_empty_field_warning = false;
            self.current_page_index = 0;
            self.current_file_handles = None;
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

    fn empty_text_input_warning(theme: &Theme) -> text_input::Style {
        text_input::Style {
            border: Border {
                color: Color::from_rgb(1.0, 0.0, 0.0),
                width: 2.0,
                radius: text_input::default(theme, text_input::Status::Active).border.radius
            },
            ..text_input::default(theme, text_input::Status::Active)
        }
    }

    fn compress_image(bytes: Vec<u8>) -> Vec<u8> {
        let mut parameters = CSParameters::new();
        parameters.png.quality = 100;
        parameters.png.optimization_level = 6;
        
        let compressed_bytes = compress_in_memory(bytes.to_vec(), &parameters).unwrap_or_else(|err| {
            println!("Error compressing image: {}", err);
            Vec::new()
        });

        return compressed_bytes
    }

    fn pdf_to_png(bytes: Vec<u8>) -> Vec<Vec<u8>> {
        let pdfium = Pdfium::default();
        let document = pdfium.load_pdf_from_byte_vec(bytes, None);
        let config = PdfRenderConfig::new()
            .rotate_if_landscape(PdfPageRenderRotation::None, true)
            .set_fixed_size(2480, 3508);
        let mut bitmaps: Vec<Vec<u8>> = Vec::new();

        for page in Result::expect(document, "Error unwrapping PDF Document").pages().iter() {
            let mut bytes: Vec<u8> = Vec::new();
            page.render_with_config(&config).unwrap()
                .as_image()
                .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Bmp)
                .expect("Error converting PDF to image bytes");

            bitmaps.push(bytes);
        }

        return bitmaps
    }

    fn export_to_pdf(byte_vec: Vec<Vec<u8>>, path: PathBuf) {
        let pdfium = Pdfium::default();
        let mut document = Result::expect(pdfium.create_new_pdf(), "Error creating new document");
        
        for bytes in byte_vec {
            let mut width: f32 = 0.0;
            let mut height: f32 = 0.0;
            let raw_bytes = match image::load_from_memory(&bytes) {
                Ok(bytes) => {
                    let mut converted_bytes: Vec<u8> = Vec::new();
                    bytes.write_to(&mut Cursor::new(&mut converted_bytes), image::ImageFormat::Png).expect("Error converting image");

                    let mut parameters = CSParameters::new();
                    parameters.png.quality = 10;
                    parameters.png.optimization_level = 6;
                    let compressed_bytes = compress_in_memory(converted_bytes, &parameters).expect("Error compressing images for PDF");
                    width = (bytes.width() / 300 * 72) as f32;
                    height = (bytes.height() / 300 * 72) as f32;

                    Some(image::load_from_memory(&compressed_bytes).expect("Error loading compressed image from memory"))
                },
                Err(err) => {
                    println!("Error loading image from memory: {}", err);
                    None
                }
            };

            let mut page = document.pages_mut().create_page_at_end(PdfPagePaperSize::a4()).expect("Error creating document page");
            page.objects_mut().create_image_object((PdfPagePaperSize::a4().width() - PdfPoints::new(width)) / 2.0, (PdfPagePaperSize::a4().height() - PdfPoints::new(height)) / 2.0, raw_bytes.as_ref().unwrap(), Some(PdfPoints::new(width)), Some(PdfPoints::new(height))).expect("Error adding image to page");
        }

        document.save_to_file(&path).expect("Error saving to PDF");
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
        DeleteDocument,
        CloseDocument,
        NewAttachment,
        SaveNewAttachment,
        OpenAttachment(Arc<Attachment>),
        SaveCurrentAttachment,
        CurrentAttachmentReferenceNumberChange(String),
        CurrentAttachmentCommentChange(String),
        DeleteAttachment,
        CloseAttachment,
        ShowConfirmDelete,
        SearchTextChange(String),
        OpenFileDialog,
        Back,
        KeyEvent(Key),
        Scan,
        Scanned(PathBuf),
        ScanFail,
        ScanTick,
        FinalizeScan,
        ClearImageFiles,
        ExportToPdf,
        PrevPage,
        NextPage,
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