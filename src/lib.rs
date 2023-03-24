mod db;
use std::error::Error;

pub use db::*;

pub struct Tag;

impl Tag {
    pub fn add_tag<'a>(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn remove_tag(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn link_tags<'a>(&self, target: &Tag, ratio: f32) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    pub fn query_tags(&self) -> std::slice::Iter<'static, Tag> {
        let a = [Tag].iter();
        a
    }
}