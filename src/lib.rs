pub mod db;
pub mod logger;
use std::{vec, collections::HashMap};

use sqlx::Row;

pub struct Tag {
    pub name: String,
    pub desc: Option<String>,
}

impl Tag {
    pub async fn add_tag<'a>(&self, db: &db::crud::Db, weights: HashMap<String, f32>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        match if let Some(words) = self.desc.clone() {
                db.create("tags", vec![String::from("tag_name"), String::from("desc")], vec![self.name.clone(), words]).await
            } else {
                db.create("tags", vec![String::from("tag_name")], vec![self.name.clone()]).await
            }
        {
            Ok(msg) => {
                let mut res = Ok(msg); // TODO: remove this temporary return value
                for (k, v) in weights {
                    res = db.create("relationship", 
                        vec![String::from("tag1"), String::from("tag2"), String::from("weight"), String::from("is_origin")],
                        vec![self.name.clone(), k.clone(), v.to_string(), String::from("true")]).await
                }
                res
            }
            Err(e) => {
                logger::tui::warn(e.to_string());
                if e.as_database_error().unwrap().is_unique_violation() { Err(e) }
                else { panic!() }
            },
        }
    }

    pub async fn remove_tag(&self, db: &db::crud::Db) -> Result<Vec<sqlx::sqlite::SqliteRow>, sqlx::Error> {
        let res1 = db.delete("relationship", "tag1", &self.name).await;
        let res2 = db.delete("relationship", "tag2", &self.name).await;
        if res1.is_ok() && res2.is_ok() { db.delete("tags", "tag_name", &self.name).await }
        else { res1 }
    }

    pub async fn link_tags<'a>(&self, db: &db::crud::Db, target: &Tag, ratio: f32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let entries = [String::from("tag1"), String::from("tag2"), String::from("weight"), String::from("is_origin")];
        let data = [self.name.clone(), target.name.clone(), ratio.to_string(), String::from("true")];
        db.update(
            "relationship", 
            &entries,
            &data,
            &entries[2..],
            &data[2..],
            "true"
        ).await
    }

    pub async fn query_relation(db: &db::crud::Db, tag1: &str, tag2: &str) -> Option<f32> {
        match db.read("relationship",
            vec![String::from("*")],
            &format!("tag1 = {} AND tag2 = {}", tag1, tag2)
        ).await {
            Ok(vrow) => { if vrow.is_empty() {
                    None
                } else {
                    if vrow.len() > 1 { logger::tui::warn(String::from("more than one queryed")); panic!() }
                    vrow.get(0).map(|row| row.get::<f32, usize>(2))
                }
            }
            Err(e) => { println!("{}", e); panic!() }
        }
        
    }
}