#[cfg(not(target_arch = "wasm32"))]
pub mod crud;
#[cfg(target_arch = "wasm32")]
pub use db::gluesql as crud;