#[cfg(not(target_arch = "wasm32"))]
mod naive;
#[cfg(not(target_arch = "wasm32"))]
pub use naive::*;

#[cfg(target_arch = "wasm32")]
mod web_logger;
#[cfg(target_arch = "wasm32")]
pub use web_logger::*;

pub mod lib_logger;