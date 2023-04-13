pub fn warn(info: String) {
    println!("{}", info);
}

pub fn watch(result: Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>) {
    match result {
        Ok(res) => { warn("success".to_string()); },
        Err(e) => { warn(e.to_string()); }
    }
}