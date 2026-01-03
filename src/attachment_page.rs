pub(crate) mod attachment_page {
    use std::{fs, sync::Arc};

    #[derive(Debug, Clone)]
    pub(crate) struct AttachmentPage {
        page_id: u32,
        image: Vec<u8>,
        file_path: Arc<String>,
        attachment_id: u32
    }

    impl AttachmentPage {
        pub(crate) fn new(page_id: u32, file_path: String, attachment_id: u32) -> AttachmentPage {
            AttachmentPage {
                page_id: page_id,
                image: match fs::read(&file_path) {
                    Ok(image) => image,
                    Err(err) => {
                        println!("Error reading image file: {}", err);
                        Vec::new()
                    }
                },
                file_path: file_path.into(),
                attachment_id: attachment_id
            }
        }

        pub(crate) fn image(&self) -> &Vec<u8> {
            &self.image
        }

        pub(crate) fn file_path(&self) -> Arc<String> {
            self.file_path.clone()
        }
    }
}