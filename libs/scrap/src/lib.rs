
#[macro_use]
extern crate cfg_if;
pub use hbb_common::libc;
#[cfg(dxgi)]
extern crate winapi;

pub use common::*;


#[cfg(dxgi)]
pub mod dxgi;


mod common;
