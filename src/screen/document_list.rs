pub(crate) mod document_list {
    use std::{default, env::{current_dir, current_exe}, fs, io::{Cursor, Error, Read}, path::PathBuf, process::{Command, Output, Stdio}, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

    use base64::{Engine, prelude::BASE64_STANDARD};
    use caesium::{compress_in_memory, convert_in_memory, parameters::{CSParameters, PngParameters}};
    use file_format::FileFormat;
    use iced::{Alignment::{self, Center}, Background, Border, Color, Element, Event, Gradient, Length, Renderer, Shadow, Subscription, Task, Theme, Window, advanced::{Widget, graphics::{core::window, futures::subscription}}, alignment::Vertical::Bottom, border::Radius, gradient::{ColorStop, Linear}, keyboard::{self, Key, key}, mouse::Interaction, theme::Palette, wgpu::rwh::{self, WindowsDisplayHandle}, widget::{Container, Id, MouseArea, PickList, ProgressBar, Space, Stack, Text, TextInput, button, center, column, container::{self, Style}, image::{Handle, Viewer}, mouse_area, operation::focus_next, progress_bar, row, rule, scrollable, span, stack, text::Rich}, window::{Settings, events}};
    use iced::widget::text_input;
    use iced_aw::{Card, Spinner, TabBar, TabBarPosition, TabLabel, Tabs, card::Status, drop_down::Offset, style::{card, tab_bar}};
    use iced_dialog::dialog;
    use image::{DynamicImage, ImageBuffer};
    use log::{error, info, warn};
    use pdfium_render::prelude::{PdfBitmap, PdfBitmapFormat, PdfPageImageObject, PdfPageObjectsCommon, PdfPageOrientation, PdfPagePaperSize, PdfPageRenderRotation, PdfPoints, PdfRenderConfig, Pdfium, PdfiumError, PdfiumLibraryBindings};
    use powershell_script::PsScriptBuilder;
    use rfd::FileDialog;
    use rusqlite::ffi::SQLITE_LIMIT_FUNCTION_ARG;
    use strum::{Display, EnumIter, IntoEnumIterator};
    use tempfile::tempfile;
    use time::{Duration, OffsetDateTime, UtcDateTime, macros::format_description};
    use which::which;

    use crate::{Config, ERROR_FERRIS, LocalTheme, State, attachment::attachment::Attachment, attachment_page::attachment_page::AttachmentPage, db::db_module::DbConnection, document::document::Document};

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
        selected_file_paths: Option<Vec<PathBuf>>,
        current_file_bytes: Option<Vec<Vec<u8>>>,
        current_file_handles: Option<Vec<Handle>>,
        current_page_index: usize,
        deleted_pages: Vec<AttachmentPage>,
        file_scanned: bool,
        files_changed: bool,
        input1_id: Option<Id>,
        input2_id: Option<Id>,
        input3_id: Option<Id>,
        scanning: bool,
        exporting: bool,
        scan_progress: f32,
        current_theme: Option<LocalTheme>,
        show_confirm_delete: bool,
        show_empty_field_warning: bool,
        show_full_image_viewer: bool,
        file_hovered: bool,
        show_scan_dialog: bool,
        device_list: Vec<String>,
        selected_device: Option<Arc<String>>,
        selected_source: PaperSource,
        selected_page_size: PaperSize,
        selected_dpi: DPI,
        selected_bitdepth: Bitdepth,
        fetching_scanners: bool,
        naps2_installed: bool
    }

    impl DocumentList {
        pub(crate) fn new() -> DocumentList {
            DocumentList {
                documents: retreive_documents(),
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
                selected_file_paths: None,
                current_file_handles: None,
                current_file_bytes: None,
                current_page_index: 0,
                deleted_pages: Vec::new(),
                file_scanned: false,
                files_changed: false,
                input1_id: Some(Id::new("1")),
                input2_id: Some(Id::new("2")),
                input3_id: Some(Id::new("3")),
                scanning: false,
                exporting: false,
                scan_progress: f32::default(),
                current_theme: None,
                show_confirm_delete: false,
                show_empty_field_warning: false,
                show_full_image_viewer: false,
                file_hovered: false,
                show_scan_dialog: false,
                device_list: Vec::new(),
                selected_device: None,
                selected_source: PaperSource::default(),
                selected_page_size: PaperSize::default(),
                selected_dpi: DPI::default(),
                selected_bitdepth: Bitdepth::default(),
                fetching_scanners: false,
                naps2_installed: false,
            }
        }

        pub(crate) fn set_current_theme(&mut self, theme: LocalTheme) {
            self.current_theme = Some(theme.clone());
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
                        self.current_document_number = self.current_document_number.trim().to_string();
                        self.current_document_number = self.current_document_number.trim_end_matches(|c: char| {
                            c.is_whitespace() || c == '.'
                        }).to_string();
                        match conn.new_document(
                            self.current_document_number.clone(),
                            self.current_document_type.clone(), 
                            self.current_comment.clone()
                        ) {
                            Err(err) => {
                                error!("Error saving new Document: {}", err);
                            },
                            _ => {}
                        }

                        self.reset_state();
                        self.documents = retreive_documents();
                        self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == conn.last_rowid().unwrap() as u32).cloned();
                        self.current_document_number = self.current_open_document.as_ref().unwrap().get_document_number().to_string();
                        self.current_document_type = self.current_open_document.as_ref().unwrap().get_document_type().to_string();
                        self.current_comment = self.current_open_document.as_ref().unwrap().get_comment().to_string();
                        let file_path = format!("./data/{}", self.current_open_document.as_ref().unwrap().get_document_number());
                        match fs::create_dir(file_path) {
                            Err(err) => {
                                error!("Error creating Document's data directory: {}", err);
                            },
                            _ => {}
                        }
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
                        let old_document_number = self.current_open_document.as_ref().unwrap().get_document_number();
                        self.current_document_number = self.current_document_number.trim().to_string();
                        self.current_document_number = self.current_document_number.trim_end_matches(|c: char| {
                            c.is_whitespace() || c == '.'
                        }).to_string();
                        let old_path = format!("./data/{}", old_document_number);
                        let path = format!("./data/{}", self.current_document_number.clone());
                        match fs::rename(&old_path, &path) {
                            Err(err) => {
                                error!("Error renaming data directory: {}", err);
                                panic!("Error renaming data directory: {}", err);
                            },
                            _ => {}
                        }
                        match conn.edit_document_details(
                            current_document_id,
                            self.current_document_number.clone(),
                            self.current_document_type.clone(),
                            self.current_comment.clone()
                        ) {
                            Err(err) => {
                                println!("Error editing document: {}", err);
                                error!("Error editing document: {}", err);
                            },
                            _ => {}
                        }

                        self.reset_state();
                        self.documents = retreive_documents();
                        self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                        self.current_document_number = self.current_open_document.as_ref().unwrap().get_document_number().to_string();
                        self.current_document_type = self.current_open_document.as_ref().unwrap().get_document_type().to_string();
                        self.current_comment = self.current_open_document.as_ref().unwrap().get_comment().to_string();
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
                    self.current_document_number = input
                        .replace("<", "")
                        .replace(">", "")
                        .replace(":", "")
                        .replace("\"", "")
                        .replace("/", "")
                        .replace("\\", "")
                        .replace("|", "")
                        .replace("?", "")
                        .replace("*", "");
                    self.data_changed = true;
                    Task::none()
                },
                Message::CurrentDocumentTypeChange(input) => {
                    self.current_document_type = input
                        .replace("<", "")
                        .replace(">", "")
                        .replace(":", "")
                        .replace("\"", "")
                        .replace("/", "")
                        .replace("\\", "")
                        .replace("|", "")
                        .replace("?", "")
                        .replace("*", "");
                    self.data_changed = true;
                    Task::none()
                },
                Message::CurrentCommentChange(input) => {
                    self.current_comment = input
                        .replace("<", "")
                        .replace(">", "")
                        .replace(":", "")
                        .replace("\"", "")
                        .replace("/", "")
                        .replace("\\", "")
                        .replace("|", "")
                        .replace("?", "")
                        .replace("*", "");
                    self.data_changed = true;
                    Task::none()
                },
                Message::NewAttachment => {
                    self.reset_attachment_state();
                    self.create_new_attachment = true;
                    self.current_file_bytes = Some(Vec::new());
                    self.update_file_handles();
                    Task::none()
                },
                Message::OpenFileDialog => {
                    let previous_file_paths = self.selected_file_paths.clone();
                    self.selected_file_paths = FileDialog::new().set_title("Select Document")
                        .add_filter("Image/PDF (.png, .jpg, .jpeg, .webp)", &["png", "jpg", "jpeg", "webp", "pdf"])
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
                                    error!("Error reading files from paths: {}", err);
                                }
                            }
                        }

                        for bytes in selected_file_bytes {
                            self.add_file_bytes(bytes);
                        }

                        self.update_file_handles();
                    }
                    Task::none()
                },
                Message::FileHover => {
                    if self.create_new_attachment || self.current_open_attachment.is_some() {
                        self.file_hovered = true;
                        println!("File Hovered");
                    }
                    Task::none()
                },
                Message::FileDrop(path) => {
                    if self.create_new_attachment || self.current_open_attachment.is_some() {
                        match fs::read(&path) {
                            Ok(bytes) => {
                                match file_format::FileFormat::from_bytes(&bytes) {
                                    FileFormat::PortableDocumentFormat => {
                                        self.files_changed = true;
                                        for image in pdf_to_png(bytes) {
                                            self.add_file_bytes(image);
                                        }
                                        self.update_file_handles();
                                    },
                                    format => {
                                        if format.kind() == file_format::Kind::Image {
                                            self.files_changed = true;
                                            self.add_file_bytes(bytes);
                                            self.update_file_handles();
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Error reading files from path: {}", err);
                            }
                        }
                    }
                    println!("File Dropped");
                    
                    Task::none()
                },
                Message::FileHoverLeft => {
                    if self.create_new_attachment || self.current_open_attachment.is_some() {
                        self.file_hovered = false;
                    }
                    println!("File left");
                    Task::none()
                }
                Message::SaveNewAttachment => {
                    if self.current_attachment_reference_number.is_empty() || self.current_file_bytes.is_none() {
                        self.show_empty_field_warning = true;
                    }
                    else {
                        self.current_attachment_reference_number = self.current_attachment_reference_number.trim().to_string();
                        self.current_attachment_reference_number = self.current_attachment_reference_number.trim_end_matches(|c: char| {
                            c.is_whitespace() || c == '.'
                        }).to_string();
                        let file_path = format!("./data/{}/{}", self.current_open_document.as_ref().unwrap().get_document_number(), self.current_attachment_reference_number);
                        match fs::create_dir(&file_path) {
                            Err(err) => {
                                error!("Error creating attachment's data folder: {}", err);
                            }
                            _ => {}
                        }
                        let mut conn = DbConnection::new();
                        let current_document_id = self.current_open_document.clone().unwrap().get_document_id();
                        let current_document_number = &self.current_document_number;

                        let mut data_file_paths: Vec<PathBuf> = Vec::new();
                        for (index, _) in self.current_file_handles.as_ref().unwrap().iter().enumerate() {
                            data_file_paths.push(format!("{}/{}_{}_{}.png", &file_path, current_document_number, self.current_attachment_reference_number, index + 1).into())
                        }
                        
                        match conn.new_attachment(data_file_paths, self.current_attachment_reference_number.clone(), self.current_attachment_comment.clone(), current_document_id) {
                            Err(err) => {
                                error!("Error creating new attachment: {}", err);
                            },
                            _ => {}
                        };

                        for (index, bytes) in self.current_file_bytes.as_mut().unwrap().iter_mut().enumerate() {
                            let file_name = format!("{}_{}_{}.png", current_document_number, self.current_attachment_reference_number, index + 1);
                            let file_path = format!("./data/{}/{}/{}", current_document_number, self.current_attachment_reference_number, file_name);
                            if FileFormat::from_bytes(&bytes) != FileFormat::PortableNetworkGraphics {
                                let img = image::load_from_memory(&bytes);
                                match img.unwrap().write_to(&mut Cursor::new(&mut *bytes), image::ImageFormat::Png) {
                                    Err(err) => error!("Error converting image format: {}", err),
                                    _ => {}
                                }
                            }

                            let compressed_bytes = compress_image(bytes.to_vec());

                            match fs::write(&file_path, compressed_bytes) {
                                Err(err) => {
                                    error!("Error writing file to data folder: {}", err);
                                },
                                _ => {}
                            };
                        }

                        self.reset_attachment_state();
                        self.documents = retreive_documents();
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

                        self.current_attachment_reference_number = self.current_attachment_reference_number.trim().to_string();
                        self.current_attachment_reference_number = self.current_attachment_reference_number.trim_end_matches(|c: char| {
                            c.is_whitespace() || c == '.'
                        }).to_string();
                        
                        match conn.edit_attachment_details(
                            current_attachment_id,
                            self.current_attachment_reference_number.clone(),
                            self.current_attachment_comment.clone()
                        ) {
                            Err(err) => {
                                error!("Error editing attachment: {}", err);
                            },
                            _ => {}
                        };

                        if self.files_changed {
                            if !self.deleted_pages.is_empty() {
                                for page in &self.deleted_pages {
                                    if !fs::exists("./restored").unwrap() {
                                        match fs::create_dir("./restored") {
                                            Err(err) => {
                                                error!("Error creating restored files directory: {}", err);
                                            },
                                            _ => {}
                                        }
                                    }
                                    let restored_path = format!("./restored/{}_{}_{}.png", self.current_open_document.as_ref().unwrap().get_document_number(), self.current_open_attachment.as_ref().unwrap().get_reference_number(), page.page_id());
                                    match fs::rename(page.file_path().to_string(), &restored_path) {
                                        Err(err) => {
                                            error!("Error moving file to restored path: {}", err);
                                        },
                                        _ => {}
                                    }
                                    match trash::delete(&restored_path) {
                                        Err(err) => {
                                            error!("Error moving file to recycle bin: {}", err);
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            let path = format!("./data/{}/{}", current_document_number, self.current_open_attachment.as_ref().unwrap().get_reference_number());
                            match fs::remove_dir_all(&path) {
                                Err(err) => {
                                    error!("Error deleting old directory: {}", err);
                                    panic!("Error deleting old directory: {}", err);
                                },
                                _ => {}
                            }
                            let mut file_paths: Vec<PathBuf> = Vec::new();
                            let file_dir = format!("./data/{}/{}", current_document_number, self.current_attachment_reference_number);
                            match fs::create_dir(&file_dir) {
                                Err(err) => {
                                    error!("Error creating new directory: {}", err);
                                    panic!("Error creating new directory: {}", err);
                                },
                                _ => {}
                            }

                            for (index, bytes) in self.current_file_bytes.as_mut().unwrap().iter_mut().enumerate() {
                                let file_name = format!("{}_{}_{}.png", current_document_number, self.current_attachment_reference_number, index + 1);
                                let file_path = format!("{}/{}", file_dir, file_name);
                                
                                if FileFormat::from_bytes(&bytes) != FileFormat::PortableNetworkGraphics {
                                    let img = image::load_from_memory(&bytes);
                                    match img.unwrap().write_to(&mut Cursor::new(&mut *bytes), image::ImageFormat::Png) {
                                        Err(err) => error!("Error converting image format: {}", err),
                                        _ => {}
                                    }
                                }

                                let compressed_bytes = compress_image(bytes.to_vec());

                                match fs::write(&file_path, compressed_bytes) {
                                    Err(err) => {
                                        error!("Error writing file to data folder: {}", err);
                                    },
                                    _ => {}
                                };

                                file_paths.push(file_path.into());
                            }

                            match conn.edit_attachment_pages(self.current_open_attachment.as_ref().unwrap().get_attachment_id(), file_paths) {
                                Err(err) => {
                                    error!("Error editing attachment pages: {}", err);
                                    panic!("Error editing attachment pages: {}", err);
                                },
                                _ => {}
                            }

                        }
                        else {
                            let mut new_file_paths: Vec<PathBuf> = Vec::new();
                            let old_file_dir = format!("./data/{}/{}", current_document_number, old_attachment_reference_number);
                            let new_file_dir = format!("./data/{}/{}", current_document_number, self.current_attachment_reference_number);
                            
                            for (index, path) in old_file_paths.into_iter().enumerate() {
                                let file_name = format!("{}_{}_{}.png", current_document_number, self.current_attachment_reference_number, index + 1);
                                let old_file_path = format!("{}/{}", old_file_dir, file_name);

                                match fs::rename(path, &old_file_path) {
                                    Err(err) => {
                                        error!("Error renaming file: {}", err);
                                        panic!("Error renaming file: {}", err);
                                    },
                                    _ => {}
                                }
                                let new_file_path = format!("{}/{}", new_file_dir, file_name);
                                new_file_paths.push(new_file_path.into());
                            }
                            match fs::rename(old_file_dir, &new_file_dir) {
                                Err(err) => {
                                    error!("Error renaming directory: {}", err);
                                    panic!("Error renaming directory: {}", err);
                                },
                                _ => {}
                            }

                            match conn.edit_attachment_pages(self.current_open_attachment.as_ref().unwrap().get_attachment_id(), new_file_paths) {
                                Err(err) => {
                                    error!("Error editing attachment pages: {}", err);
                                    panic!("Error editing attachment pages: {}", err);
                                },
                                _ => {}
                            }
                        }

                        self.reset_attachment_state();
                        self.documents = retreive_documents();
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
                    self.current_attachment_reference_number = input
                        .replace("<", "")
                        .replace(">", "")
                        .replace(":", "")
                        .replace("\"", "")
                        .replace("/", "")
                        .replace("\\", "")
                        .replace("|", "")
                        .replace("?", "")
                        .replace("*", "");
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
                Message::ShowScanDialog => {
                    if cfg!(target_os = "linux") {
                        self.naps2_installed = which("naps2").is_ok();
                    }
                    else if cfg!(target_os = "windows") {
                        self.naps2_installed = match fs::exists("./naps2/NAPS2.Console.exe") {
                            Ok(bool) => bool,
                            Err(err) => {
                                error!("Error checking for NAPS2: {}", err);
                                false
                            }
                        };
                    }

                    if self.naps2_installed {
                        self.show_scan_dialog = true;
                        if self.device_list.is_empty() {
                            self.fetching_scanners = true;
                            Task::perform(fetch_scanners(), |result| match result {
                                Ok(list) => {
                                    Message::ScannersFound(list)
                                },
                                Err(err) => {
                                    error!("Error fetching scanners: {}", err);
                                    Message::ScannerFetchFail
                                }
                            })
                        }
                        else {
                            Task::none()
                        }
                    }
                    else {
                        Task::none()
                    }
                },
                Message::ScannersFound(list) => {
                    self.device_list.clear();
                    for device in list {
                        self.device_list.push(device);
                    }
                    if self.device_list.len() > 0 {
                        self.selected_device = Some(Arc::new(self.device_list[0].clone()));
                    }
                    self.fetching_scanners = false;
                    Task::none()
                },
                Message::ScannerFetchFail => {
                    self.fetching_scanners = false;
                    Task::none()
                },
                Message::RefreshDeviceList => {
                    self.fetching_scanners = true;
                    Task::perform(fetch_scanners(), |result| match result {
                        Ok(list) => {
                            Message::ScannersFound(list)
                        },
                        Err(err) => {
                            error!("Error fetching scanners: {}", err);
                            Message::ScannerFetchFail
                        }
                    })
                },
                Message::SelectDevice(device) => {
                    self.selected_device = Some(Arc::new(device));
                    Task::none()
                },
                Message::SelectSource(source) => {
                    self.selected_source = source;
                    Task::none()
                },
                Message::SelectPageSize(page_size) => {
                    self.selected_page_size = page_size;
                    Task::none()
                },
                Message::SelectDPI(dpi) => {
                    self.selected_dpi = dpi;
                    Task::none()
                },
                Message::SelectBitdepth(bitdepth) => {
                    self.selected_bitdepth = bitdepth;
                    Task::none()
                },
                Message::CloseScanDialog => {
                    self.show_scan_dialog = false;
                    Task::none()
                },
                Message::Scan => {
                    self.scanning = true;
                    self.show_scan_dialog = false;
                    self.scan_progress = 0.3;
                    let selected_device = self.selected_device.as_ref().unwrap().clone().to_string();
                    let selected_source = self.selected_source.to_naps2_arg();
                    let selected_page_size = self.selected_page_size.to_naps2_arg();
                    let selected_dpi = self.selected_dpi.to_string();
                    let selected_bitdepth = self.selected_bitdepth.to_naps2_arg();
                    Task::perform(
                        run_scan(selected_device, selected_source, selected_page_size, selected_dpi, selected_bitdepth),
                        move |result| {
                            match result {
                                Ok(path) => {
                                    Message::Scanned(path)
                                },
                                Err(err) => {
                                    error!("Error scanning: {}", err);
                                    Message::ScanFail
                                }
                            }
                        }
                    )
                },
                Message::Scanned(path) => {
                    self.scanning = false;
                    self.scan_progress = 1.0;
                    match fs::read(&path) {
                        Ok(bytes) => {
                            self.file_scanned = true;
                            self.files_changed = true;
                            self.data_changed = true;
                            for image in pdf_to_png(bytes) {
                                self.add_file_bytes(image);
                            }
                            self.update_file_handles();

                            // let mut converted_bytes: Vec<u8> = Vec::new();
                            // match image::load_from_memory(&bytes) {
                            //     Ok(img) => {
                            //         img.write_to(&mut Cursor::new(&mut converted_bytes), image::ImageFormat::Png)
                            //             .unwrap_or_else(|err| error!("Error converting scanned bytes: {}", err));
                            //     }
                            //     Err(err) => {
                            //         error!("Error loading scanned bytes from memory: {}", err);
                            //     }
                            // }
                            // self.add_file_bytes(converted_bytes);
                        }
                        Err(err) => {
                            error!("Error reading temp file paths: {}", err);
                        }
                        
                    }
                    match fs::remove_file(&path) {
                        Err(err) => error!("Error removing temp file: {}", err),
                        _ => {}
                    }
                    Task::none()
                },
                Message::ScanFail => {
                    self.scanning = false;
                    self.scan_progress = 0.0;
                    error!("Scan failed or was cancelled.");
                    Task::none()
                },
                Message::ScanTick => {
                    if self.scanning {
                        if self.scan_progress < 0.9 {
                            self.scan_progress += 0.01;
                        }
                    }
                    Task::none()
                },
                Message::RotateImage => {
                    let current_file_bytes = self.current_file_bytes.as_ref().unwrap()[self.current_page_index].clone();
                    let current_image = match image::load_from_memory(&current_file_bytes) {
                        Ok(image) => image,
                        Err(err) => {
                            error!("Error loading current selected image: {}", err);
                            panic!("Error loading current selected image: {}", err);
                        }
                    };
                    let rotated_image = current_image.rotate90();
                    let mut rotated_bytes: Vec<u8> = Vec::new();
                    match rotated_image.write_to(&mut Cursor::new(&mut rotated_bytes), image::ImageFormat::Png) {
                        Err(err) => {
                            error!("Error rotating image: {}", err);
                        },
                        _ => {}
                    };
                    self.current_file_bytes.as_mut().unwrap()[self.current_page_index] = rotated_bytes;
                    self.update_file_handles();
                    self.files_changed = true;
                    Task::none()
                },
                Message::RemoveImage(index) => {
                    if self.current_open_attachment.is_some() {
                        self.deleted_pages.push(self.current_open_attachment.as_ref().unwrap().page(self.current_page_index).clone());
                    }
                    
                    if self.current_file_bytes.as_ref().unwrap().len() > 1 {
                        if self.current_page_index > 0 {
                            self.current_page_index = self.current_page_index - 1;
                        }
                        self.current_file_bytes.as_mut().unwrap().remove(index);
                        self.update_file_handles();
                        self.files_changed = true;
                    }
                    Task::none()
                },
                Message::OpenFullImageViewer => {
                    self.show_full_image_viewer = true;
                    Task::none()
                },
                Message::CloseFullImageViewer => {
                    self.show_full_image_viewer = false;
                    Task::none()
                },
                Message::ClearImageFiles => {
                    self.current_file_bytes = None;
                    self.update_file_handles();
                    Task::none()
                },
                Message::DeleteDocument => {
                    let mut conn = DbConnection::new();
                    match conn.set_document_deleted(self.current_open_document.as_ref().unwrap().get_document_id()) {
                        Err(err) => {
                            error!("Error deleting document: {}", err);
                            panic!("Error deleting document: {}", err);
                        },
                        _ => {}
                    }

                    self.documents = retreive_documents();
                    self.reset_state();
                    
                    Task::none()
                },
                Message::DeleteAttachment => {
                    let mut conn = DbConnection::new();
                    match conn.set_attachment_deleted(self.current_open_attachment.as_ref().unwrap().get_attachment_id()) {
                        Ok(_) => {},
                        Err(err) => error!("Error deleting attachment: {}", err)
                    }

                    let current_document_id = self.current_open_document.as_ref().unwrap().get_document_id();
                    self.documents = retreive_documents();
                    self.current_open_document = self.documents.iter().find(|document| document.get_document_id() == current_document_id).cloned();
                    self.reset_attachment_state();
                    
                    Task::none()
                },
                Message::ShowConfirmDelete => {
                    self.show_confirm_delete = true;
                    Task::none()
                },
                Message::ExportToPdf => {
                    self.exporting = true;
                    let current_open_document = self.current_open_document.as_ref().unwrap().clone();
                    let current_open_attachment = self.current_open_attachment.as_ref().unwrap().clone();
                    let current_file_bytes = self.current_file_bytes.as_ref().unwrap().clone();
                    Task::future(
                        async move {
                            let file_name = format!("{}_{}.pdf", current_open_document.get_document_number(), current_open_attachment.get_reference_number());
                            let path = format!("./data/{}/{}/{}", current_open_document.get_document_number(), current_open_attachment.get_reference_number(), file_name);
                            match export_to_pdf(current_file_bytes, path.clone().into()) {
                                Ok(_) => {
                                    Message::ExportSuccess(path.clone().into())
                                },
                                Err(err) => {
                                    error!("Error creating PDF file: {}", err);
                                    Message::ExportFail
                                }
                            }
                        }
                    )
                },
                Message::ExportSuccess(path) => {
                    self.exporting = false;
                    reveal_file(&path);
                    Task::none()
                },
                Message::ExportFail => {
                    self.exporting = false;
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
                },
                Message::OpenLink(link) => {
                    match opener::open(link) {
                        Err(err) => {
                            error!("Error opening link: {}", err);
                        },
                        _ => {}
                    };
                    Task::none()
                }
            }
        }


        pub(crate) fn view(&self) -> Element<'_, Message> {
            //let mut document_cards: Vec<MouseArea<'static, Message>> = Vec::new();
            let mut document_cards: Vec<DataCard> = Vec::new();

            for document in &self.documents {
                document_cards.push(DataCard::new(Some(document.clone()), None, self.current_theme.clone().unwrap()));
            }

            let main_content = match &self.current_open_document {
                None => {
                    match self.create_new_document {
                        // New Document Screen
                        true => {
                            Container::new(
                                column![
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
                                    ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
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
                                ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
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
                                        
                                    ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill),
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
                                                let mut content: Stack<Message> = Stack::new();
                                                content = content.push(
                                                    Container::new(column![
                                                        Container::new(row![
                                                            if self.scanning {
                                                                button("<")
                                                            }
                                                            else {
                                                                button("<").on_press(Message::CloseAttachment)
                                                            },
                                                            if self.scanning {
                                                                button("Save")
                                                            }
                                                            else {
                                                                button("Save").on_press(Message::SaveNewAttachment)
                                                            }
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
                                                                        if self.scanning {
                                                                            row![
                                                                                button("Select"),
                                                                                button(Spinner::new())
                                                                            ].spacing(5)
                                                                        }
                                                                        else {
                                                                            row![
                                                                                button("Select").on_press(Message::OpenFileDialog),
                                                                                button("Scan").on_press(Message::ShowScanDialog)
                                                                            ].spacing(5)
                                                                        }
                                                                    ].spacing(5).width(Length::FillPortion(4)),
                                                                    if self.scanning {
                                                                        button(Text::new("Clear").align_x(Center).width(Length::Fill)).width(Length::Fill)
                                                                    }
                                                                    else {
                                                                        button(Text::new("Clear").align_x(Center).width(Length::Fill)).on_press(Message::ClearImageFiles).width(Length::Fill)
                                                                    }
                                                                ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)),
                                                                rule::vertical(2),
                                                                Container::new(
                                                                    column![
                                                                        if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                            Container::new(Space::new().width(Length::Fill).height(Length::Fill))
                                                                        }
                                                                        else {
                                                                            Container::new(Viewer::new(self.current_file_handles.as_ref().unwrap()[self.current_page_index].clone()).width(Length::Fill).height(Length::Fill))
                                                                        },
                                                                        rule::horizontal(2),
                                                                        row![
                                                                            Space::new().width(Length::FillPortion(1)),
                                                                            Container::new(
                                                                                row![
                                                                                    if self.current_page_index > 0 {
                                                                                        button("<").on_press(Message::PrevPage)
                                                                                    }
                                                                                    else {
                                                                                        button("<")
                                                                                    },
                                                                                    Text::new(self.current_page_index + 1).center(),
                                                                                    if self.current_page_index + 1 < self.current_file_handles.as_ref().unwrap().len() {
                                                                                        button(">").on_press(Message::NextPage)
                                                                                    }
                                                                                    else {
                                                                                        button(">")
                                                                                    }
                                                                                ].spacing(10).align_y(Center)
                                                                            ).width(Length::FillPortion(1)).align_x(Center),
                                                                            Container::new(
                                                                                row![
                                                                                    if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                                        button("↷")
                                                                                    }
                                                                                    else {
                                                                                        button("↷").on_press(Message::RotateImage)
                                                                                        
                                                                                    },
                                                                                    if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                                        button("🗙")
                                                                                    }
                                                                                    else {
                                                                                        button("🗙").on_press(Message::RemoveImage(self.current_page_index))
                                                                                        
                                                                                    },
                                                                                    if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                                        button("⛶")
                                                                                    }
                                                                                    else {
                                                                                        button("⛶").on_press(Message::OpenFullImageViewer)
                                                                                    },
                                                                                    
                                                                                ].spacing(5).align_y(Center)
                                                                            ).width(Length::FillPortion(1)).align_x(Alignment::End)
                                                                        ].width(Length::Fill).align_y(Center)
                                                                    ].spacing(5).align_x(Center).width(Length::Fill).height(Length::Fill)
                                                                ).padding(5).style(container::bordered_box).width(Length::FillPortion(3)).height(Length::Fill)
                                                            ].spacing(5),
                                                        ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                                    ].spacing(5)).width(Length::Fill).height(Length::Fill)
                                                );
                                                if self.show_scan_dialog {
                                                    content = content.extend(
                                                        vec![
                                                            MouseArea::new(Space::new().width(Length::Fill).height(Length::Fill)).interaction(Interaction::Idle).into(),
                                                            Container::new(
                                                                center(scan_dialog(self.device_list.clone(), self.selected_device.clone(), self.fetching_scanners.clone(), self.selected_source.clone(), self.selected_page_size.clone(), self.selected_dpi.clone(), self.selected_bitdepth.clone()))
                                                            ).width(Length::Fill).height(Length::Fill).style(|_theme| container::Style {
                                                                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.4).into()), // dims the content behind scan dialog
                                                                border: Border {
                                                                    radius: 5.0.into(),
                                                                    ..Default::default()
                                                                }, 
                                                                ..Default::default()
                                                            }).into()
                                                        ]
                                                    );
                                                }
                                                if self.naps2_installed == false {
                                                    content = content.extend(
                                                        vec![
                                                            MouseArea::new(Space::new().width(Length::Fill).height(Length::Fill)).interaction(Interaction::Idle).into(),
                                                            Container::new(
                                                                center(naps2_install_dialog())
                                                            ).width(Length::Fill).height(Length::Fill).style(|_theme| container::Style {
                                                                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.4).into()), // dims the content behind scan dialog
                                                                border: Border {
                                                                    radius: 5.0.into(),
                                                                    ..Default::default()
                                                                }, 
                                                                ..Default::default()
                                                            }).into()
                                                        ]
                                                    );
                                                }
                                                Container::new(content)
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
                                                ].spacing(5)
                                                ).width(Length::Fill).height(Length::Fill)
                                            }
                                        }
                                    },
                                    // Attachment Details Screen
                                    Some(attachment) => {
                                        let mut content: Stack<Message> = Stack::new();
                                        content = content.push(
                                            Container::new(column![
                                                Container::new(row![
                                                    if self.scanning || self.exporting {
                                                        button("<")
                                                    }
                                                    else {
                                                        button("<").on_press(Message::CloseAttachment)
                                                    }
                                                    ,
                                                    if self.data_changed || self.files_changed || !self.scanning {
                                                        button("Save").on_press(Message::SaveCurrentAttachment)
                                                    }
                                                    else {
                                                        button("Save")
                                                    },
                                                    if self.scanning || self.exporting {
                                                        button("New")
                                                    }
                                                    else {
                                                        button("New").on_press(Message::NewAttachment)
                                                    },
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
                                                                    if self.show_empty_field_warning && self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                        text_input("", &self.current_file_handles.as_ref().unwrap().len().to_string().as_str()).style(|theme, _| empty_text_input_warning(theme))
                                                                    }
                                                                    else {
                                                                        text_input("", &self.current_file_handles.as_ref().unwrap().len().to_string().as_str())
                                                                    },
                                                                    if self.scanning {
                                                                        row![
                                                                            button("Select"),
                                                                            button(Spinner::new())
                                                                        ].spacing(5)
                                                                    }
                                                                    else if self.exporting {
                                                                        row![
                                                                            button("Select"),
                                                                            button("Scan")
                                                                        ].spacing(5)
                                                                    }
                                                                    else {
                                                                        row![
                                                                            button("Select").on_press(Message::OpenFileDialog),
                                                                            button("Scan").on_press(Message::ShowScanDialog),
                                                                        ].spacing(5)
                                                                    }
                                                                ].spacing(5).width(Length::Fill),
                                                                if self.scanning {
                                                                    row![
                                                                        button(Text::new("Export").center()).width(Length::FillPortion(1)),
                                                                        button(Text::new("Clear").center()).width(Length::FillPortion(1))
                                                                    ].spacing(5)
                                                                }
                                                                else if self.exporting {
                                                                    row![
                                                                        button(Spinner::new()).width(Length::FillPortion(1)),
                                                                        button(Text::new("Clear").center()).width(Length::FillPortion(1))
                                                                    ].spacing(5)
                                                                }
                                                                else {
                                                                    row![
                                                                        button(Text::new("Export").center()).on_press(Message::ExportToPdf).width(Length::FillPortion(1)),
                                                                        button(Text::new("Clear").center()).on_press(Message::ClearImageFiles).width(Length::FillPortion(1))
                                                                    ].spacing(5)
                                                                }
                                                            ].spacing(5),
                                                            //ProgressBar::new(0.0..=1.0, self.scan_progress)
                                                        ].spacing(5)).padding(5).style(container::bordered_box).width(Length::FillPortion(1)).height(Length::Fill),
                                                        rule::vertical(2),
                                                        Container::new(
                                                            column![
                                                                if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                    Container::new(Space::new().width(Length::Fill).height(Length::Fill))
                                                                }
                                                                else {
                                                                    Container::new(Viewer::new(self.current_file_handles.as_ref().unwrap()[self.current_page_index].clone()).width(Length::Fill).height(Length::Fill))
                                                                },
                                                                rule::horizontal(2),
                                                                row![
                                                                    Space::new().width(Length::FillPortion(1)),
                                                                    Container::new(
                                                                        row![
                                                                            if self.current_page_index > 0 {
                                                                                button("<").on_press(Message::PrevPage)
                                                                            }
                                                                            else {
                                                                                button("<")
                                                                            },
                                                                            Text::new(self.current_page_index + 1).center(),
                                                                            if self.current_page_index + 1 < self.current_file_handles.as_ref().unwrap().len() {
                                                                                button(">").on_press(Message::NextPage)
                                                                            }
                                                                            else {
                                                                                button(">")
                                                                            }
                                                                        ].spacing(10).align_y(Center)
                                                                    ).width(Length::FillPortion(1)).align_x(Center),
                                                                    Container::new(
                                                                        if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                                                            
                                                                            row![
                                                                                button("↷"),
                                                                                button("🗙"),
                                                                                button("⛶")
                                                                            ].spacing(5).align_y(Center)
                                                                        }
                                                                        else {
                                                                            row![
                                                                                button("↷").on_press(Message::RotateImage),
                                                                                button("🗙").on_press(Message::RemoveImage(self.current_page_index)),
                                                                                button("⛶").on_press(Message::OpenFullImageViewer)
                                                                            ].spacing(5).align_y(Center)
                                                                        }
                                                                    ).width(Length::FillPortion(1)).align_x(Alignment::End)
                                                                ].width(Length::Fill).align_y(Center)
                                                            ].spacing(5).align_x(Center)
                                                        ).padding(5).style(container::bordered_box).width(Length::FillPortion(3)).height(Length::Fill)
                                                    ].spacing(5),
                                                ].spacing(5)).padding(10).style(container::bordered_box).width(Length::Fill).height(Length::Fill)
                                            ].spacing(5)).height(Length::Fill).width(Length::Fill)
                                        );
                                        if self.show_scan_dialog {
                                            content = content.extend(
                                                vec![
                                                    MouseArea::new(Space::new().width(Length::Fill).height(Length::Fill)).interaction(Interaction::Idle).into(),
                                                    Container::new(
                                                        center(scan_dialog(self.device_list.clone(), self.selected_device.clone(), self.fetching_scanners.clone(), self.selected_source.clone(), self.selected_page_size.clone(), self.selected_dpi.clone(), self.selected_bitdepth.clone()))
                                                    ).width(Length::Fill).height(Length::Fill).style(|_theme| container::Style {
                                                        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.4).into()), // dims the content behind scan dialog
                                                        border: Border {
                                                            radius: 5.0.into(),
                                                            ..Default::default()
                                                        }, 
                                                        ..Default::default()
                                                    }).into()
                                                ]
                                            );
                                        }
                                        Container::new(content)
                                    }
                                }
                            },
                        },
                        tab_bar(self.current_document_tab.clone())
                    ].spacing(5)).into()
                }
            };

            //let image_viewer: Element<'_, Message, Theme, Renderer> = ;

            if self.show_full_image_viewer {
                Container::new(
                    stack![
                        column![
                            Viewer::new(self.current_file_handles.as_ref().unwrap()[self.current_page_index].clone()).width(Length::Fill).height(Length::Fill),
                            row![
                                Space::new().width(Length::FillPortion(1)),
                                Container::new(
                                    row![
                                        if self.current_page_index > 0 {
                                            button("<").on_press(Message::PrevPage)
                                        }
                                        else {
                                            button("<")
                                        },
                                        Text::new(self.current_page_index + 1).center(),
                                        if self.current_page_index + 1 < self.current_file_handles.as_ref().unwrap().len() {
                                            button(">").on_press(Message::NextPage)
                                        }
                                        else {
                                            button(">")
                                        }
                                    ].spacing(10).align_y(Center)
                                ).width(Length::FillPortion(1)).align_x(Center),
                                Container::new(
                                    if self.current_file_handles.is_none() || self.current_file_handles.as_ref().unwrap().len() == 0 {
                                        
                                        row![
                                            button("↷"),
                                            button("🗙"),
                                            button("⛶")
                                        ].spacing(5).align_y(Center)
                                    }
                                    else {
                                        row![
                                            button("↷").on_press(Message::RotateImage),
                                            button("🗙").on_press(Message::RemoveImage(self.current_page_index)),
                                            button("⛶").on_press(Message::CloseFullImageViewer)
                                        ].spacing(5).align_y(Center)
                                    }
                                ).width(Length::FillPortion(1)).align_x(Alignment::End)
                            ].width(Length::Fill).align_y(Center),
                        ].spacing(5),
                        Container::new(
                            button(Text::new("←").center()).on_press(Message::CloseFullImageViewer)
                        ).width(Length::Fill).align_x(Alignment::End)
                    ].width(Length::Fill).height(Length::Fill),
                ).padding(5).style(container::bordered_box).into()
            }
            else {
                main_content
            }
        }

        pub(crate) fn subscription(&self) -> Subscription<Message> {
            let kb_event = iced::event::listen_with(|event, _, _| {
                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key, ..}) => {
                        Some(Message::KeyEvent(key))
                    }
                    _ => None
                }
            });

            let file_drop_event = iced::event::listen_with(|event, _status, _| {
                match event {
                    Event::Window(iced::window::Event::FileDropped(path)) => {
                        return Some(Message::FileDrop(path))
                    },
                    Event::Window(iced::window::Event::FileHovered(_)) => {
                        return Some(Message::FileHover)
                    },
                    Event::Window(iced::window::Event::FilesHoveredLeft) => {
                        return Some(Message::FileHoverLeft)
                    }
                    _ => { None }
                }
            });
            
            Subscription::batch(vec![kb_event, file_drop_event])
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

        pub(crate) fn reset_state(&mut self) {
            self.current_open_document = None;
            self.current_document_number.clear();
            self.current_document_type.clear();
            self.current_comment.clear();
            self.create_new_document = false;
            self.current_document_tab = Tab::Details;
            self.reset_attachment_state();
        }

        pub(crate) fn reset_attachment_state(&mut self) {
            self.current_open_attachment = None;
            self.current_attachment_reference_number.clear();
            self.current_attachment_comment.clear();
            self.current_file_bytes = None;
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
            self.selected_file_paths = None;
            self.deleted_pages.clear();
        }

        pub(crate) fn refresh_data(&mut self) {
            self.documents = retreive_documents();
        }
    }

    fn tab_bar(selected_tab: Tab) -> Element<'static, Message> {
        Container::new(
            row![
                button(Text::new("Details").center().size(20).height(Length::Fill)).on_press(Message::SwitchTab(Tab::Details)).style(move |theme, status| 
                    if selected_tab == Tab::Details {
                        tab_bar_button_selected_style(theme)
                    }
                    else {
                        tab_bar_button_style(theme, status)
                    }
                ).width(Length::FillPortion(1)).height(Length::Fill),
                button(Text::new("Attachments").center().size(20).height(Length::Fill)).on_press(Message::SwitchTab(Tab::Attachments)).style(move |theme, status|
                    if selected_tab == Tab::Attachments {
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

    fn retreive_documents() -> Vec<Arc<Document>> {
        match DbConnection::new().read_document_table() {
            Ok(documents) => documents,
            Err(err) => {
                error!("Error retreiving documents: {}", err);
                panic!("Error retreiving documents: {}", err);
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

    fn scan_dialog(device_list: Vec<String>, selected_device: Option<Arc<String>>, fetching_scanners: bool, selected_source: PaperSource, selected_page_size: PaperSize, selected_dpi: DPI, selected_bitdepth: Bitdepth) -> Container<'static, Message> {
        Container::new(
            column![
                row![
                    Text::new("Scan").size(20).align_y(Center).height(Length::Fill),
                    Space::new().width(Length::Fill),
                    button("🗙").on_press(Message::CloseScanDialog)
                ].height(Length::Shrink),
                rule::horizontal(2),
                row![
                    Text::new("Device:").width(80),
                    PickList::new(device_list, selected_device.clone(), Message::SelectDevice),
                    if fetching_scanners {
                        Container::new(
                            row![
                                Text::new("Fetching..."),
                                Spinner::new()
                            ].spacing(5).align_y(Center)
                        )
                    }
                    else {
                        Container::new(
                            button(Text::new("🗘").center().height(Length::Fill)).on_press(Message::RefreshDeviceList).height(Length::Shrink)
                        )
                    }
                ].spacing(5).align_y(Center),
                row![
                    Text::new("Source:").width(80),
                    PickList::new(PaperSource::iter().collect::<Vec<_>>(), Some(selected_source), Message::SelectSource)
                ].spacing(5).align_y(Center),
                row![
                    Text::new("Page Size:").width(80),
                    PickList::new(PaperSize::iter().collect::<Vec<_>>(), Some(selected_page_size), Message::SelectPageSize)
                ].spacing(5).align_y(Center),
                row![
                    Text::new("DPI:").width(80),
                    PickList::new(DPI::iter().collect::<Vec<_>>(), Some(selected_dpi), Message::SelectDPI)
                ].spacing(5).align_y(Center),
                row![
                    Text::new("Bit depth:").width(80),
                    PickList::new(Bitdepth::iter().collect::<Vec<_>>(), Some(selected_bitdepth), Message::SelectBitdepth)
                ].spacing(5).align_y(Center),
                Space::new().height(Length::Fill),
                if selected_device.clone().is_some() {
                    center(button("Scan").on_press(Message::Scan)).width(Length::Fill).height(Length::Shrink)
                }
                else {
                    center(button("Scan")).width(Length::Fill).height(Length::Shrink)
                }
            ].spacing(5)
        ).padding(5).style(container::bordered_box).width(Length::Fixed(800.0)).height(Length::Fixed(400.0))
    }

    fn naps2_install_dialog() -> Container<'static, Message> {
        Container::new(
            column![
                Text::new("NAPS2 is not installed"),
                rule::horizontal(2),
                Rich::with_spans(
                    vec![
                        span("NAPS2 is required to use the scanning feature.\n"),
                        span("Download:\n"),
                        span("Official site (naps2.com)\n").color(Color::from_rgb(0.2, 0.5, 1.0)).link("http://www.naps2.com"),
                        span("or from your distro's package manager.")
                    ]
                ).on_link_click(|link: String| Message::OpenLink(link))
            ]
        ).padding(5).style(container::bordered_box).width(600).height(400)
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
    enum PaperSource {
        #[strum(serialize = "Glass")]
        #[default]
        Glass,
        #[strum(serialize = "Feeder")]
        Feeder,
        #[strum(serialize = "Duplex")]
        Duplex
    }

    impl PaperSource {
        fn to_naps2_arg(&self) -> String {
            match self {
                PaperSource::Glass => String::from("glass"),
                PaperSource::Feeder => String::from("feeder"),
                PaperSource::Duplex => String::from("duplex")
            }
        }
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
    enum PaperSize {
        #[strum(serialize = "A4 (210 x 297 mm)")]
        #[default]
        A4,
        #[strum(serialize = "Letter (8.5 x 11 in)")]
        Letter,
        #[strum(serialize = "Long (8.5 x 13 in)")]
        Long,
        #[strum(serialize = "Legal (8.5 x 14 in)")]
        Legal
    }

    impl PaperSize {
        fn to_naps2_arg(&self) -> String {
            match self {
                PaperSize::A4 => String::from("a4"),
                PaperSize::Letter => String::from("letter"),
                PaperSize::Long => String::from("8.5x13in"),
                PaperSize::Legal => String::from("legal")
            }
        }
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
    enum DPI {
        #[strum(serialize = "100")]
        DPI100,
        #[strum(serialize = "200")]
        DPI200,
        #[strum(serialize = "300")]
        #[default]
        DPI300,
        #[strum(serialize = "400")]
        DPI400,
        #[strum(serialize = "600")]
        DPI600,
        #[strum(serialize = "800")]
        DPI800,
        #[strum(serialize = "1200")]
        DPI1200
    }

    impl DPI {
        fn to_naps2_arg(&self) -> String {
            match self {
                DPI::DPI100 => String::from("100"),
                DPI::DPI200 => String::from("200"),
                DPI::DPI300 => String::from("300"),
                DPI::DPI400 => String::from("400"),
                DPI::DPI600 => String::from("600"),
                DPI::DPI800 => String::from("800"),
                DPI::DPI1200 => String::from("1200")
            }
        }
    }

    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
    enum Bitdepth {
        #[strum(serialize = "24-bit Color")]
        #[default]
        Color,
        #[strum(serialize = "Grayscale")]
        Grayscale,
        #[strum(serialize = "Black & White")]
        BlackAndWhite
    }

    impl Bitdepth {
        fn to_naps2_arg(&self) -> String {
            match self {
                Bitdepth::Color => String::from("color"),
                Bitdepth::Grayscale => String::from("gray"),
                Bitdepth::BlackAndWhite => String::from("bw")
            }
        }
    }

    async fn fetch_scanners() -> Result<Vec<String>, String> {
        #[cfg(target_os="windows")]
        let output = Command::new("./naps2/NAPS2.Console.exe")
            .arg("--listdevices")
            .arg("--driver")
            .arg("wia")
            .output()
            .map_err(|err| err.to_string())?;

        // Requires NAPS2 to be installed
        #[cfg(target_os="linux")]
        let output = Command::new("naps2")
            .arg("console")
            .arg("--listdevices")
            .arg("--driver")
            .arg("sane")
            .output()
            .map_err(|err| err.to_string())?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let list = stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
                Ok(list)
        }
        else {
            Err("Error fetching scanners".to_string())
        }
    }

    async fn run_scan(device: String, source: String, page_size: String, dpi: String, bitdepth: String) -> Result<PathBuf, String> {
        // let temp_path = std::env::temp_dir().join("scan_temp/scan.pdf");
        // if let Some(parent) = temp_path.parent() {
        //     let _ = fs::create_dir_all(parent);
        // }

        // match fs::create_dir("./scan_temp") {
        //     Err(err) => {
        //         error!("Error creating temp dir: {}", err);
        //     },
        //     _ => {}
        // }
        let temp_path = PathBuf::from("./scan.pdf");
        #[cfg(target_os = "windows")]
        let mut output = Command::new("./naps2/NAPS2.Console.exe")
            .arg("-o")
            .arg(format!("{}", &temp_path.to_string_lossy()))
            .arg("--driver")
            .arg("wia")
            .arg("--device")
            .arg(device)
            .arg("--source")
            .arg(source)
            .arg("--pagesize")
            .arg(page_size)
            .arg("--dpi")
            .arg(dpi)
            .arg("--bitdepth")
            .arg(bitdepth)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| err.to_string())?;

        #[cfg(target_os = "linux")]
        let mut output = Command::new("naps2")
            .arg("console")
            .arg("-o")
            .arg(format!("{}", &temp_path.to_string_lossy()))
            .arg("--driver")
            .arg("sane")
            .arg("--device")
            .arg(device)
            .arg("--source")
            .arg(source)
            .arg("--pagesize")
            .arg(page_size)
            .arg("--dpi")
            .arg(dpi)
            .arg("--bitdepth")
            .arg(bitdepth)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| err.to_string())?;

        let status = output.wait().map_err(|err| err.to_string())?;

        if !status.success() {
            let mut err_msg = String::new();
            output.stderr.take().unwrap().read_to_string(&mut err_msg).ok();
            return Err(format!("NAPS2 Error: {}", err_msg));
        }

        Ok(temp_path)
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
        
        let compressed_bytes = match compress_in_memory(bytes.to_vec(), &parameters) {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("Error compressing image: {}", err);
                Vec::new()
            }
        };

        return compressed_bytes
    }

    fn pdf_to_png(bytes: Vec<u8>) -> Vec<Vec<u8>> {
        let pdfium = Pdfium::default();
        let document = match pdfium.load_pdf_from_byte_vec(bytes, None) {
            Ok(document) => document,
            Err(err) => {
                error!("Error loading PDF document: {}", err);
                pdfium.create_new_pdf().unwrap()
            }
        };
        
        let mut bitmaps: Vec<Vec<u8>> = Vec::new();

        for page in document.pages().iter() {
            let mut config = PdfRenderConfig::new().set_fixed_size(2480, 3508);
            if page.is_landscape() {
                config = PdfRenderConfig::new().set_fixed_size(3508, 2480);
            }
            let mut bytes: Vec<u8> = Vec::new();
            match page.render_with_config(&config).unwrap()
                .as_image()
                .write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Bmp) {
                    Err(err) => {
                        error!("Error creating PDF page from image bytes: {}", err);
                    }
                    _ => {}
                }

            bitmaps.push(bytes);
        }

        return bitmaps
    }

    fn export_to_pdf(byte_vec: Vec<Vec<u8>>, path: PathBuf) -> Result<(), PdfiumError> {
        let pdfium = Pdfium::default();
        let mut document = match pdfium.create_new_pdf() {
            Ok(document) => document,
            Err(err) => {
                error!("Error creating new PDF document: {}", err);
                panic!("Error creating new PDF document: {}", err);
            }
        };
        
        for bytes in byte_vec {
            let mut width: f32 = 0.0;
            let mut height: f32 = 0.0;
            let raw_bytes = match image::load_from_memory(&bytes) {
                Ok(bytes) => {
                    let mut converted_bytes: Vec<u8> = Vec::new();
                    if bytes.width() > bytes.height() {
                        match bytes.resize(3508, 2480, image::imageops::FilterType::Lanczos3).write_to(&mut Cursor::new(&mut converted_bytes), image::ImageFormat::Png) {
                            Err(err) => {
                                error!("Error converting image: {}", err);
                            }
                            _ => {}
                        }
                        width = (3508 / 300 * 72) as f32;
                        height = (2480 / 300 * 72) as f32;
                    }
                    else {
                        match bytes.resize(2480, 3508, image::imageops::FilterType::Lanczos3).write_to(&mut Cursor::new(&mut converted_bytes), image::ImageFormat::Png) {
                            Err(err) => {
                                error!("Error converting image: {}", err);
                            }
                            _ => {}
                        }
                        width = (2480 / 300 * 72) as f32;
                        height = (3508 / 300 * 72) as f32;
                    }
                    

                    let mut parameters = CSParameters::new();
                    parameters.png.quality = 10;
                    parameters.png.optimization_level = 6;
                    let compressed_bytes = match compress_in_memory(converted_bytes, &parameters) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            error!("Error compressing image: {}", err);
                            panic!("Error compressing image: {}", err);
                        }
                    };
                    

                    Some(match image::load_from_memory(&compressed_bytes) {
                        Ok(image) => image,
                        Err(err) => {
                            error!("Error loading compressed bytes from memory: {}", err);
                            panic!("Error loading compressed bytes from memory: {}", err);
                        }
                    })
                },
                Err(err) => {
                    error!("Error loading image from memory: {}", err);
                    panic!("Error loading image from memory: {}", err);
                }
            };
            let mut page = match width > height {
                true => match document.pages_mut().create_page_at_end(PdfPagePaperSize::a4().landscape()) {
                    Ok(page) => page,
                    Err(err) => {
                        error!("Error creating PDF page: {}", err);
                        panic!("Error creating PDF page: {}", err);
                    }
                },
                false => match document.pages_mut().create_page_at_end(PdfPagePaperSize::a4()) {
                    Ok(page) => page,
                    Err(err) => {
                        error!("Error creating PDF page: {}", err);
                        panic!("Error creating PDF page: {}", err);
                    }
                },
            };
            
            if width > height {
                match page.objects_mut().create_image_object((PdfPagePaperSize::a4().height() - PdfPoints::new(width)) / 2.0, (PdfPagePaperSize::a4().width() - PdfPoints::new(height)) / 2.0, raw_bytes.as_ref().unwrap(), Some(PdfPoints::new(width)), Some(PdfPoints::new(height))) {
                    Err(err) => {
                        error!("Error adding image to PDF page: {}", err);
                        panic!("Error adding image to PDF page: {}", err);
                    },
                    _ => {}
                };
            }
            else {
                match page.objects_mut().create_image_object((PdfPagePaperSize::a4().width() - PdfPoints::new(width)) / 2.0, (PdfPagePaperSize::a4().height() - PdfPoints::new(height)) / 2.0, raw_bytes.as_ref().unwrap(), Some(PdfPoints::new(width)), Some(PdfPoints::new(height))) {
                    Err(err) => {
                        error!("Error adding image to PDF page: {}", err);
                        panic!("Error adding image to PDF page: {}", err);
                    },
                    _ => {}
                };
            }
            
        }

        match document.save_to_file(&path) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!("Error saving PDF file: {}", err);
                return Err(err)
            }
        }
    }

    fn reveal_file(path: &PathBuf) {
        match opener::reveal(path) {
            Err(err) => {
                error!("Error revealing file: {}", err);
            }
            _ => {}
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
        FileHover,
        FileDrop(PathBuf),
        FileHoverLeft,
        Back,
        KeyEvent(Key),
        ShowScanDialog,
        RefreshDeviceList,
        ScannersFound(Vec<String>),
        ScannerFetchFail,
        SelectDevice(String),
        SelectSource(PaperSource),
        SelectPageSize(PaperSize),
        SelectDPI(DPI),
        SelectBitdepth(Bitdepth),
        CloseScanDialog,
        Scan,
        Scanned(PathBuf),
        ScanFail,
        ScanTick,
        RotateImage,
        RemoveImage(usize),
        OpenFullImageViewer,
        CloseFullImageViewer,
        ClearImageFiles,
        ExportToPdf,
        ExportSuccess(PathBuf),
        ExportFail,
        PrevPage,
        NextPage,
        OpenLink(String),
        None,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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