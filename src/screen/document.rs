pub(crate) mod document {
    use std::sync::Arc;

    use crate::screen::attachment::attachment::Attachment;
    use iced::{Element, widget::Container};

    #[derive(Debug, Clone)]
    pub(crate) struct Document {
        document_id: u32,
        document_number: Arc<String>,
        document_type: Arc<String>,
        attachments: Option<Vec<Arc<Attachment>>>,
        comment: Arc<String>,
        date_added: i64,
    }

    impl Document {
        pub(crate) fn new(document_id:u32, document_number: Arc<String>, document_type: Arc<String>, attachments: Option<Vec<Arc<Attachment>>>, comment: Arc<String>, date_added: i64) -> Document {
            Document {
                document_id: document_id,
                document_number: document_number,
                document_type: document_type,
                attachments: attachments,
                comment: comment,
                date_added: date_added,
            }
        }

        pub(crate) fn get_document_id(&self) -> u32 {
            return self.document_id
        }

        pub(crate) fn get_document_number(&self) -> Arc<String> {
            return self.document_number.clone()
        }

        pub(crate) fn get_document_type(&self) -> Arc<String> {
            return self.document_type.clone()
        }

        pub(crate) fn get_attachments(&self) -> Option<Vec<Arc<Attachment>>> {
            return self.attachments.clone()
        }

        pub(crate) fn get_comment(&self) -> Arc<String> {
            return self.comment.clone()
        }

        pub(crate) fn get_date_added(&self) -> i64 {
            return self.date_added
        }

        pub(crate) fn update(&mut self, message: Message) {

        }

        pub(crate) fn view(&self) -> Element<Message> {
            Container::new("Document").into()
        }
    }

    pub(crate) enum Message {
        OpenAttachment,
        None,
    }
}