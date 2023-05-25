//! The abstraction of CRUD operations.
//!
//! This module is used to abstract the CRUD operations. The table has two default
//! tables named 'tag' and 'relationship'. 'Tag' table has a 'tag_name' field and an
//! optional 'info' field (which is used to store the http link for now). 'Relationship'
//! table has 'tag1', 'tag2', 'weight' and 'is_origin' fields. The 'weight' is a 0 ~ 1
//! float number, and 'is_origin' is used in inner operations.
//!
//! # Example
//!
//! ```
//! use frictune::db::crud::*;
//!
//! fn open_db() {
//!     let mut db = Database::sync_new("./tags.db").unwrap();
//!     assert!(matches!(db, Database));
//! }
//! ```
//!
//! # Note
//!
//! This module has both wasm and native features, but they are conflicted.
//!

#[cfg(not(target_arch = "wasm32"))]
pub mod crud;

pub mod gluesql;
#[cfg(target_arch = "wasm32")]
pub mod crud {
    pub use crate::db::gluesql::*;
}