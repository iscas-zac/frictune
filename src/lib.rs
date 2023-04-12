pub mod db;
use std::error::Error;

pub struct Tag {
    pub name: String,
}

#[macro_export]
macro_rules! add_tag {
    ( $src: expr, $db: expr $(, $tag: expr, $weight: expr)* ) => {
        {
            let mut temp_vec = Vec::new();
            $(temp_vec.push(format!("{}, {}", $tag, $weight));)*
            $src.add_tag($db, temp_vec).await
        }
    }
}

impl Tag {
    pub async fn add_tag<'a>(&self, db: &db::crud::Db, weights: Vec<String>) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
        db.create(&self.name, weights).await
    }

    pub fn remove_tag(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn link_tags<'a>(&self, target: &Tag, ratio: f32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub async fn query_tags(db: &db::crud::Db, tag: &str) -> String/*std::slice::Iter<'static, Tag>*/ {
        db.read(tag).await
    }
}