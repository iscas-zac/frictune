use frictune::{logger::watch, db, Tag};
use futures::executor::block_on;

fn test_primitive_apis() {
        block_on(async {
        let mut conn = db::crud::Database::new("./tags.db").await.unwrap();
        watch(conn.delete("relationship", "1", "1").await);
        let a = frictune::Tag{ name: String::from("7"), desc: None };
        let b = frictune::Tag{ name: String::from("8"), desc: None };
        watch(a.add_tag::<String>(&mut conn, &[]).await);
        watch(b.add_tag(&mut conn, &[(String::from("7"), 0.4)]).await);
        let c = frictune::Tag{ name: String::from("9"), desc: None };
        watch(c.add_tag::<String>(&mut conn, &[]).await);
        watch(a.link_tags(&mut conn, &c.name, 0.8).await);
        b.auto_update_links(&mut conn).await;
        let d = frictune::Tag{ name: String::from("10"), desc: None };
        watch(d.add_tag(&mut conn, &[(String::from("7"), 0.4)]).await);
        watch(b.link_tags(&mut conn, &d.name, 0.2).await);
        watch(d.link_tags(&mut conn, &b.name, 0.9).await);
        Tag::update_all_links(&mut conn).await;
        //println!("{}", Tag::query_relation(&mut conn, "7", "8").await.unwrap());

        // match c.remove_tag(&mut conn).await {
        //     Ok(_) => {},
        //     Err(e) => {println!("{}", e)},
        // }
    });
}