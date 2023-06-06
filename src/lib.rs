pub mod db;
pub mod logger;

use db::crud::{DatabaseResult, DatabaseError};

use futures::executor::block_on;
use logger::watch;

pub struct Tag {
    pub name: String,
    pub desc: Option<String>,
}

pub trait MakeTag {
    fn get_name(&self) -> String;
    fn get_desc(&self) -> Option<String>;
    fn get_tag(&self) -> Tag;
}

impl MakeTag for Tag {
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_desc(&self) -> Option<String> {
        self.desc.clone()
    }

    fn get_tag(&self) -> Tag {
        Tag { name: self.get_name(), desc: self.get_desc() }
    }
}

impl MakeTag for String {
    fn get_name(&self) -> String {
        format!("'{}'", self)
    }

    fn get_desc(&self) -> Option<String> {
        None
    }

    fn get_tag(&self) -> Tag {
        Tag { name: format!("'{}'", self), desc: None }
    }
}

impl Tag {
    pub fn new(name: &str) -> Self {
        Tag { name: format!("'{}'", name), desc: None }
    }

    pub fn new_with_desc(name: &str, desc: Option<String>) -> Self {
        let desc = desc.unwrap_or_default();
        Tag {
            name: format!("'{}'", name),
            desc: if !desc.is_empty() { Some(desc) }
                else { None },
        }
    }
    /// Add a tag to the database.
    /// A series of tag/weight pairs can follow to initialize the mutual link weights.
    /// They will first be added to the database if not existing.
    /// The tags can be described by a struct implementing [`frictune::MakeTag`]
    /// trait (either a [`frictune::Tag`] or a [`String`] for now).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use std::collections::HashMap;
    /// futures::executor::block_on(async {
    ///     let mut conn = frictune::db::crud::Database::new("./sample.db").await.unwrap();
    ///     let sample = frictune::Tag { name: "'sample'".to_string(), desc: None };
    ///     sample.add_tag::<String>(&mut conn, &[]).await;
    ///     let sample2 = frictune::Tag { name: "'sample2'".to_string(), desc: None };
    ///     sample2.add_tag(&mut conn, &[(String::from("sample"), 0.4)]).await;
    ///     assert_eq!(frictune::Tag::query_relation(&mut conn, &sample2, &sample).await.unwrap(), 0.4);
    /// });
    /// ```
    pub async fn add_tag<T: MakeTag>(&self, db: &mut db::crud::Database, name_weight_pairs: &[(T, f32)]) -> Result<DatabaseResult, DatabaseError> {
        match if let Some(words) = self.desc.clone() {
                db.create("tags", &[String::from("tag_name"), String::from("info")], &[self.name.clone(), words]).await
            } else {
                db.create("tags", &[String::from("tag_name")], &[self.name.clone()]).await
            }
        {
            Ok(_) => { }
            Err(e) => {
                logger::warn(e.to_string());
                if let DatabaseError::UniqueViolation = e { }
                else { logger::warn(e.to_string()); panic!() }
            },
        }

        for (k, v) in name_weight_pairs {
            match if let Some(words) = k.get_desc().clone() {
                    db.create("tags", &[String::from("tag_name"), String::from("info")], &[k.get_name().clone(), words]).await
                } else {
                    db.create("tags", &[String::from("tag_name")], &[k.get_name().clone()]).await
                }
            {
                Ok(_) => { }
                Err(e) => {
                    logger::warn(e.to_string());
                    if let DatabaseError::UniqueViolation = e { }
                    else { logger::warn(e.to_string()); panic!() }
                },
            }
            logger::watch(self.link_tags(db, k, *v).await);
            self.auto_update_links(db).await;
        }

        // TODO: change the return value later
        Ok(DatabaseResult::Success("add_tag successful".into()))
    }

    /// The non-async version of `add_tag`
    pub fn add_sync<T: MakeTag>(&self, db: &mut db::crud::Database, name_weight_pairs: &[(T, f32)]) {
        block_on(async { watch(self.add_tag(db, name_weight_pairs).await) });
    }

