use frictune::{db, Tag};
use futures::executor::block_on;

fn main() {
    block_on(async {
        let conn = db::crud::Db::new().await.unwrap();
        let a = frictune::Tag{ name: String::from("3") };
        let b = frictune::Tag{ name: String::from("2") };
        match a.add_tag(&conn, Vec::new()).await {
            Ok(res) => {println!("{:?}", res)},
            Err(e) => {println!("{}", e)},
        }
        match b.add_tag(&conn, vec![String::from("3, 0.4")]).await {
            Ok(res) => {println!("{:?}", res)},
            Err(e) => {println!("{}", e)},
        }
        //frictune::add_tag!(b, &conn, "3", 0.4);
        println!("1111111111111111111\n\n\n\n");
        println!("{}", Tag::query_tags(&conn, "2").await)
    });
}
