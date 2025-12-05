pub(crate) mod db_module {
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;

    use rusqlite::Connection;
    use rusqlite::Result;

    use crate::screen::Attachment;
    use crate::screen::Document;

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
                conn.execute(
                "CREATE TABLE document (
                        document_id INTEGER PRIMARY KEY,
                        document_number TEXT NOT NULL,
                        document_type TEXT,
                        comment TEXT,
                        date_added INTEGER NOT NULL DEFAULT (unixepoch('now'))
                    )", ())?;

                conn.execute("CREATE TABLE attachment (
                        attachment_id INTEGER PRIMARY KEY,
                        file_path TEXT NOT NULL,
                        reference_number TEXT,
                        comment TEXT,
                        date_added INTEGER NOT NULL DEFAULT (unixepoch('now')),
                        document_id INTEGER NOT NULL,
                        FOREIGN KEY(document_id) REFERENCES document(document_id)
                    )", ())?;


                    conn.execute("INSERT INTO document (document_number, document_type, comment) VALUES (?1, ?2, ?3)", ("Test Number", "Test Type", "Test Comment"))?;
                    conn.execute("INSERT INTO document (document_number, document_type, comment) VALUES (?1, ?2, ?3)", ("Test Number 2", "Test Type 2", "Test Comment"))?;
                    conn.execute("INSERT INTO attachment (file_path, reference_number, comment, document_id) VALUES (?1, ?2, ?3, ?4)", ("Test Path", "12345", "Test Comment", 1))?;
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
            let mut stmt = self.conn.prepare("SELECT attachment_id, file_path, reference_number, comment, date_added, document_id FROM attachment WHERE document_id = ?1").unwrap();
            let attachment_data = stmt.query_map([document_id], |row| {
                Ok(Attachment::new(row.get(0)?, Arc::new(row.get(1)?), Arc::new(row.get(2)?), Arc::new(row.get(3)?), row.get(4)?, row.get(5)?))
            })?;

            let mut attachments: Vec<Arc<Attachment>> = Vec::new();
            for attachment in attachment_data {
                attachments.push(Arc::new(attachment?));
            }

            return Ok(attachments)
        }

        pub(crate) fn new_document(&mut self, document_number: String, document_type: String, comment: String) -> Result<usize, rusqlite::Error> {
            let result =  self.conn.execute("INSERT INTO document (document_number, document_type, comment) VALUES (?1, ?2, ?3)", (document_number, document_type, comment));
            self.last_rowid = Some(self.conn.last_insert_rowid());
            return result
        }

        pub(crate) fn new_attachment(&mut self, file_path: String, reference_number: String, comment: String, document_id: u32) -> Result<usize, rusqlite::Error> {
            let result = self.conn.execute("INSERT INTO attachment (file_path, reference_number, comment, document_id) VALUES (?1, ?2, ?3, ?4)", (file_path, reference_number, comment, document_id));
            self.last_rowid = Some(self.conn.last_insert_rowid());
            return result
        }

        pub(crate) fn edit_document_details(&mut self, document_id: u32, document_number: String, document_type: String, comment: String) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE document SET document_number = ?1, document_type = ?2, comment = ?3 WHERE document_id = ?4", (document_number, document_type, comment, document_id))
        }

        pub(crate) fn edit_attachment_details(&mut self, attachment_id: u32, reference_number: String, comment: String) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE attachment SET reference_number = ?1, comment = ?2 WHERE attachment_id = ?3", (reference_number, comment, attachment_id))
        }

        pub(crate) fn edit_attachment_file_path(&mut self, attachment_id: u32, file_path: String) -> Result<usize, rusqlite::Error> {
            return self.conn.execute("UPDATE attachment SET file_path = ?1 WHERE attachment_id = ?2", (file_path, attachment_id))
        }

        pub(crate) fn get_last_rowid(&self) -> Option<i64> {
            return self.last_rowid
        }
    }

    

    pub(crate) enum DbTable {
        DocumentTable,
        FilePathTable,
    }

    
}