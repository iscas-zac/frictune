
use core::panic;
use std::error::Error;


use futures::executor::block_on;
use sqlx::{SqliteConnection, Connection, migrate::MigrateDatabase, Executor};

pub struct Database {
    conn: SqliteConnection,
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    UniqueViolation,
    SqlxError(sqlx::Error),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UniqueViolation => f.write_str("unique violation"),
            Self::SqlxError(e) => f.write_str(e.to_string().as_str()),
        }
    }
}

pub enum DatabaseResult {
    Success(String),
    Things(Vec<sqlx::sqlite::SqliteRow>),
}

impl From<Vec<sqlx::sqlite::SqliteRow>> for DatabaseResult {
    fn from(value: Vec<sqlx::sqlite::SqliteRow>) -> Self {
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

    pub fn get<'r, T>(&'r self, index: usize) -> Vec<T>
    where
        T: sqlx::Decode<'r, sqlx::sqlite::Sqlite> + sqlx::Type<sqlx::sqlite::Sqlite>,
    {
        match self {
            Self::Success(_) => vec![],
            Self::Things(v) =>
                v.iter().map(|row|
                    sqlx::Row::get::<T, usize>(row, index)
                ).collect()
        }
    }
}

impl From<sqlx::sqlite::SqliteQueryResult> for DatabaseResult {
    fn from(value: sqlx::sqlite::SqliteQueryResult) -> Self {
        DatabaseResult::Success(format!("last inserted row is {}",
            value.last_insert_rowid()))
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::Database(dbe)
                if dbe.is_unique_violation() => DatabaseError::UniqueViolation,
            other => DatabaseError::SqlxError(other),

        }
    }
}

impl Database {
    pub fn sync_new(db_url: &str) -> anyhow::Result<Database> {
        block_on(async { Database::new(db_url).await } )
    }

    pub async fn new(db_url: &str) -> anyhow::Result<Database> {
        if !sqlx::Sqlite::database_exists(&db_url).await? {
            sqlx::Sqlite::create_database(&db_url).await?;
        }

        let mut conn = SqliteConnection::connect(&db_url).await?;

        let query = sqlx::query("CREATE TABLE IF NOT EXISTS tags
        (
            tag_name    TEXT PRIMARY KEY NOT NULL,
            info     TEXT
        );
        CREATE TABLE IF NOT EXISTS relationship
        (
            tag1 TEXT NOT NULL,
            tag2 TEXT NOT NULL,
            weight REAL,
            is_origin INTEGER DEFAULT false,
            CONSTRAINT relationship_id1_fk FOREIGN KEY (tag1) REFERENCES tags(tag_name),
            CONSTRAINT relationship_id2_fk FOREIGN KEY (tag2) REFERENCES tags(tag_name),
            CONSTRAINT relation_pk PRIMARY KEY (tag1, tag2)
        );");

        match conn
            .execute(query)
            .await {
            Ok(_) => Ok(Database { conn }),
            Err(e) => { print!("err, {}", e); panic!() }
        }
    }
    
    pub async fn create(&mut self, table: &str, entry: &[String], data: &[String]) -> Result<DatabaseResult, DatabaseError> {
        sqlx::query(
            &format!("INSERT INTO {} ({}) VALUES({});", table, entry.join(", "), data.join(", "))
        ).execute(&mut self.conn)
            .await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }

    pub async fn delete(&mut self, table: &str, entry: &str, data: &str) -> Result<DatabaseResult, DatabaseError> {
        sqlx::query(
            &format!("DELETE FROM {}
                WHERE {} = {}", table, entry, data)
        ).execute(&mut self.conn)
            .await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }

    pub async fn read(&mut self, table: &str, entry: &[String], cond: &str, opts: &str) -> Result<DatabaseResult, DatabaseError> {
        sqlx::query(
            &format!("SELECT {} FROM {}
                WHERE {} {}", entry.join(", "), table, cond, opts)
        ).fetch_all(&mut self.conn)
            .await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }
    
    pub async fn update(&mut self, table: &str, entry: &[String], data: &[String],
            updated_entry: &[String], updated_data: &[String], cond: &str) -> Result<DatabaseResult, DatabaseError> {
        sqlx::query(
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
        ).execute(&mut self.conn)
            .await
            .map(|value| DatabaseResult::from(value))
            .map_err(|value| DatabaseError::from(value))
    }
}