pub use si_trace_print::*;

pub fn warn(info: String) {
    pf2ñ!("{}", info);
}

/// Print out error and exit.
pub fn rupt(info: &str) -> ! {
    pfn!("{}", info);
    panic!();
}

pub fn watch(result: Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>) {
    match result {
        Ok(_res) => { warn("success".to_string()); },
        Err(e) => { warn(e.to_string()); }
    }
}

pub fn print(info: &str) {
    pfn!("{}", info);
}
