use std::collections::HashMap;

use frictune::{db, Tag, logger::tui::watch};
use futures::executor::block_on;

fn main() {
    block_on(async {
        let mut conn = db::crud::Db::new("./tags.db").await.unwrap();
        watch(conn.delete("relationship", "1", "1").await);
        let a = frictune::Tag{ name: String::from("7"), desc: None };
        let b = frictune::Tag{ name: String::from("8"), desc: None };
        watch(a.add_tag(&mut conn, HashMap::new()).await);
        watch(b.add_tag(&mut conn, HashMap::from([(String::from("7"), 0.4)])).await);
        let c = frictune::Tag{ name: String::from("9"), desc: None };
        watch(c.add_tag(&mut conn, HashMap::new()).await);
        watch(a.link_tags(&mut conn, &c, 0.8).await);
        b.auto_update_links(&mut conn).await;
        let d = frictune::Tag{ name: String::from("10"), desc: None };
        watch(d.add_tag(&mut conn, HashMap::from([(String::from("7"), 0.4)])).await);
        watch(b.link_tags(&mut conn, &d, 0.2).await);
        watch(d.link_tags(&mut conn, &b, 0.9).await);
        Tag::update_all_links(&mut conn).await;
        //println!("{}", Tag::query_relation(&mut conn, "7", "8").await.unwrap());

        // match c.remove_tag(&mut conn).await {
        //     Ok(_) => {},
        //     Err(e) => {println!("{}", e)},
        // }
    });
}
