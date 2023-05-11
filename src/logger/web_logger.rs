use crate::db::crud::{DatabaseResult, DatabaseError};

pub fn warn(info: String) {
    log::warn!("{}", info);
}

/// Print out error and exit.
pub fn rupt(info: &str) -> ! {
    log::error!("{}", info);
    panic!();
}

pub fn watch(result: Result<DatabaseResult, DatabaseError>) {
    match result {
        Ok(_) => { warn("success".to_string()); },
        Err(e) => { warn(e.to_string()); }
    }
}

pub fn print(info: &str) {
    log::info!("{}", info);
}
