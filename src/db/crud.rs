
use core::panic;
use std::error::Error;


use sqlx::{SqliteConnection, Connection, migrate::MigrateDatabase, Executor, Row};

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
            is_origin INTEGER,
            CONSTRAINT relationship_id1_fk FOREIGN KEY (tag1) REFERENCES tags(tag_name),
            CONSTRAINT relationship_id2_fk FOREIGN KEY (tag2) REFERENCES tags(tag_name),
            CONSTRAINT relation_pk PRIMARY KEY (tag1, tag2)
        );");

        let result = connection
            .execute(query)
            .await.map_err(|err| {print!("err, {}", err); panic!() });

        Ok(Db { url: String::from(db_url) })
            
    }

    pub async fn create(&self, name: &str, data: Vec<String>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        sqlx::query(
            &(
                format!("INSERT INTO tags (tag_name) VALUES({});", name) +
                &data.into_iter().flat_map(|s|
                    format!("INSERT INTO relationship (tag1, tag2, weight) VALUES({}, {});", name, s).chars().collect::<Vec<_>>()
                ).collect::<String>()
            )
        ).execute(&mut SqliteConnection::connect(&self.url).await.unwrap())
            .await
    }

    pub async fn delete(&self) {}
    pub async fn read(&self, name: &str) -> String {
        let result = sqlx::query(
            &format!("SELECT * FROM relationship
                WHERE tag1 = {}", name)
        ).fetch_all(&mut SqliteConnection::connect(&self.url).await.unwrap())
            .await;

        match result {
            Ok(v) => { if v.is_empty() {
                    println!("empty row");
                    String::from("")
                } else {
                    v.into_iter()
                        .map(|s| s.get::<f32, usize>(2).to_string())
                        .collect()
                }
            }
            Err(e) => { println!("{}", e); panic!() }
        }
    }
    pub async fn update(&self) {}
}