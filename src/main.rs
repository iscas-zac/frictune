use std::collections::HashMap;

use frictune::{db, Tag};
use futures::executor::block_on;

fn main() {
    block_on(async {
        let conn = db::crud::Db::new().await.unwrap();
        let a = frictune::Tag{ name: String::from("7"), desc: None };
        let b = frictune::Tag{ name: String::from("8"), desc: None };
        match a.add_tag(&conn, HashMap::new()).await {
            Ok(res) => {println!("{:?}", res)},
            Err(e) => {println!("{}", e)},
        }
        match b.add_tag(&conn, HashMap::from([(String::from("7"), 0.4)])).await {
            Ok(res) => {println!("{:?}", res)},
            Err(e) => {println!("{}", e)},
        }
        let c = frictune::Tag{ name: String::from("9"), desc: None };
        match c.link_tags(&conn, &a, 0.7).await {
            Ok(res) => {println!("{:?}", res)},
            Err(e) => {println!("{}", e)},
        }
        println!("{}", Tag::query_relation(&conn, "9", "7").await.unwrap());

        match c.remove_tag(&conn).await {
            Ok(_) => {},
            Err(e) => {println!("{}", e)},
        }
    });
}
