pub(crate) mod main_menu;
pub(crate) mod document_list;
pub(crate) mod document;
pub(crate) mod attachment;
pub(crate) mod settings;

pub(crate) use crate::screen::main_menu::main_menu::MainMenu;
pub(crate) use crate::screen::document_list::document_list::DocumentList;
pub(crate) use crate::screen::document::document::Document;
pub(crate) use crate::screen::attachment::attachment::Attachment;
pub(crate) use crate::screen::settings::settings::Settings;