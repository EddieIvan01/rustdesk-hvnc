#[cfg(target_os = "windows")]
extern crate winapi;
#[cfg(target_os = "windows")]
mod common;
#[cfg(target_os = "windows")]
mod display;
#[cfg(target_os = "windows")]
mod grab;
#[cfg(target_os = "windows")]
mod keyboard;
mod keycodes;
#[cfg(target_os = "windows")]
mod listen;
#[cfg(target_os = "windows")]
mod simulate;

#[cfg(target_os = "windows")]
pub use crate::windows::common::*;
#[cfg(target_os = "windows")]
pub use crate::windows::display::display_size;
#[cfg(target_os = "windows")]
pub use crate::windows::grab::{exit_grab, grab, set_event_popup, set_get_key_unicode};
#[cfg(target_os = "windows")]
pub use crate::windows::keyboard::Keyboard;
pub use crate::windows::keycodes::*;
#[cfg(target_os = "windows")]
pub use crate::windows::listen::listen;
#[cfg(target_os = "windows")]
pub use crate::windows::simulate::*;