    /// Updates the autonomous links between this tag and other tags.
    /// The autonomous link weight is a product of existent weights.
    /// 
    /// # Example (TODO)
    /// 
    pub async fn auto_update_links(&self, db: &mut db::crud::Database) {
        // TODO: a reverse-way propagation
        let affected_tags: Vec<(String, f32)> = match db.read(
            "relationship",
            &["tag2".to_string(), "weight".to_string()],
            &format!("tag1 = {}", self.name),
            ""
        ).await {
            Ok(things) => 
                things.get::<String>(0).into_iter().zip(
                    things.get::<f32>(1)
                ).collect(),
            Err(e) => { logger::warn(e.to_string()); panic!() }
        };

        for (tag, weight) in affected_tags {
            for (n, w) in match db.read(
                "relationship",
                &[String::from("*")],
                &format!("tag1 = '{}'", tag),
                ""
            ).await
            {
                Ok(things) => { if things.len() == 0 {
                        continue
                    } else {
                        things.get::<String>(0).into_iter().zip(
                            things.get::<f32>(2)
                        )
                    }
                }
                Err(e) => { logger::warn(e.to_string()); continue }
            } {
                let entries = [String::from("tag1"), String::from("tag2"), String::from("weight"), String::from("is_origin")];
                let data = [
                    self.name.clone(),
                    format!("'{}'", n), 
                    (w * weight).to_string(),
                    String::from("false")];
                logger::watch(db.update(
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

    /// > **WARNING**
    /// 
    /// The function does not recursively update all links, which
    /// means some links are not properly updated.
    pub async fn update_all_links(db: &mut db::crud::Database) {
        for name in match db.read(
            "tags",
            &["tag_name".to_string()],
            &format!("true"),
            ""
        ).await {
            Ok(vrow) => vrow.get::<String>(0),
            Err(e) => { logger::warn(e.to_string()); panic!() }
        } { Tag { name, desc: None }.auto_update_links(db).await; }
    }

    pub async fn force_update_all_links(db: &mut db::crud::Database) {
        logger::watch(db.delete("relationship", "is_origin", "false").await);
        Tag::update_all_links(db).await;
    }

    pub async fn modify_tag(&self, db: &mut db::crud::Database, desc: &str) -> Result<DatabaseResult, DatabaseError> {
        let entry = ["tag_name".to_string(), "info".to_string()];
        let data = [self.get_name(), desc.into()];
        db.update("tags", &entry, &data, &entry[1..], &data[1..], "true").await
    }

    pub fn mod_sync(&self, db: &mut db::crud::Database, desc: &str) {
        block_on(async { watch(self.modify_tag(db, desc).await) });
    }

    pub async fn remove_tag(&self, db: &mut db::crud::Database) -> Result<DatabaseResult, DatabaseError> {
        let res1 = db.delete("relationship", "tag1", &self.name).await;
        let res2 = db.delete("relationship", "tag2", &self.name).await;
        if res1.is_ok() && res2.is_ok() { db.delete("tags", "tag_name", &self.name).await }
        else { res1 }
    }

    pub fn rem_sync(&self, db: &mut db::crud::Database) {
        block_on(async { watch(self.remove_tag(db).await) });
    }

    /// link this tag to another tag with weight `ratio`. The `target`
    /// tag is a generic type.
    /// 
    /// > **WARNING**
    /// 
    /// This function does not check the tags's existence. Be sure to
    /// add them first.
    pub async fn link_tags<T: MakeTag>(&self, db: &mut db::crud::Database, target: &T, ratio: f32) -> Result<DatabaseResult, DatabaseError> {
        let entries = [String::from("tag1"), String::from("tag2"), String::from("weight"), String::from("is_origin")];
        let data = [self.name.clone(), format!("{}", target.get_name()), ratio.to_string(), String::from("true")];
        
        match db.update(
            "relationship", 
            &entries,
            &data,
            &entries[2..],
            &data[2..],
            "true"
        ).await
        {
            Ok(res) => { self.auto_update_links(db).await; Ok(res.into()) },
            Err(e) => Err(e)
        }
    }

    pub fn link_sync<T: MakeTag>(&self, db: &mut db::crud::Database, target: &T, ratio: f32) {
        block_on(async { watch(self.link_tags(db, target, ratio).await) });
    }

    // TODO: change the f32 to f64
    pub async fn query_relation<T1: MakeTag, T2: MakeTag>(db: &mut db::crud::Database, tag1: &T1, tag2: &T2) -> Option<f32> {
        match db.read("relationship",
            &[String::from("*")],
            &format!("tag1 = {} AND tag2 = {}", tag1.get_name(), tag2.get_name()),
            ""
        ).await {
            Ok(things) => {
                if things.len() != 1 { logger::warn(String::from("other than one queryed")); panic!() }
                things.get::<f32>(2).get(0).copied()
            },
            Err(e) => { logger::warn(e.to_string()); panic!() }
        }
    }

    pub fn query_sync<T1: MakeTag, T2: MakeTag>(db: &mut db::crud::Database, tag1: &T1, tag2: &T2) -> Option<f32> {
        block_on(async { Tag::query_relation(db, tag1, tag2).await })
    }

    pub async fn query_top_related(&self, db: &mut db::crud::Database) -> Vec<String> {
        match db.read(
            "relationship",
            &["tag2".into()],
            &format!("tag1 = {}", self.name),
            "ORDER BY weight"
        ).await {
            Ok(things) => 
                things.get::<String>(0),
            Err(e) => { logger::warn(e.to_string()); panic!() }
        }
    }

    pub fn qtr_sync(&self, db: &mut db::crud::Database) -> Vec<String> {
        block_on(async { self.query_top_related(db).await })
    }

    pub async fn query_desc(&self, db: &mut db::crud::Database) -> Option<String> {
        match db.read(
            "tags", 
            &[String::from("info")], 
            &format!("tag_name = {}", self.name),
            ""
        ).await {
            Ok(things) => {
                if things.len() != 1 { logger::warn(String::from("other than one queryed"));
                logger::warn(things.get::<String>(0).join("\n")); panic!() }
                things.get::<String>(0).get(0).cloned()
            }
            Err(e) => { logger::warn(e.to_string()); panic!() }
        }
    }

    pub fn qd_sync(&self, db: &mut db::crud::Database) -> Option<String> {
        block_on(async { self.query_desc(db).await })
    }

    pub fn qtrd(&self, db: &mut db::crud::Database) -> Vec<(String, Option<String>, Option<f32>)> {
        let tags = self.qtr_sync(db);
        tags.iter().map(|tag|
            (
                tag.into(),
                Tag::new(tag).qd_sync(db),
                Tag::query_sync(db, self, &Tag::new(tag))
            )).collect()
    }

    pub fn get_tags(db: &mut db::crud::Database) -> Vec<String> {
        block_on(async {
            db.read(
                "tags", 
                &[String::from("tag_name")], 
                &format!("true"),
                ""
            ).await
            .map(|thing| thing.get::<String>(0))
            .unwrap_or_default()
        })
    }
}