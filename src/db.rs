#[cfg(not(target_arch = "wasm32"))]
pub mod crud;
#[cfg(target_arch = "wasm32")]
pub mod gluesql;
#[cfg(target_arch = "wasm32")]
pub mod crud {
    pub use crate::db::gluesql::*;
}