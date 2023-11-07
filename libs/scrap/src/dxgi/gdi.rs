use std::mem;
use std::ptr;
use std::{ffi::c_void, mem::size_of};

use hbb_common::futures_util::StreamExt;
use winapi::um::processthreadsapi::CreateProcessA;
use winapi::um::processthreadsapi::PROCESS_INFORMATION;
use winapi::um::processthreadsapi::STARTUPINFOA;
use winapi::um::sysinfoapi::GetVersionExA;
use winapi::um::wincrypt::CERT_STORE_CTRL_COMMIT;
use winapi::um::winnt::OSVERSIONINFOA;
use winapi::um::winuser::GetDesktopWindow;
use winapi::um::winuser::GetWindowDC;
use winapi::um::winuser::GetWindowInfo;
use winapi::um::winuser::GetWindowLongA;
use winapi::um::winuser::GetWindowTextA;
use winapi::um::winuser::IsWindowVisible;
use winapi::um::winuser::SetWindowLongA;
use winapi::um::winuser::GWL_EXSTYLE;
use winapi::um::winuser::GW_HWNDPREV;
use winapi::um::winuser::PW_CLIENTONLY;
use winapi::um::winuser::WINDOWINFO;
use winapi::um::winuser::WS_EX_COMPOSITED;
use winapi::um::winuser::{GetWindowRect, PrintWindow};
use winapi::{
    shared::windef::{HBITMAP, HDC, HDESK, HWND, RECT},
    um::{
        errhandlingapi::GetLastError,
        wingdi::{
            BitBlt,
            CreateCompatibleBitmap,
            CreateCompatibleDC,
            CreateDCW,
            DeleteDC,
            DeleteObject,
            GetDIBits,
            SelectObject,
            BITMAPINFO,
            BITMAPINFOHEADER,
            BI_RGB,
            CAPTUREBLT,
            DIB_RGB_COLORS, //CAPTUREBLT,
            HGDI_ERROR,
            RGBQUAD,
            SRCCOPY,
        },
        winnt::GENERIC_ALL,
        winuser::{
            CloseDesktop, CreateDesktopA, GetDC, GetTopWindow, GetWindow, OpenDesktopA,
            SetThreadDesktop, GW_HWNDLAST,
        },
    },
};

const PIXEL_WIDTH: i32 = 4;

pub struct CapturerGDI {
    screen_dc: HDC,
    dc: HDC,
    bmp: HBITMAP,
    width: i32,
    height: i32,
    desktop: HDESK,
}

impl CapturerGDI {
    pub fn new(name: &[u16], width: i32, height: i32) -> Result<Self, Box<dyn std::error::Error>> {
        /* or Enumerate monitors with EnumDisplayMonitors,
        https://stackoverflow.com/questions/34987695/how-can-i-get-an-hmonitor-handle-from-a-display-device-name
            #[no_mangle]
            pub extern "C" fn callback(m: HMONITOR, dc: HDC, rect: LPRECT, lp: LPARAM) -> BOOL {}
        */
        /*
        shared::windef::HMONITOR,
        winuser::{GetMonitorInfoW, GetSystemMetrics, MONITORINFOEXW},
        let mut mi: MONITORINFOEXW = std::mem::MaybeUninit::uninit().assume_init();
        mi.cbSize = size_of::<MONITORINFOEXW>() as _;
        if GetMonitorInfoW(m, &mut mi as *mut MONITORINFOEXW as _) == 0 {
            return Err(format!("Failed to get monitor information of: {:?}", m).into());
        }
        */

        unsafe {
            let mut desktop = OpenDesktopA(
                hbb_common::config::DESKTOP_NAME.as_ptr() as _,
                0,
                1,
                GENERIC_ALL,
            );
            if desktop.is_null() {
                desktop = CreateDesktopA(
                    hbb_common::config::DESKTOP_NAME.as_ptr() as _,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    0,
                    GENERIC_ALL,
                    ptr::null_mut(),
                );
            }
            SetThreadDesktop(desktop);

            let dc = GetDC(ptr::null_mut());
            if dc.is_null() {
                return Err("Failed to create dc from monitor name".into());
            }

            // if name.is_empty() {
            //     return Err("Empty display name".into());
            // }
            // let screen_dc = CreateDCW(&name[0], 0 as _, 0 as _, 0 as _);
            // if screen_dc.is_null() {
            //     return Err("Failed to create dc from monitor name".into());
            // }

            // Create a Windows Bitmap, and copy the bits into it
            let screen_dc = CreateCompatibleDC(dc);
            if screen_dc.is_null() {
                DeleteDC(screen_dc);
                return Err("Can't get a Windows display".into());
            }

            let bmp = CreateCompatibleBitmap(dc, width, height);
            if bmp.is_null() {
                DeleteDC(screen_dc);
                DeleteDC(dc);
                return Err("Can't create a Windows buffer".into());
            }

            let res = SelectObject(screen_dc, bmp as _);
            if res.is_null() || res == HGDI_ERROR {
                DeleteDC(screen_dc);
                DeleteDC(dc);
                DeleteObject(bmp as _);
                return Err("Can't select Windows buffer".into());
            }

            Ok(Self {
                screen_dc,
                dc,
                bmp,
                width,
                height,
                desktop,
            })
        }
    }

    // pub fn frame_hidden(&self, data: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    //     unsafe {

    //     }
    // }

