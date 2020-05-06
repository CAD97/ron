extern crate serde; // fix the error messages D:

#[macro_use]
mod macros;

pub mod error;
pub mod ser;
pub mod value;

pub mod parse;
pub mod pretty;

pub use self::{
    error::{Error, Result},
    ser::{to_string, to_vec, to_writer},
    value::{to_value, Value},
};

#[doc(hidden)]
pub use std;
