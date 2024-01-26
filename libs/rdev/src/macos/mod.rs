#[cfg(target_os = "macos")]
mod common;
#[cfg(target_os = "macos")]
mod display;
#[cfg(target_os = "macos")]
mod grab;
#[cfg(target_os = "macos")]
mod keyboard;
mod keycodes;
#[cfg(target_os = "macos")]
mod listen;
#[cfg(target_os = "macos")]
mod simulate;
pub mod virtual_keycodes;

#[cfg(target_os = "macos")]
pub use crate::macos::common::{map_keycode, set_is_main_thread};
#[cfg(target_os = "macos")]
pub use crate::macos::display::display_size;
#[cfg(target_os = "macos")]
pub use crate::macos::grab::{exit_grab, grab};
#[cfg(target_os = "macos")]
pub use crate::macos::keyboard::Keyboard;
pub use crate::macos::keycodes::*;
#[cfg(target_os = "macos")]
pub use crate::macos::listen::listen;
#[cfg(target_os = "macos")]
pub use crate::macos::simulate::{
    set_keyboard_extra_info, set_mouse_extra_info, simulate, VirtualInput,
};
