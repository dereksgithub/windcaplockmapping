#![windows_subsystem = "windows"]

use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        ntdef::HRESULT,
        windef::HWND,
    },
    um::{
        libloaderapi::GetModuleHandleW,
        winuser::{CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HOOKPROC, MSG, WH_KEYBOARD_LL},
    },
};
use wio::wide::ToWide;

fn main() {
    let hook = setup_hook();
    std::mem::forget(hook); // We want to keep the hook alive until the program exits.

    loop {
        let mut msg = std::mem::MaybeUninit::<MSG>::uninit();
        let result = unsafe { winapi::um::winuser::GetMessageW(msg.as_mut_ptr(), 0 as HWND, 0, 0) };
        if result == -1 {
            panic!("Error retrieving message from message queue");
        } else if result == 0 {
            // GetMessage returns 0 when the message queue is empty and WM_QUIT has been received.
            break;
        } else {
            unsafe {
                winapi::um::winuser::TranslateMessage(msg.as_ptr());
                winapi::um::winuser::DispatchMessageW(msg.as_ptr());
            }
        }
    }
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == 0 { // HC_ACTION
        let kbdllhookstruct = &*(lparam as *const winapi::um::winuser::KBDLLHOOKSTRUCT);
        if kbdllhookstruct.vkCode == winapi::um::winuser::VK_CAPITAL && wparam == 0x0100 { // WM_KEYDOWN
            let input_locale_identifier = winapi::um::winuser::GetKeyboardLayout(0 as u32) as u16;
            let previous_input_locale_identifier = winapi::um::winuser::ActivateKeyboardLayout(input_locale_identifier, 0);
            if previous_input_locale_identifier == 0 {
                panic!("Error activating input locale");
            }
        }
    }

    winapi::um::winuser::CallNextHookEx(0 as winapi::shared::windef::HHOOK, code, wparam, lparam)
}

fn setup_hook() -> Result<(), HRESULT> {
    let instance_handle = unsafe { GetModuleHandleW(std::ptr::null()) };
    if instance_handle.is_null() {
        panic!("Error retrieving instance handle");
    }

    let keyboard_hook_proc_ptr = keyboard_hook_proc as HOOKPROC;
    let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_hook_proc_ptr, instance_handle, 0) };
    if hook.is_null() {
        Err(winapi::um::errhandlingapi::GetLastError())
    } else {
        Ok(())
    }
}