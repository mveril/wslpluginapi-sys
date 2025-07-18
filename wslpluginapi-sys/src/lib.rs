#![cfg_attr(docsrs, feature(doc_auto_cfg))]
mod bindgen;
mod manual;
pub extern crate windows_sys;
pub use crate::bindgen::*;
pub use manual::*;
