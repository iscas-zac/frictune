use std::io::Read;

use gluesql::{prelude::{MemoryStorage, Glue, Payload, Row, DataType, Value}, core::{executor::ValidateError, result}};

pub struct Database {
    conn: Glue<MemoryStorage>,
}

/// Database error type wrapper
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    UniqueViolation,
    GlueError(result::Error),
}

/// Gluesql data type wrapper and converter
pub trait GlueType {
    fn get_glue_type() -> DataType;
    fn get_content(thing: Value) -> Self;
}

impl GlueType for i32 {
    fn get_glue_type() -> DataType {
        DataType::Int32
    }
    fn get_content(thing: Value) -> Self {
        if let Value::I32(content) = thing {
            content
        }
        else { Self::default() }
    }
}

impl GlueType for f32 {
    fn get_glue_type() -> DataType {
        DataType::Float
    }
    fn get_content(thing: Value) -> Self {
        if let Value::F64(content) = thing {
            content as f32
        }
        else { Self::default() }
    }
}

impl GlueType for String {
    fn get_glue_type() -> DataType {
        DataType::Text
    }
    fn get_content(thing: Value) -> Self {
        if let Value::Str(content) = thing {
            content
        }
        else { Self::default() }
    }
}

impl GlueType for bool {
    fn get_glue_type() -> DataType {
        DataType::Boolean
    }
    fn get_content(thing: Value) -> Self {
        if let Value::Bool(content) = thing {
            content
        }
        else { Self::default() }
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UniqueViolation => f.write_str("unique violation"),
            Self::GlueError(e) => f.write_str(e.to_string().as_str()),
        }
    }
}

pub enum DatabaseResult {
    Success(String),
    Things(Vec<Row>),
}

impl From<Vec<Row>> for DatabaseResult {
    fn from(value: Vec<Row>) -> Self {
        DatabaseResult::Things(value)
    }
}

impl DatabaseResult {
    pub fn len(&self) -> usize {
        match self {
            Self::Success(_) => 0,
            Self::Things(v) =>
                v.len()
        }
    }

    pub fn get<T: GlueType>(&self, index: usize) -> Vec<T>
    {
        match self {
            Self::Success(_) => vec![],
            Self::Things(v) =>
                v.iter().map(|row|
                    T::get_content(
                        row.get_value_by_index(index).unwrap()
                            .cast(&T::get_glue_type()).unwrap()
                    )
                ).collect()
        }
    }
}


impl From<Vec<Payload>> for DatabaseResult {
    fn from(values: Vec<Payload>) -> Self {
        DatabaseResult::Things(values.into_iter().fold(vec![], |acc, payload|
            match payload {
                Payload::Select { labels: _, rows } => 
                    [acc, rows].concat(),
                _ => acc,
            }
        ))
    }
}

impl From<gluesql::core::result::Error> for DatabaseError {
    fn from(value: gluesql::core::result::Error) -> Self {
        match value {
            gluesql::core::result::Error::Validate(ValidateError::DuplicateEntryOnPrimaryKeyField(_k))
                => DatabaseError::UniqueViolation,
            other => DatabaseError::GlueError(other),
        }
    }
}

impl Database {
    fn init_command() -> &'static str {
        "CREATE TABLE IF NOT EXISTS tags
        (
            tag_name    TEXT PRIMARY KEY NOT NULL,
            info     TEXT DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS relationship
        (
            tag1 TEXT NOT NULL,
            tag2 TEXT NOT NULL,
            weight DECIMAL,
            is_origin BOOLEAN DEFAULT false,
            CONSTRAINT relationship_id1_fk FOREIGN KEY (tag1) REFERENCES tags(tag_name),
            CONSTRAINT relationship_id2_fk FOREIGN KEY (tag2) REFERENCES tags(tag_name),
            CONSTRAINT relation_pk PRIMARY KEY (tag1, tag2)
        );"
    }

    pub fn sync_new(db_url: &str) -> anyhow::Result<Database> {
        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                fn get_memory(db_url: &str) -> anyhow::Result<MemoryStorage> {
                    let mut f = std::fs::OpenOptions::new().read(true).write(true).create(true).open(db_url)?;
                    let mut buf = vec![];
                    std::io::Read::read_to_end(&mut f, &mut buf)?;
                    if !buf.is_empty()
                    { Ok(bincode::deserialize::<MemoryStorage>(&buf[..])?) }
                    else { anyhow::bail!("file is empty") }
                }
                let conn = match get_memory(db_url) {
                    Ok(ms) => Glue::new(ms),
                    Err(_) =>  {
                        let mut conn = Glue::new(MemoryStorage::default());
                        conn.execute(Self::init_command())?;
                        conn
                    }
                };
                Ok(Database { conn })
            }
            else {
                anyhow::bail!("wasm mode")
            }
        }
    }

    pub fn save(&self, db_url: &str) -> anyhow::Result<()> {
        cfg_if::cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                let mut f = std::fs::OpenOptions::new().write(true).create(true).open(db_url)?;
                let storage = match self.conn.storage.clone()
                { Some(s) => s, None => anyhow::bail!("no storage")};
                let buf = bincode::serialize(&storage)?;
                std::io::Write::write_all(&mut f, &buf)?;
                Ok(())
            }
            else {
                anyhow::bail!("wasm mode")
            }
        }
    }

    pub fn deser_new(content: &[u8]) -> anyhow::Result<Database> {
        let storage: MemoryStorage = bincode::deserialize(&content[..])?;
        let mut conn = Glue::new(storage);
        conn.execute(Self::init_command())?;
        Ok(Database { conn })
    }

    pub async fn create(&mut self, table: &str, entry: &[String], data: &[String]) -> Result<DatabaseResult, DatabaseError> {
        let query = &format!("INSERT INTO {} ({}) VALUES({});", table, entry.join(", "), data.join(", "));
        crate::logger::print(&query);
        self.conn.execute_async(
            query
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }

    pub async fn delete(&mut self, table: &str, entry: &str, data: &str) -> Result<DatabaseResult, DatabaseError> {
        let query = &format!("DELETE FROM {} WHERE {} = {}", table, entry, data);
        crate::logger::print(&query);
        self.conn.execute_async(
            query
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }

    pub async fn read(&mut self, table: &str, entry: &[String], cond: &str, opts: &str) -> Result<DatabaseResult, DatabaseError> {
        let query = &format!("SELECT {} FROM {} WHERE {} {}", entry.join(", "), table, cond, opts);
        crate::logger::print(&query);
        self.conn.execute_async(
            query
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }
    
    pub async fn update(&mut self, table: &str, entry: &[String], data: &[String],
            updated_entry: &[String], updated_data: &[String], cond: &str) -> Result<DatabaseResult, DatabaseError> {
        let mut same_items = vec![];
        for (idx, sing_entry) in entry.iter().enumerate() {
            if updated_entry.iter().find(|ue| ue == &sing_entry).is_some()
            { same_items.push(format!("{} = {}", sing_entry, data[idx])) }
        }
        let predicate = same_items.join(" AND ");
        let query = &format!("INSERT INTO {} ({}) VALUES ({}); {}",
            table,
            entry.join(", "),
            data.join(", "),
            updated_entry.iter().zip(updated_data).map(|(entry, data)|
                format!("UPDATE {} SET {} = {} WHERE {} AND {};",
                    table,
                    entry,
                    data,
                    predicate,
                    cond
                )
            ).collect::<Vec<_>>()
                .join("\n")
        );
        crate::logger::print(&query);
        self.conn.execute_async(
            query
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }
}