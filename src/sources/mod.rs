use crate::model::Entry;
use anyhow::Result;

pub trait Source {
    fn scan(&self) -> Result<Vec<Entry>>;
}

pub mod desktop;
pub mod bin;
pub mod history;
pub mod scripts;
