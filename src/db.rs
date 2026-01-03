pub(crate) mod db_module {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

    use rusqlite::Connection;
    use rusqlite::Result;
    use rusqlite::config::DbConfig;
    use rusqlite::ffi::SQLITE_DBCONFIG_ENABLE_FKEY;

    use crate::attachment::attachment::Attachment;
    use crate::attachment_page::attachment_page::AttachmentPage;
    use crate::document::document::Document;

    #[derive(Debug)]
    pub(crate) struct DbConnection {
        conn: Connection,
        last_rowid: Option<i64>,
    }

    impl DbConnection {
        pub(crate) fn new() -> DbConnection {
            DbConnection { 
                conn: Result::expect(Self::db_init(), "Error connecting to database."),
                last_rowid: None,
            }
        }

        pub(crate) fn db_init() -> Result<Connection, rusqlite::Error> {
            if !Path::new("./data.db").exists() {
                let conn = Connection::open("./data.db")?;
                conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true);
                conn.execute(
                "CREATE TABLE document (
                        document_id INTEGER PRIMARY KEY,
                        document_number TEXT NOT NULL UNIQUE,
                        document_type TEXT,
                        comment TEXT,
                        date_added INTEGER NOT NULL DEFAULT (unixepoch('now'))
                    )", ()
                )?;

                conn.execute("CREATE TABLE attachment (
                        attachment_id INTEGER PRIMARY KEY,
                        reference_number TEXT NOT NULL UNIQUE,
                        comment TEXT,
                        date_added INTEGER NOT NULL DEFAULT (unixepoch('now')),
                        document_id INTEGER NOT NULL,
                        FOREIGN KEY(document_id) REFERENCES document(document_id) ON DELETE CASCADE
                    )", ()
                )?;

                conn.execute("CREATE TABLE page (
                        page_id INTEGER PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        attachment_id INTEGER NOT NULL,
                        FOREIGN KEY(attachment_id) REFERENCES attachment(attachment_id) ON DELETE CASCADE
                )", ()
                )?;

                    fs::create_dir("data").unwrap_or_else(|err| {
                        println!("Error creating data folder: {}", err);
                    });
                    println!("DB initialized.");

                return Ok(conn);
                
            }
            else {
                println!("DB initialized.");
                return Self::connect();
            }
            
            
        }

        fn connect() -> Result<Connection, rusqlite::Error> {
            return Connection::open("./data.db");
        }

        pub(crate) fn read_document_table(&self) -> Result<Vec<Arc<Document>>, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT document_id, document_number, document_type, comment, date_added FROM document").unwrap();
            let document_data = stmt.query_map([], |row| {
                Ok(Document::new(
                    row.get(0)?,
                    Arc::new(row.get(1)?),
                    Arc::new(row.get(2)?),
                    Some(Result::expect(self.read_attachment_table(row.get(0)?), "Error reading attachment table.")),
                    Arc::new(row.get(3)?),
                    row.get(4)?,
                ))
            })?;

            let mut documents: Vec<Arc<Document>> = Vec::new();
            for document in document_data {
                documents.push(Arc::new(document?));
            }

            return Ok(documents);
        }

        pub(crate) fn read_attachment_table(&self, document_id: u32) -> Result<Vec<Arc<Attachment>>, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT attachment_id, reference_number, comment, date_added, document_id FROM attachment WHERE document_id = ?1").unwrap();
            let attachment_data = stmt.query_map([document_id], |row| {
                Ok(Attachment::new(
                    row.get(0)?,
                    Result::expect(self.read_pages_table(row.get(0)?), "Error reading pages table."),
                    Arc::new(row.get(1)?),
                    Arc::new(row.get(2)?),
                    row.get(3)?,
                    row.get(4)?
                ))
            })?;

            let mut attachments: Vec<Arc<Attachment>> = Vec::new();
            for attachment in attachment_data {
                attachments.push(Arc::new(attachment?));
            }

            return Ok(attachments)
        }
        
        pub(crate) fn read_pages_table(&self, attachment_id: u32) -> Result<Vec<AttachmentPage>, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT page_id, file_path, attachment_id FROM page WHERE attachment_id = ?1").unwrap();
            let page_data = stmt.query_map([attachment_id], |row| {
                Ok(AttachmentPage::new(
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?
                ))
            })?;

            let mut pages: Vec<AttachmentPage> = Vec::new();
            for page in page_data {
                pages.push(page?);
            }

            return Ok(pages)
        }

        pub(crate) fn new_document(&mut self, document_number: String, document_type: String, comment: String) -> Result<usize, rusqlite::Error> {
            let result =  self.conn.execute("INSERT INTO document (document_number, document_type, comment) VALUES (?1, ?2, ?3)", (document_number, document_type, comment));
            self.last_rowid = Some(self.conn.last_insert_rowid());
            return result
        }

        pub(crate) fn new_attachment(&mut self, file_paths: Vec<PathBuf>, reference_number: String, comment: String, document_id: u32) -> Result<(), rusqlite::Error> {
            self.conn.execute("INSERT INTO attachment (reference_number, comment, document_id) VALUES (?1, ?2, ?3)", (reference_number, comment, document_id))?;
            self.last_rowid = Some(self.conn.last_insert_rowid());
            let transaction = self.conn.transaction()?;
            for path in file_paths {
                transaction.execute("INSERT INTO page (file_path, attachment_id) VALUES (?1, ?2)", (path.to_string_lossy(), self.last_rowid))?;
            }
            transaction.commit()
        }

        pub(crate) fn edit_document_details(&mut self, document_id: u32, document_number: String, document_type: String, comment: String) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE document SET document_number = ?1, document_type = ?2, comment = ?3 WHERE document_id = ?4", (document_number, document_type, comment, document_id))
        }

        pub(crate) fn edit_attachment_details(&mut self, attachment_id: u32, reference_number: String, comment: String) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE attachment SET reference_number = ?1, comment = ?2 WHERE attachment_id = ?3", (reference_number, comment, attachment_id))
        }

        pub(crate) fn edit_attachment_pages(&mut self, attachment_id: u32, file_paths: Vec<PathBuf>) -> Result<(), rusqlite::Error> {
            let transaction = self.conn.transaction().expect("Error creating transaction");
            transaction.execute("DELETE FROM page WHERE attachment_id = ?1", (attachment_id,))?;
            for path in file_paths {
                transaction.execute("INSERT INTO page (file_path, attachment_id) VALUES (?1, ?2)", (path.to_string_lossy(), attachment_id))?;
            }
            transaction.commit()
        }

        pub(crate) fn edit_attachment_file_path(&mut self, attachment_id: u32, file_path: String) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE attachment SET file_path = ?1 WHERE attachment_id = ?2", (file_path, attachment_id))
        }

        pub(crate) fn delete_document(&mut self, document_id: u32) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("DELETE FROM document WHERE document_id = ?1", (document_id.clone(),))
        }

        pub(crate) fn delete_attachment(&mut self, attachment_id: u32) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("DELETE FROM attachment WHERE attachment_id = ?1", (attachment_id,))
            // let transaction = self.conn.transaction().expect("Error creating transaction");
            // transaction.execute("DELETE FROM page WHERE attachment_id = ?1", (attachment_id,))?;
            // transaction.execute("DELETE FROM attachment WHERE attachment_id = ?1", (attachment_id,))?;
            // transaction.commit()
        }

        pub(crate) fn last_rowid(&self) -> Option<i64> {
            return self.last_rowid
        }
    }

    

    pub(crate) enum DbTable {
        DocumentTable,
        FilePathTable,
    }

    
}