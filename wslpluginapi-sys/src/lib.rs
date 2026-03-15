#![cfg_attr(docsrs, feature(doc_cfg))]
mod bindgen;
mod manual;
/// Re-export the [windows_sys] crate for use in higher-level crates
pub extern crate windows_sys;
pub use crate::bindgen::*;
pub use manual::*;
