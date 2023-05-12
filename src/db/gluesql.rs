use gluesql::{sled_storage::SledStorage, prelude::{MemoryStorage, Glue, Payload, Row, DataType, Value}, core::{executor::ValidateError, result}};
use serde::{Serialize, Deserialize};
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
                Payload::Select { labels, rows } => 
                    [acc, rows].concat(),
                _ => acc,
            }
        ))
    }
}

impl From<gluesql::core::result::Error> for DatabaseError {
    fn from(value: gluesql::core::result::Error) -> Self {
        match value {
            gluesql::core::result::Error::Validate(ValidateError::DuplicateEntryOnPrimaryKeyField(k))
                => DatabaseError::UniqueViolation,
            other => DatabaseError::GlueError(other),
        }
    }
}

impl Database {
    // pub fn sync_new(db_url: &str) -> anyhow::Result<Database> {
    //     let storage = SledStorage::new(db_url)?;
    //     let mut conn = Glue::new(storage);
    //     conn.execute("CREATE TABLE IF NOT EXISTS tags
    //     (
    //         tag_name    TEXT PRIMARY KEY NOT NULL,
    //         info     TEXT
    //     );
    //     CREATE TABLE IF NOT EXISTS relationship
    //     (
    //         tag1 TEXT NOT NULL,
    //         tag2 TEXT NOT NULL,
    //         weight REAL,
    //         is_origin INTEGER DEFAULT false,
    //         CONSTRAINT relationship_id1_fk FOREIGN KEY (tag1) REFERENCES tags(tag_name),
    //         CONSTRAINT relationship_id2_fk FOREIGN KEY (tag2) REFERENCES tags(tag_name),
    //         CONSTRAINT relation_pk PRIMARY KEY (tag1, tag2)
    //     );")?;
    //     Ok(Database { conn })
    // }

    pub fn deser_new(content: &[u8]) -> anyhow::Result<Database> {
        let storage: MemoryStorage = bincode::deserialize(&content[..])?;
        let mut conn = Glue::new(storage);
        conn.execute("CREATE TABLE IF NOT EXISTS tags
        (
            tag_name    TEXT PRIMARY KEY NOT NULL,
            info     TEXT
        );
        CREATE TABLE IF NOT EXISTS relationship
        (
            tag1 TEXT NOT NULL,
            tag2 TEXT NOT NULL,
            weight DECIMAL,
            is_origin INTEGER DEFAULT false,
            CONSTRAINT relationship_id1_fk FOREIGN KEY (tag1) REFERENCES tags(tag_name),
            CONSTRAINT relationship_id2_fk FOREIGN KEY (tag2) REFERENCES tags(tag_name),
            CONSTRAINT relation_pk PRIMARY KEY (tag1, tag2)
        );")?;
        Ok(Database { conn })
    }

    pub async fn create(&mut self, table: &str, entry: &[String], data: &[String]) -> Result<DatabaseResult, DatabaseError> {
        self.conn.execute_async(
            &format!("INSERT INTO {} ({}) VALUES({});", table, entry.join(", "), data.join(", "))
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }

    pub async fn delete(&mut self, table: &str, entry: &str, data: &str) -> Result<DatabaseResult, DatabaseError> {
        self.conn.execute_async(
            &format!("DELETE FROM {}
                WHERE {} = {}", table, entry, data)
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }

    pub async fn read(&mut self, table: &str, entry: &[String], cond: &str, opts: &str) -> Result<DatabaseResult, DatabaseError> {
        self.conn.execute_async(
            &format!("SELECT {} FROM {}
                WHERE {} {}", entry.join(", "), table, cond, opts)
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }
    
    pub async fn update(&mut self, table: &str, entry: &[String], data: &[String],
            updated_entry: &[String], updated_data: &[String], cond: &str) -> Result<DatabaseResult, DatabaseError> {
        self.conn.execute_async(
            &format!("INSERT INTO {} ({}) VALUES ({})
            ON CONFLICT DO
            UPDATE SET {}
            WHERE {};",
                table,
                entry.join(", "),
                data.join(", "),
                updated_entry.iter().zip(updated_data.iter()).map(|(e, d)|
                    format!("{} = {}", e, d)).collect::<Vec<_>>().join(", "),
                cond
            )
        ).await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }
}