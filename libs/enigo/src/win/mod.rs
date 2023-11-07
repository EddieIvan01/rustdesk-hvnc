mod win_impl;

pub mod keycodes;
pub use self::win_impl::{Enigo, ENIGO_INPUT_EXTRA_VALUE};

pub use win_impl::get_gbl_wnd;