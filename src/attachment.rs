pub(crate) mod attachment {
    
    use std::sync::Arc;

    use crate::attachment_page::attachment_page::AttachmentPage;


    #[derive(Debug, Clone)]
    pub(crate) struct Attachment {
        attachment_id: u32,
        pages: Vec<AttachmentPage>,
        reference_number: Arc<String>,
        comment: Arc<String>,
        date_added: i64,
        document_id: u32,
    }

    impl Attachment {
        pub(crate) fn new(attachment_id: u32, pages: Vec<AttachmentPage>, reference_number: Arc<String>, comment: Arc<String>, date_added: i64, document_id: u32) -> Attachment {
            Attachment {
                attachment_id: attachment_id,
                pages: pages,
                reference_number: reference_number,
                comment: comment,
                date_added: date_added,
                document_id: document_id,
            }
        }

        pub(crate) fn get_attachment_id(&self) -> u32 {
            return self.attachment_id
        }

        pub(crate) fn pages(&self) -> &Vec<AttachmentPage> {
            return &self.pages
        }

        pub(crate) fn page(&self, i: usize) -> &AttachmentPage {
            return &self.pages[i]
        }

        pub(crate) fn get_reference_number(&self) -> Arc<String> {
            return self.reference_number.clone()
        }

        pub(crate) fn get_comment(&self) -> Arc<String> {
            return self.comment.clone()
        }

        pub(crate) fn get_date_added(&self) -> i64 {
            return self.date_added
        }

        pub(crate) fn get_document_id(&self) -> u32 {
            return self.document_id
        }
    }
}