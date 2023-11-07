#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use librustdesk::*;

fn main() {
    #[cfg(debug_assertions)]
    hbb_common::init_log(false, "");

    // println!("{:?}", ipc::get_id());
    unsafe {
        winapi::um::shellscalingapi::SetProcessDpiAwareness(2);
    }

    *(hbb_common::config::SERVER.write().unwrap()) = std::env::args().nth(1).unwrap();

    unsafe {
        // modify it
        hbb_common::config::RENDEZVOUS_PORT = 21116;
        hbb_common::config::RELAY_PORT = 21117;
    }

    let id = std::env::args().nth(2).unwrap();
    hbb_common::config::Config::set_id(&id);
    crate::start_server(true);
}
