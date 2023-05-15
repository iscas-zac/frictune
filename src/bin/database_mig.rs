use futures::executor::block_on;

fn main() {
    let mut args = std::env::args();
    args.next();
    let mut sqlite3_db = frictune::db::crud::Database::sync_new(
        &args.next().unwrap_or("./tags.sqlite3".to_owned())
    ).unwrap();
    let glue_url = &args.next().unwrap_or("./tags.gluesql".to_owned());
    let mut gluesql_db = frictune::db::gluesql::Database::sync_new(
        glue_url
    ).unwrap();
    block_on(async {
        let tags = sqlite3_db.read("tags", &["*".to_string()], "TRUE", "")
            .await.unwrap();
        let v_tag_name: Vec<String> = tags.get(0);
        let v_info: Vec<String> = tags.get(1);
        for (tag_name, info) in v_tag_name.into_iter().zip(v_info.into_iter()) {
            let entry = ["tag_name".into(), "info".into()];
            let data = [format!("'{}'", tag_name), format!("'{}'", info)];
            gluesql_db.update("tags", &entry, &data,
                &entry[1..], &data[1..], "TRUE")
                .await
                .unwrap();
        }
        let relationship = sqlite3_db.read("relationship", &["*".to_string()], "TRUE", "")
            .await.unwrap();
        let v_tag1: Vec<String> = relationship.get(0);
        let v_tag2: Vec<String> = relationship.get(1);
        let v_weight: Vec<f32> = relationship.get(2);
        let v_is_origin: Vec<bool> = relationship.get(3);
        for (tag1, tag2, weight, is_origin) in itertools::izip!(v_tag1, v_tag2, v_weight, v_is_origin) {
            let entry = ["tag1".into(), "tag2".into(), "weight".into(), "is_origin".into()];
            let data = [format!("'{}'", tag1), format!("'{tag2}'"),
                weight.to_string(), is_origin.to_string()];
            gluesql_db.update("relationship", &entry, &data,
                &entry[2..], &data[2..], "TRUE")
                .await
                .unwrap();
        }
        gluesql_db.save(glue_url).unwrap();
    });
}