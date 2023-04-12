
use core::panic;
use std::error::Error;


use sqlx::{SqliteConnection, Connection, migrate::MigrateDatabase, Executor};

pub struct Db {
    url: String,
}

impl Db {
    pub async fn new() -> Result<Db, Box<dyn Error>> {
        let db_url = "./tags.db";
        if !sqlx::Sqlite::database_exists(&db_url).await? {
            sqlx::Sqlite::create_database(&db_url).await?;
        }

        let mut connection = SqliteConnection::connect(&db_url).await?;

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

        match connection
            .execute(query)
            .await {
            Ok(_) => Ok(Db { url: String::from(db_url) }),
            Err(e) => { print!("err, {}", e); panic!() }
        }    
    }
    // TODO: API update with [] / &[] instead of vec
    pub async fn create(&self, table: &str, entry: Vec<String>, data: Vec<String>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query(
            &format!("INSERT INTO {} ({}) VALUES({});", table, entry.join(", "), data.join(", "))
        ).execute(&mut SqliteConnection::connect(&self.url).await.unwrap())
            .await
    }

    pub async fn delete(&self, table: &str, entry: &str, data: &str) -> Result<Vec<sqlx::sqlite::SqliteRow>, sqlx::Error> {
        sqlx::query(
            &format!("DELETE FROM {}
                WHERE {} = {}", table, entry, data)
        ).fetch_all(&mut SqliteConnection::connect(&self.url).await.unwrap())
            .await
    }

    pub async fn read(&self, table: &str, entry: Vec<String>, cond: &str) -> Result<Vec<sqlx::sqlite::SqliteRow>, sqlx::Error> {
        sqlx::query(
            &format!("SELECT {} FROM {}
                WHERE {}", entry.join(", "), table, cond)
        ).fetch_all(&mut SqliteConnection::connect(&self.url).await.unwrap())
            .await
    }
    
    pub async fn update(&self, table: &str, entry: &[String], data: &[String],
            updated_entry: &[String], updated_data: &[String], cond: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
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
        ).execute(&mut SqliteConnection::connect(&self.url).await.unwrap())
            .await
    }
}