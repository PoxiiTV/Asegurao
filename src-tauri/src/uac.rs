// Segunda llave: confirmación de Windows (UAC). Lanza una operación elevada
// trivial con el verbo "runas"; si el usuario aprueba el diálogo seguro de
// Windows, ShellExecuteW devuelve > 32. Si lo cancela, devuelve <= 32.

#![allow(non_snake_case)]

use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn confirm() -> bool {
    let verb = wide("runas");
    let file = wide("cmd.exe");
    let params = wide("/c exit");
    unsafe {
        let result = ShellExecuteW(
            HWND(std::ptr::null_mut()),
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR(params.as_ptr()),
            PCWSTR::null(),
            SW_HIDE,
        );
        result.0 as isize > 32
    }
}
