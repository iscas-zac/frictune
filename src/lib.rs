pub mod db;
pub mod logger;
use std::collections::HashMap;

use futures::executor::block_on;
use logger::naive::watch;
use sqlx::Row;

pub struct Tag {
    pub name: String,
    pub desc: Option<String>,
}

impl Tag {
    pub fn new(name: &str) -> Self {
        Tag { name: format!("'{}'", name), desc: None }
    }
    /// Add a tag to the database.
    /// A series of tag/weight pairs can follow to initialize the mutual link weights.
    /// 
    /// # Examples
    /// 
    /// ```
    /// async {
    ///     let mut conn = db::crud::Db::new("./sample.db").await.unwrap();
    ///     let sample = frictune::Tag { name: "sample", desc: None };
    ///     sample.add_tag(db, HashMap::new()).await;
    ///     let sample2 = frictune::Tag { name: "sample2", desc: None };
    ///     sample2.add_tag(db, HashMap::from([(String::from("sample"), 0.4)]));
    ///     assert_eq!(sample2.query_relation(db).unwrap(), 0.4);
    /// }
    /// ```
    pub async fn add_tag(&self, db: &mut db::crud::Db, weights: HashMap<String, f32>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        match if let Some(words) = self.desc.clone() {
                db.create("tags", &[String::from("tag_name"), String::from("desc")], &[self.name.clone(), words]).await
            } else {
                db.create("tags", &[String::from("tag_name")], &[self.name.clone()]).await
            }
        {
            Ok(_) => { }
            Err(e) => {
                logger::naive::warn(e.to_string());
                if e.as_database_error().unwrap().is_unique_violation() { }
                else { panic!() }
            },
        }

        for (k, v) in weights {
            logger::naive::watch(self.link_tags(db, &k, v).await);
            self.auto_update_links(db).await;
        }

        // TODO: change the return value later
        Ok(sqlx::sqlite::SqliteQueryResult::default())
    }

    /// The non-async version of `add_tag`
    pub fn add_sync(&self, db: &mut db::crud::Db, weights: HashMap<String, f32>) {
        block_on(async { watch(self.add_tag(db, weights).await) });
    }

    /// Updates the autonomous links between tags
    pub async fn auto_update_links(&self, db: &mut db::crud::Db) {
        let affected_tags: Vec<(String, f32)> = match db.read(
            "relationship",
            &["tag2".to_string(), "weight".to_string()],
            &format!("tag1 = {}", self.name)
        ).await {
            Ok(vrow) => vrow.iter().map(|row|
                (row.get::<String, usize>(0), row.get::<f32, usize>(1))).collect(),
            Err(e) => { logger::naive::warn(e.to_string()); panic!() }
        };

        for (tag, weight) in affected_tags {
            for row in match db.read(
                "relationship",
                &[String::from("*")],
                &format!("tag1 = '{}'", tag)
            ).await
            {
                Ok(vrow) => { if vrow.is_empty() {
                        continue
                    } else {
                        vrow
                    }
                }
                Err(e) => { logger::naive::warn(e.to_string()); continue }
            } {
                let entries = [String::from("tag1"), String::from("tag2"), String::from("weight"), String::from("is_origin")];
                let data = [
                    self.name.clone(),
                    row.get::<String, usize>(1), 
                    (row.get::<f32, usize>(2) * weight).to_string(),
                    String::from("false")];
                logger::naive::watch(db.update(
                    "relationship", 
                    &entries,
                    &data,
                    &entries[2..],
                    &data[2..],
                    "excluded.weight > weight AND is_origin = false"
                ).await);
            };
        }
    }

    pub async fn update_all_links(db: &mut db::crud::Db) {
        for name in match db.read(
            "tags",
            &["tag_name".to_string()],
            &format!("true")
        ).await {
            Ok(vrow) => vrow.into_iter().map(|row|
                row.get::<String, usize>(0)),
            Err(e) => { logger::naive::warn(e.to_string()); panic!() }
        } { Tag { name, desc: None }.auto_update_links(db).await; }
    }

    pub async fn force_update_all_links(db: &mut db::crud::Db) {
        logger::naive::watch(db.delete("relationship", "is_origin", "false").await);
        Tag::update_all_links(db).await;
    }

    pub async fn remove_tag(&self, db: &mut db::crud::Db) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let res1 = db.delete("relationship", "tag1", &self.name).await;
        let res2 = db.delete("relationship", "tag2", &self.name).await;
        if res1.is_ok() && res2.is_ok() { db.delete("tags", "tag_name", &self.name).await }
        else { res1 }
    }

    pub fn rem_sync(&self, db: &mut db::crud::Db) {
        block_on(async { watch(self.remove_tag(db).await) });
    }

    pub async fn link_tags(&self, db: &mut db::crud::Db, target: &str, ratio: f32) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        let entries = [String::from("tag1"), String::from("tag2"), String::from("weight"), String::from("is_origin")];
        let data = [self.name.clone(), format!("'{}'", target), ratio.to_string(), String::from("true")];
        
        match db.update(
            "relationship", 
            &entries,
            &data,
            &entries[2..],
            &data[2..],
            "true"
        ).await
        {
            Ok(res) => { self.auto_update_links(db).await; Ok(res) },
            Err(e) => Err(e)
        }
    }

    pub fn link_sync(&self, db: &mut db::crud::Db, target: &str, ratio: f32) {
        block_on(async { watch(self.link_tags(db, target, ratio).await) });
    }

    pub async fn query_relation(db: &mut db::crud::Db, tag1: &str, tag2: &str) -> Option<f32> {
        match db.read("relationship",
            &[String::from("*")],
            &format!("tag1 = '{}' AND tag2 = '{}'", tag1, tag2)
        ).await {
            Ok(vrow) => { if vrow.is_empty() {
                    None
                } else {
                    if vrow.len() > 1 { logger::naive::warn(String::from("more than one queryed")); panic!() }
                    vrow.get(0).map(|row| row.get::<f32, usize>(2))
                }
            }
            Err(e) => { logger::naive::warn(e.to_string()); panic!() }
        }
    }

    pub fn query_sync(db: &mut db::crud::Db, tag1: &str, tag2: &str) -> Option<f32> {
        block_on(async { Tag::query_relation(db, tag1, tag2).await })
    }
}