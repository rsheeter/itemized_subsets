//! Exploratory hackery

pub(crate) mod error;
pub mod fallback_chain;

use fallback_chain::Family;

pub use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Run<'a> {
    pub family: &'a Family,
    pub start: usize,
    pub end: usize,
}
