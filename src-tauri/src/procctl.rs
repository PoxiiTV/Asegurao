// Control de procesos en Windows: listar, congelar (suspender), reanudar,
// cerrar, lanzar y extraer el icono real del .exe como PNG.

#![allow(non_snake_case)]

use base64::Engine;
use std::ffi::c_void;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::Threading::{
    CreateProcessW, OpenProcess, QueryFullProcessImageNameW, TerminateProcess, PROCESS_INFORMATION,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SUSPEND_RESUME,
    PROCESS_TERMINATE, STARTUPINFOW,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub struct Proc {
    pub pid: u32,
    pub exe: String,
}

pub fn list_processes() -> Vec<Proc> {
    let mut out = Vec::new();
    unsafe {
        let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return out,
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
                let exe = String::from_utf16_lossy(&entry.szExeFile[..len]);
                out.push(Proc { pid: entry.th32ProcessID, exe });
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    out
}

pub fn running_pids(exe_name: &str) -> Vec<u32> {
    list_processes()
        .into_iter()
        .filter(|p| p.exe.eq_ignore_ascii_case(exe_name))
        .map(|p| p.pid)
        .collect()
}

pub fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 260];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        if ok.is_ok() && len > 0 {
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            None
        }
    }
}

type NtProcFn = unsafe extern "system" fn(HANDLE) -> i32;

fn ntdll_fn(name: &[u8]) -> Option<NtProcFn> {
    unsafe {
        let lib = LoadLibraryW(PCWSTR(wide("ntdll.dll").as_ptr())).ok()?;
        let proc = GetProcAddress(lib, windows::core::PCSTR(name.as_ptr()))?;
        Some(std::mem::transmute::<_, NtProcFn>(proc))
    }
}

fn for_each_pid(exe_name: &str, access: windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS, f: impl Fn(HANDLE)) {
    for pid in running_pids(exe_name) {
        unsafe {
            if let Ok(handle) = OpenProcess(access, false, pid) {
                f(handle);
                let _ = CloseHandle(handle);
            }
        }
    }
}

pub fn suspend(exe_name: &str) {
    if let Some(nt) = ntdll_fn(b"NtSuspendProcess\0") {
        for_each_pid(exe_name, PROCESS_SUSPEND_RESUME, |h| unsafe {
            nt(h);
        });
    }
}

pub fn resume(exe_name: &str) {
    if let Some(nt) = ntdll_fn(b"NtResumeProcess\0") {
        for_each_pid(exe_name, PROCESS_SUSPEND_RESUME, |h| unsafe {
            nt(h);
        });
    }
}

pub fn terminate(exe_name: &str) {
    for_each_pid(exe_name, PROCESS_TERMINATE, |h| unsafe {
        let _ = TerminateProcess(h, 1);
    });
}

#[allow(dead_code)]
pub fn launch(path: &str) {
    if path.is_empty() {
        return;
    }
    unsafe {
        let mut cmd = wide(path);
        let si = STARTUPINFOW {
            cb: std::mem::size_of::<STARTUPINFOW>() as u32,
            ..Default::default()
        };
        let mut pi = PROCESS_INFORMATION::default();
        if CreateProcessW(
            PCWSTR::null(),
            PWSTR(cmd.as_mut_ptr()),
            None,
            None,
            false,
            Default::default(),
            None,
            PCWSTR::null(),
            &si,
            &mut pi,
        )
        .is_ok()
        {
            let _ = CloseHandle(pi.hProcess);
            let _ = CloseHandle(pi.hThread);
        }
    }
}

/// Extrae el icono real del ejecutable y lo devuelve como PNG en base64.
pub fn icon_png_base64(path: &str) -> Option<String> {
    unsafe {
        let mut shfi = SHFILEINFOW::default();
        let res = SHGetFileInfoW(
            PCWSTR(wide(path).as_ptr()),
            Default::default(),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if res == 0 || shfi.hIcon.is_invalid() {
            return None;
        }
        let png = hicon_to_png(shfi.hIcon);
        let _ = DestroyIcon(shfi.hIcon);
        png.map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes))
    }
}

unsafe fn hicon_to_png(hicon: HICON) -> Option<Vec<u8>> {
    let mut info = ICONINFO::default();
    GetIconInfo(hicon, &mut info).ok()?;
    let color: HBITMAP = info.hbmColor;
    let mask: HBITMAP = info.hbmMask;

    let mut bm = BITMAP::default();
    GetObjectW(
        color,
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bm as *mut _ as *mut c_void),
    );
    let w = bm.bmWidth;
    let h = bm.bmHeight;
    if w <= 0 || h <= 0 {
        let _ = DeleteObject(color);
        let _ = DeleteObject(mask);
        return None;
    }

    let mut bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let pixels = (w * h) as usize;
    let mut buf = vec![0u8; pixels * 4];
    let hdc = GetDC(HWND(std::ptr::null_mut()));
    let scan = GetDIBits(
        hdc,
        color,
        0,
        h as u32,
        Some(buf.as_mut_ptr() as *mut c_void),
        &mut bi,
        DIB_RGB_COLORS,
    );
    ReleaseDC(HWND(std::ptr::null_mut()), hdc);
    let _ = DeleteObject(color);
    let _ = DeleteObject(mask);
    if scan == 0 {
        return None;
    }

    // BGRA -> RGBA; si el icono no trae alfa (todo 0), lo ponemos opaco.
    let mut any_alpha = false;
    for px in buf.chunks_exact(4) {
        if px[3] != 0 {
            any_alpha = true;
            break;
        }
    }
    for px in buf.chunks_exact_mut(4) {
        px.swap(0, 2); // B<->R
        if !any_alpha {
            px[3] = 255;
        }
    }

    let img = image::RgbaImage::from_raw(w as u32, h as u32, buf)?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}
