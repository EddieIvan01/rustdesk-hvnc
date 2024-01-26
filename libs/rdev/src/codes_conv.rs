#[cfg(any(target_os = "windows", target_os = "macos"))]
use crate::linux::code_from_key as linux_code_from_keycode;
#[cfg(target_os = "linux")]
use crate::linux::key_from_code as linux_keycode_from_code;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use crate::macos::{code_from_key as macos_code_from_keycode, virtual_keycodes::*};
#[cfg(target_os = "macos")]
use crate::macos::{key_from_code as macos_keycode_from_code, map_keycode};
#[cfg(target_os = "windows")]
use crate::windows::key_from_scancode as win_key_from_scancode;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use crate::windows::scancode_from_key as win_scancode_from_key;
use crate::{Key, KeyCode};

macro_rules! conv_keycodes {
    ($fnname:ident, $key_from_code:ident, $code_from_key:ident) => {
        pub fn $fnname(code: u32) -> Option<KeyCode> {
            let key = $key_from_code(code as _);
            match key {
                Key::Unknown(..) => None,
                Key::RawKey(..) => None,
                _ => $code_from_key(key).map(|c| c as KeyCode),
            }
        }
    };
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[allow(non_upper_case_globals)]
fn macos_iso_code_from_keycode(key: Key) -> Option<KeyCode> {
    match macos_code_from_keycode(key)? {
        kVK_ISO_Section => Some(kVK_ANSI_Grave),
        kVK_ANSI_Grave => Some(kVK_ISO_Section),
        code => Some(code as _),
    }
}

#[cfg(target_os = "macos")]
#[allow(non_upper_case_globals)]
fn macos_keycode_from_code_(code: KeyCode) -> Key {
    macos_keycode_from_code(map_keycode(code))
}

#[cfg(target_os = "windows")]
conv_keycodes!(
    win_scancode_to_linux_code,
    win_key_from_scancode,
    linux_code_from_keycode
);
#[cfg(target_os = "windows")]
conv_keycodes!(
    win_scancode_to_macos_code,
    win_key_from_scancode,
    macos_code_from_keycode
);
#[cfg(target_os = "windows")]
// From Win scancode to MacOS keycode(ISO Layout)
conv_keycodes!(
    win_scancode_to_macos_iso_code,
    win_key_from_scancode,
    macos_iso_code_from_keycode
);
#[cfg(target_os = "linux")]
conv_keycodes!(
    linux_code_to_win_scancode,
    linux_keycode_from_code,
    win_scancode_from_key
);
#[cfg(target_os = "linux")]
conv_keycodes!(
    linux_code_to_macos_code,
    linux_keycode_from_code,
    macos_code_from_keycode
);
#[cfg(target_os = "linux")]
// From Linux scancode to MacOS keycode(ISO Layout)
conv_keycodes!(
    linux_code_to_macos_iso_code,
    linux_keycode_from_code,
    macos_iso_code_from_keycode
);
#[cfg(target_os = "macos")]
conv_keycodes!(
    macos_code_to_win_scancode,
    macos_keycode_from_code_,
    win_scancode_from_key
);
#[cfg(target_os = "macos")]
conv_keycodes!(
    macos_code_to_linux_code,
    macos_keycode_from_code_,
    linux_code_from_keycode
);