    fn paint_window(&self, wnd: HWND) -> bool {
        let mut ret = false;

        unsafe {
            let mut rect: RECT = mem::zeroed();
            if GetWindowRect(wnd, &mut rect) == 0 {
                return false;
            }

            // println!("paint {} {} {}", print_wnd_name(wnd), rect.right - rect.left, rect.bottom - rect.top);

            let dc_window = CreateCompatibleDC(self.dc);
            let bmp_window =
                CreateCompatibleBitmap(self.dc, rect.right - rect.left, rect.bottom - rect.top);

            if SelectObject(dc_window, bmp_window as _).is_null() {
                println!("SelectObject");
            }

            if PrintWindow(wnd, dc_window, 0) != 0 {
                if 0 == BitBlt(
                    self.screen_dc,
                    rect.left,
                    rect.top,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    dc_window,
                    0,
                    0,
                    SRCCOPY | CAPTUREBLT,
                ) {
                    println!("bitble");
                }

                ret = true;
            }

            DeleteObject(bmp_window as _);
            DeleteObject(dc_window as _);
        }

        ret
    }

    fn enum_windows_print(&self, wnd: HWND) -> bool {
        unsafe {
            if 0 == IsWindowVisible(wnd) {
                return true;
            }

            self.paint_window(wnd);

            let style = GetWindowLongA(wnd, GWL_EXSTYLE);
            SetWindowLongA(wnd, GWL_EXSTYLE, style | WS_EX_COMPOSITED as i32);

            let mut version: OSVERSIONINFOA = mem::zeroed();
            version.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOA>() as _;

            GetVersionExA(&mut version);
            if version.dwMajorVersion < 6 {
                self.enum_windows_top_to_down(wnd);
            }

            true
        }
    }

    fn enum_windows_top_to_down(&self, owner: HWND) {
        unsafe {
            let mut current_window = GetTopWindow(owner);
            if current_window.is_null() {
                return;
            }

            current_window = GetWindow(current_window, GW_HWNDLAST);

            if current_window.is_null() {
                return;
            }

            loop {
                if !self.enum_windows_print(current_window) {
                    break;
                }

                current_window = GetWindow(current_window, GW_HWNDPREV);
                if current_window.is_null() {
                    break;
                }
            }
        }
    }

    pub fn frame(&self, data: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // let res = BitBlt(
            //     self.dc,
            //     0,
            //     0,
            //     self.width,
            //     self.height,
            //     self.screen_dc,
            //     0,
            //     0,
            //     SRCCOPY | CAPTUREBLT, // CAPTUREBLT enable layered window but also make cursor blinking
            // );

            // if res == 0 {
            //     return Err("Failed to copy screen to Windows buffer".into());
            // }
            // SetThreadDesktop(self.desktop);

            self.enum_windows_top_to_down(ptr::null_mut());
            // self.paint_window(GetDesktopWindow());

            let stride = self.width * PIXEL_WIDTH;
            let size: usize = (stride * self.height) as usize;
            let mut data1: Vec<u8> = Vec::with_capacity(size);
            data1.set_len(size);
            data.resize(size, 0);

            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as _,
                    biWidth: self.width as _,
                    biHeight: self.height as _,
                    biPlanes: 1,
                    biBitCount: (8 * PIXEL_WIDTH) as _,
                    biCompression: BI_RGB,
                    biSizeImage: (self.width * self.height * PIXEL_WIDTH) as _,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };

            // copy bits into Vec
            let res = GetDIBits(
                self.screen_dc,
                self.bmp,
                0,
                self.height as _,
                &mut data[0] as *mut u8 as _,
                &mut bmi as _,
                DIB_RGB_COLORS,
            );
            if res == 0 {
                return Err("GetDIBits failed".into());
            }
            crate::common::ARGBMirror(
                data.as_ptr(),
                stride,
                data1.as_mut_ptr(),
                stride,
                self.width,
                self.height,
            );
            crate::common::ARGBRotate(
                data1.as_ptr(),
                stride,
                data.as_mut_ptr(),
                stride,
                self.width,
                self.height,
                180,
            );
            Ok(())
        }
    }
}

impl Drop for CapturerGDI {
    fn drop(&mut self) {
        unsafe {
            DeleteDC(self.screen_dc);
            DeleteDC(self.dc);
            DeleteObject(self.bmp as _);
            CloseDesktop(self.desktop);
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::super::*;
//     use super::*;
//     #[test]
//     fn test() {
//         match Displays::new().unwrap().next() {
//             Some(d) => {
//                 let w = d.width();
//                 let h = d.height();
//                 let c = CapturerGDI::new(d.name(), w, h).unwrap();
//                 let mut data = Vec::new();
//                 c.frame(&mut data).unwrap();
//                 let mut bitflipped = Vec::with_capacity((w * h * 4) as usize);
//                 for y in 0..h {
//                     for x in 0..w {
//                         let i = (w * 4 * y + 4 * x) as usize;
//                         bitflipped.extend_from_slice(&[data[i + 2], data[i + 1], data[i], 255]);
//                     }
//                 }
//                 repng::encode(
//                     std::fs::File::create("gdi_screen.png").unwrap(),
//                     d.width() as u32,
//                     d.height() as u32,
//                     &bitflipped,
//                 )
//                 .unwrap();
//             }
//             _ => {
//                 assert!(false);
//             }
//         }
//     }
// }
