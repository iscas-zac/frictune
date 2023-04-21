pub fn warn(info: String) {
    println!("{}", info);
}

/// Print out error and exit.
pub fn rupt(info: &str) -> ! {
    println!("{}", info);
    panic!();
}

pub fn watch(result: Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>) {
    match result {
        Ok(res) => { warn("success".to_string()); },
        Err(e) => { warn(e.to_string()); }
    }
}

pub fn print(info: &str) {
    println!("{}", info);
}
