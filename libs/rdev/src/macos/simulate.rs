use super::virtual_keycodes::{
    kVK_DownArrow, kVK_End, kVK_ForwardDelete, kVK_Help, kVK_Home, kVK_LeftArrow, kVK_PageDown,
    kVK_PageUp, kVK_RightArrow, kVK_UpArrow,
};
use crate::macos::{common::CGEventSourceKeyState, keycodes::code_from_key};
use crate::rdev::{Button, EventType, RawKey, SimulateError};
use core_graphics::{
    event::{
        CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton,
        EventField, ScrollEventUnit,
    },
    event_source::{CGEventSource, CGEventSourceStateID},
    geometry::CGPoint,
};
use std::convert::TryInto;

static mut MOUSE_EXTRA_INFO: i64 = 0;
static mut KEYBOARD_EXTRA_INFO: i64 = 0;

pub fn set_mouse_extra_info(extra: i64) {
    unsafe { MOUSE_EXTRA_INFO = extra }
}

pub fn set_keyboard_extra_info(extra: i64) {
    unsafe { KEYBOARD_EXTRA_INFO = extra }
}

#[allow(non_upper_case_globals)]
fn workaround_fn(event: CGEvent, keycode: CGKeyCode) -> CGEvent {
    match keycode {
        kVK_Help | kVK_ForwardDelete | kVK_Home | kVK_End | kVK_PageDown | kVK_PageUp => {
            let flags = event.get_flags();
            event.set_flags(flags & (!(CGEventFlags::CGEventFlagSecondaryFn)));
        }
        kVK_UpArrow | kVK_DownArrow | kVK_LeftArrow | kVK_RightArrow => {
            let flags = event.get_flags();
            event.set_flags(
                flags
                    & (!(CGEventFlags::CGEventFlagSecondaryFn
                        | CGEventFlags::CGEventFlagNumericPad)),
            );
        }
        _ => {}
    }
    event
}

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CGEventSource,
) -> Option<CGEvent> {
    match event_type {
        EventType::KeyPress(key) => match key {
            crate::Key::RawKey(rawkey) => {
                if let RawKey::MacVirtualKeycode(keycode) = rawkey {
                    CGEvent::new_keyboard_event(source, *keycode as _, true)
                        .and_then(|event| Ok(workaround_fn(event, *keycode)))
                        .ok()
                } else {
                    None
                }
            }
            _ => {
                let code = code_from_key(*key)?;
                CGEvent::new_keyboard_event(source, code as _, true)
                    .and_then(|event| Ok(workaround_fn(event, code as _)))
                    .ok()
            }
        },
        EventType::KeyRelease(key) => match key {
            crate::Key::RawKey(rawkey) => {
                if let RawKey::MacVirtualKeycode(keycode) = rawkey {
                    CGEvent::new_keyboard_event(source, *keycode as _, false)
                        .and_then(|event| Ok(workaround_fn(event, *keycode)))
                        .ok()
                } else {
                    None
                }
            }
            _ => {
                let code = code_from_key(*key)?;
                CGEvent::new_keyboard_event(source, code as _, false)
                    .and_then(|event| Ok(workaround_fn(event, code as _)))
                    .ok()
            }
        },
        EventType::ButtonPress(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseDown,
                Button::Right => CGEventType::RightMouseDown,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                source,
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
            .ok()
        }
        EventType::ButtonRelease(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseUp,
                Button::Right => CGEventType::RightMouseUp,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                source,
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
            .ok()
        }
        EventType::MouseMove { x, y } => {
            let point = CGPoint { x: (*x), y: (*y) };
            CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left)
                .ok()
        }
        EventType::Wheel { delta_x, delta_y } => {
            let wheel_count = 2;
            CGEvent::new_scroll_event(
                source,
                ScrollEventUnit::PIXEL,
                wheel_count,
                (*delta_y).try_into().ok()?,
                (*delta_x).try_into().ok()?,
                0,
            )
            .ok()
        }
    }
}

unsafe fn convert_native(event_type: &EventType) -> Option<CGEvent> {
    // https://developer.apple.com/documentation/coregraphics/cgeventsourcestateid#:~:text=kCGEventSourceStatePrivate
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    convert_native_with_source(event_type, source)
}

unsafe fn get_current_mouse_location() -> Option<CGPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    let event = CGEvent::new(source).ok()?;
    Some(event.location())
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    unsafe {
        if let Some(cg_event) = convert_native(event_type) {
            cg_event.set_integer_value_field(EventField::EVENT_SOURCE_USER_DATA, MOUSE_EXTRA_INFO);
            cg_event.post(CGEventTapLocation::HID);
            Ok(())
        } else {
            Err(SimulateError)
        }
    }
}

pub struct VirtualInput {
    source: CGEventSource,
    tap_loc: CGEventTapLocation,
}

impl VirtualInput {
    pub fn new(state_id: CGEventSourceStateID, tap_loc: CGEventTapLocation) -> Result<Self, ()> {
        Ok(Self {
            source: CGEventSource::new(state_id)?,
            tap_loc,
        })
    }

    pub fn simulate(&self, event_type: &EventType) -> Result<(), SimulateError> {
        unsafe {
            if let Some(cg_event) = convert_native_with_source(event_type, self.source.clone()) {
                cg_event.post(self.tap_loc);
                Ok(())
            } else {
                Err(SimulateError)
            }
        }
    }

    // keycode is defined in rdev::macos::virtual_keycodes
    pub fn get_key_state(state_id: CGEventSourceStateID, keycode: CGKeyCode) -> bool {
        unsafe { CGEventSourceKeyState(state_id, keycode) }
    }
}
