#[cfg(target_os = "linux")]
extern crate libc;
#[cfg(target_os = "linux")]
extern crate x11;
#[cfg(target_os = "linux")]
mod common;
#[cfg(target_os = "linux")]
mod display;
#[cfg(target_os = "linux")]
mod grab;
#[cfg(target_os = "linux")]
mod keyboard;
mod keycodes;
#[cfg(target_os = "linux")]
mod listen;
#[cfg(target_os = "linux")]
mod simulate;

#[cfg(target_os = "linux")]
pub use crate::linux::display::display_size;
#[cfg(target_os = "linux")]
pub use crate::linux::grab::{disable_grab, enable_grab, exit_grab_listen, start_grab_listen};
#[cfg(target_os = "linux")]
pub use crate::linux::keyboard::Keyboard;
pub use crate::linux::keycodes::*;
#[cfg(target_os = "linux")]
pub use crate::linux::listen::listen;
#[cfg(target_os = "linux")]
pub use crate::linux::simulate::{simulate, simulate_char, simulate_unicode};
