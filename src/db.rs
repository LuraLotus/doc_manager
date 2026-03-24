pub(crate) mod db_module {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

    use log::error;
    use log::info;
    use rusqlite::Connection;
    use rusqlite::Result;
    use rusqlite::config::DbConfig;
    use rusqlite::ffi::SQLITE_DBCONFIG_ENABLE_FKEY;
    use rusqlite_migration::M;
    use rusqlite_migration::Migrations;

    use crate::attachment::attachment::Attachment;
    use crate::attachment_page::attachment_page::AttachmentPage;
    use crate::document;
    use crate::document::document::Document;

    #[derive(Debug)]
    pub(crate) struct DbConnection {
        conn: Connection,
        last_rowid: Option<i64>,
    }

    impl DbConnection {
        pub(crate) fn new() -> DbConnection {
            DbConnection { 
                conn: Self::db_init(),
                last_rowid: None,
            }
        }

        pub(crate) fn db_init() -> Connection {
            if !Path::new("./data.db").exists() {
                let conn = Self::connect();
                match conn.execute(
                "CREATE TABLE document (
                        document_id INTEGER PRIMARY KEY,
                        document_number TEXT NOT NULL UNIQUE,
                        document_type TEXT,
                        comment TEXT,
                        date_added INTEGER NOT NULL DEFAULT (unixepoch('now'))
                    )", ()
                ) {
                    Err(err) => {
                        error!("Error creating document table: {}", err);
                        panic!("Error creating document table: {}", err);
                    },
                    _ => {}
                };

                match conn.execute("CREATE TABLE attachment (
                        attachment_id INTEGER PRIMARY KEY,
                        reference_number TEXT NOT NULL UNIQUE,
                        comment TEXT,
                        date_added INTEGER NOT NULL DEFAULT (unixepoch('now')),
                        document_id INTEGER NOT NULL,
                        FOREIGN KEY(document_id) REFERENCES document(document_id) ON DELETE CASCADE
                    )", ()
                ) {
                    Err(err) => {
                        error!("Error creating attachment table: {}", err);
                        panic!("Error creating attachment table: {}", err);
                    },
                    _ => {}
                };

                match conn.execute("CREATE TABLE page (
                        page_id INTEGER PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        attachment_id INTEGER NOT NULL,
                        FOREIGN KEY(attachment_id) REFERENCES attachment(attachment_id) ON DELETE CASCADE
                )", ()
                ) {
                    Err(err) => {
                        error!("Error creating page table: {}", err);
                        panic!("Error creating page table: {}", err);
                    },
                    _ => {}
                };

                match fs::create_dir("data") {
                    Err(err) => {
                        error!("Error creating data directory: {}", err);
                        panic!("Error creating data directory: {}", err);
                    },
                    _ => {}
                }
                Self::update_db();
                info!("DB initialized.");

                return conn;
                
            }
            else {
                Self::update_db();

                let conn = Self::connect();
                info!("DB initialized.");
                return conn;
            }
        }

        fn update_db() {
            let mut conn = Self::connect();
            let migrations = Migrations::new(vec![
                M::up("ALTER TABLE document ADD COLUMN date_deleted INTEGER"),
                M::up("ALTER TABLE attachment ADD COLUMN date_deleted INTEGER")
            ]);

            match migrations.to_latest(&mut conn) {
                Ok(_) => info!("Database migration successful"),
                Err(err) => error!("Database migration failed: {}", err)
            }

            match Self::delete_old_inactive_documents(&mut conn) {
                Err(err) => error!("Error deleting old inactive documents: {}", err),
                _ => {}
            };
            match Self::delete_old_inactive_attachments(&mut conn) {
                Err(err) => error!("Error deleting old inactive attachments: {}", err),
                _ => {}
            };
        }

        fn connect() -> Connection {
            let conn = match Connection::open("./data.db") {
                Ok(conn) => conn,
                Err(err) => {
                    error!("Error connecting to database: {}", err);
                    panic!("Error connecting to database: {}", err);
                }
            };
            match conn.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY, true) {
                Err(err) => {
                    error!("Error enabling foreign keys during database connection: {}", err);
                    panic!("Error enabling foreign keys during database connection: {}", err);
                },
                _ => {}
            };
            return conn
        }

        pub(crate) fn read_document_table(&self) -> Result<Vec<Arc<Document>>, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT document_id, document_number, document_type, comment, date_added FROM document WHERE date_deleted IS NULL").unwrap();
            let document_data = stmt.query_map([], |row| {
                Ok(Document::new(
                    row.get(0)?,
                    Arc::new(row.get(1)?),
                    Arc::new(row.get(2)?),
                    Some(match self.read_attachment_table(row.get(0)?) {
                        Ok(attachments) => attachments,
                        Err(err) => {
                            error!("Error reading attachment table: {}", err);
                            panic!("Error reading attachment table: {}", err);
                        }
                    }),
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

        pub(crate) fn read_deleted_documents(&self) -> Result<Vec<Arc<Document>>, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT document_id, document_number, document_type, comment, date_added FROM document WHERE date_deleted IS NOT NULL").unwrap();
            let document_data = stmt.query_map([], |row| {
                Ok(Document::new(
                    row.get(0)?,
                    Arc::new(row.get(1)?),
                    Arc::new(row.get(2)?),
                    Some(match self.read_attachment_table(row.get(0)?) {
                        Ok(attachments) => attachments,
                        Err(err) => {
                            error!("Error reading attachment table: {}", err);
                            panic!("Error reading attachment table: {}", err);
                        }
                    }),
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
            let mut stmt = self.conn.prepare("SELECT attachment_id, reference_number, comment, date_added, document_id FROM attachment WHERE document_id = ?1 AND date_deleted IS NULL").unwrap();
            let attachment_data = stmt.query_map([document_id], |row| {
                Ok(Attachment::new(
                    row.get(0)?,
                    match self.read_pages_table(row.get(0)?) {
                        Ok(pages) => pages,
                        Err(err) => {
                            error!("Error reading pages table: {}", err);
                            panic!("Error reading pages table: {}", err);
                        }
                    },
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

        pub(crate) fn read_deleted_attachments(&self) -> Result<Vec<Arc<Attachment>>, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT attachment_id, reference_number, comment, date_added, document_id FROM attachment WHERE date_deleted IS NOT NULL").unwrap();
            let attachment_data = stmt.query_map([], |row| {
                Ok(Attachment::new(
                    row.get(0)?,
                    match self.read_pages_table(row.get(0)?) {
                        Ok(pages) => pages,
                        Err(err) => {
                            error!("Error reading pages table: {}", err);
                            panic!("Error reading pages table: {}", err);
                        }
                    },
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
            let transaction = match self.conn.transaction() {
                Ok(transaction) => transaction,
                Err(err) => {
                    error!("Error creating transaction: {}", err);
                    panic!("Error creating transaction: {}", err);
                }
            };
            transaction.execute("DELETE FROM page WHERE attachment_id = ?1", (attachment_id,))?;
            for path in file_paths {
                transaction.execute("INSERT INTO page (file_path, attachment_id) VALUES (?1, ?2)", (path.to_string_lossy(), attachment_id))?;
            }
            transaction.commit()
        }

        pub(crate) fn edit_attachment_file_path(&mut self, attachment_id: u32, file_path: String) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE attachment SET file_path = ?1 WHERE attachment_id = ?2", (file_path, attachment_id))
        }

        pub(crate) fn set_document_deleted(&mut self, document_id: u32) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE document SET date_deleted = unixepoch('now') WHERE document_id = ?1", (document_id,));
        }

        pub(crate) fn set_attachment_deleted(&mut self, attachment_id: u32) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE attachment SET date_deleted = unixepoch('now') WHERE attachment_id = ?1", (attachment_id,));
        }

        pub(crate) fn restore_document(&mut self, document_id: u32) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE document SET date_deleted = NULL WHERE document_id = ?1", (document_id,));
        }

        pub(crate) fn restore_attachment(&mut self, attachment_id: u32) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE attachment SET date_deleted = NULL WHERE attachment_id = ?1", (attachment_id,));
        }

        pub(crate) fn delete_document(&mut self, document_id: u32) -> Result<usize, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT document_number FROM document WHERE document_id = ?1")?;
            let document_number: String = stmt.query_row([document_id], |row| row.get(0))?;
            let result = self.conn.execute("DELETE FROM document WHERE document_id = ?1", (document_id.clone(),));
            match result {
                Ok(_) => {
                    match fs::remove_dir_all(format!("./data/{}", document_number)) {
                        Err(err) => error!("Error deleting data directory: {}", err),
                        Ok(_) => {}
                    };
                },
                _ => {}
            };
            return result
        }

        pub(crate) fn delete_attachment(&mut self, attachment_id: u32) -> Result<usize, rusqlite::Error> {
            let mut stmt = self.conn.prepare("SELECT reference_number, document_id FROM attachment WHERE attachment_id = ?1")?;
            let row_data: (String, u32) = stmt.query_row([attachment_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let attachment_number = row_data.0;
            let document_id = row_data.1;
            
            let mut stmt = self.conn.prepare("SELECT document_number From document WHERE document_id = ?1")?;
            let document_number: String = stmt.query_row([document_id], |row| row.get(0))?;
            let result = self.conn.execute("DELETE FROM attachment WHERE attachment_id = ?1", (attachment_id,));
            match result {
                Ok(_) => {
                    match fs::remove_dir_all(format!("./data/{}/{}", document_number, attachment_number)) {
                        Err(err) => error!("Error deleting file: {}", err),
                        Ok(_) => {}
                    }
                },
                _ => {}
            };
            
            return result
        }

        fn delete_old_inactive_documents(conn: &mut Connection) -> Result<usize, rusqlite::Error> {
            let mut stmt = conn.prepare("SELECT document_number FROM document WHERE date_deleted < unixepoch(date('now', '-30 day'))")?;
            let mut document_numbers: Vec<String> = Vec::new();
            let rows =  stmt.query_map([], |row| {
                Ok(row.get(0)?)
            })?;

            for row in rows {
                document_numbers.push(row?);
            }

            let document_delete_result = conn.execute("DELETE FROM document WHERE date_deleted < unixepoch(date('now', '-30 day'))", ());
            match document_delete_result {
                Ok(_) => {
                    for document_number in document_numbers {
                        match fs::remove_dir_all(format!("./data/{}", document_number)) {
                            Err(err) => error!("Error deleting data directory: {}", err),
                            Ok(_) => {}
                        };
                    }
                },
                _ => {}
            };

            return document_delete_result
        }

        fn delete_old_inactive_attachments(conn: &mut Connection) -> Result<usize, rusqlite::Error> {
            let mut stmt = conn.prepare("SELECT reference_number, document.document_number FROM attachment INNER JOIN document ON attachment.document_id = document.document_id WHERE attachment.date_deleted < unixepoch(date('now', '-30 day'))")?;
            let mut number_list: Vec<(String, String)> = Vec::new();
            let rows = stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?;

            for data in rows {
                number_list.push(data?);
            }

            let attachment_delete_result = conn.execute("DELETE FROM attachment WHERE attachment.date_deleted < unixepoch(date('now', '-30 day'))", ());
            match attachment_delete_result {
                Ok(_) => {
                    for numbers in number_list {
                        match fs::remove_dir_all(format!("./data/{}/{}", numbers.1, numbers.0)) {
                            Err(err) => error!("Error deleting data directory: {}", err),
                            Ok(_) => {}
                        };
                    }
                },
                _ => {}
            };

            return attachment_delete_result
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