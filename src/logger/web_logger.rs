use crate::db::crud::{DatabaseResult, DatabaseError};

pub fn warn(info: String) {
    //log::warn!("{}", info);
    web_sys::console::log_2(&"[WARNING] ".into(), &info.into());
}

/// Print out error and exit.
pub fn rupt(info: &str) -> ! {
    //log::error!("{}", info);
    web_sys::console::log_2(&"[ERROR] ".into(), &info.into());
    panic!();
}

pub fn watch(result: Result<DatabaseResult, DatabaseError>) {
    match result {
        Ok(_) => { warn("success".to_string()); },
        Err(e) => { warn(e.to_string()); }
    }
}

pub fn print(info: &str) {
    //log::info!("{}", info);
    web_sys::console::log_2(&"[INFO] ".into(), &info.into());
}
