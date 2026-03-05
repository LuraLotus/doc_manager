pub(crate) mod main_menu;
pub(crate) mod document_list;
pub(crate) mod recycle_bin;
pub(crate) mod settings;

pub(crate) use crate::screen::main_menu::main_menu::MainMenu;
pub(crate) use crate::screen::document_list::document_list::DocumentList;
pub(crate) use crate::screen::recycle_bin::recycle_bin::RecycleBin;
pub(crate) use crate::screen::settings::settings::Settings;