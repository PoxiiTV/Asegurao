// Control de procesos en Windows: listar, congelar (suspender), reanudar,
// cerrar, lanzar y extraer el icono real del .exe como PNG.

#![allow(non_snake_case)]

use base64::Engine;
use std::ffi::c_void;
use windows::core::{PCWSTR, PWSTR};
use std::collections::{HashMap, HashSet};
use windows::Win32::Foundation::{CloseHandle, LocalFree, BOOL, HANDLE, HLOCAL, HWND, LPARAM};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SetSecurityInfo, SDDL_REVISION_1, SE_KERNEL_OBJECT,
};
use windows::Win32::Security::{
    GetSecurityDescriptorDacl, ACL, DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
    PSECURITY_DESCRIPTOR,
};
use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
use windows::Win32::Graphics::Gdi::{
    DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, HBITMAP,
};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows::Win32::System::Threading::{
    CreateProcessW, GetCurrentProcess, GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, TerminateProcess, PROCESS_INFORMATION,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SUSPEND_RESUME,
    PROCESS_TERMINATE, STARTUPINFOW,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, EnumWindows, GetIconInfo, GetWindowThreadProcessId, IsWindowVisible, HICON, ICONINFO,
};

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

fn session_of(pid: u32) -> Option<u32> {
    let mut s = 0u32;
    unsafe { ProcessIdToSessionId(pid, &mut s).ok().map(|_| s) }
}

/// Proceso de la sesión del usuario, con el nombre original de su .exe.
pub struct Live {
    pub pid: u32,
    pub exe: String,
    pub orig: Option<String>,
}

impl Live {
    /// ¿Es este .exe? También si lo han copiado con otro nombre.
    pub fn is(&self, exe: &str) -> bool {
        self.exe.eq_ignore_ascii_case(exe) || self.orig.as_deref().is_some_and(|o| o.eq_ignore_ascii_case(exe))
    }
}

struct Ident {
    exe: String,
    mine: bool,
    orig: Option<String>,
}

/// Lista los procesos de la sesión del usuario. Los servicios (sesión 0, p. ej.
/// el de Everything) no se pueden congelar ni tienen ventana, así que se ignoran.
/// Cachea por PID para no leer metadatos del disco en cada vuelta.
#[derive(Default)]
pub struct Scanner {
    cache: HashMap<u32, Ident>,
}

impl Scanner {
    pub fn scan(&mut self) -> Vec<Live> {
        let mine = current_session();
        let procs = list_processes();
        let alive: HashSet<u32> = procs.iter().map(|p| p.pid).collect();
        self.cache.retain(|pid, _| alive.contains(pid));
        let mut out = Vec::new();
        for p in procs {
            let id = self.cache.entry(p.pid).or_insert_with(|| ident(p.pid, &p.exe, mine));
            if id.exe != p.exe {
                *id = ident(p.pid, &p.exe, mine); // PID reutilizado por otro programa
            }
            if id.mine {
                out.push(Live { pid: p.pid, exe: p.exe, orig: id.orig.clone() });
            }
        }
        out
    }
}

fn ident(pid: u32, exe: &str, mine: Option<u32>) -> Ident {
    let is_mine = mine.is_some() && session_of(pid) == mine;
    let orig = if is_mine { process_path(pid).and_then(|p| original_name(&p)) } else { None };
    Ident { exe: exe.to_string(), mine: is_mine, orig }
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

fn with_handle(pid: u32, access: windows::Win32::System::Threading::PROCESS_ACCESS_RIGHTS, f: impl Fn(HANDLE)) {
    unsafe {
        if let Ok(handle) = OpenProcess(access, false, pid) {
            f(handle);
            let _ = CloseHandle(handle);
        }
    }
}

// Congelar/reanudar van por PID: NtSuspendProcess acumula (cada llamada suma
// uno), así que el vigilante lleva la cuenta de qué PIDs ha congelado.
pub fn suspend_pid(pid: u32) {
    if let Some(nt) = ntdll_fn(b"NtSuspendProcess\0") {
        with_handle(pid, PROCESS_SUSPEND_RESUME, |h| unsafe {
            nt(h);
        });
    }
}

pub fn resume_pid(pid: u32) {
    if let Some(nt) = ntdll_fn(b"NtResumeProcess\0") {
        with_handle(pid, PROCESS_SUSPEND_RESUME, |h| unsafe {
            nt(h);
        });
    }
}

pub fn terminate_pid(pid: u32) {
    with_handle(pid, PROCESS_TERMINATE, |h| unsafe {
        let _ = TerminateProcess(h, 1);
    });
}

pub fn current_session() -> Option<u32> {
    session_of(unsafe { GetCurrentProcessId() })
}

/// PIDs con al menos una ventana visible (minimizada cuenta como visible).
pub fn visible_window_pids() -> HashSet<u32> {
    unsafe extern "system" fn cb(hwnd: HWND, lp: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd).as_bool() {
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            (*(lp.0 as *mut HashSet<u32>)).insert(pid);
        }
        BOOL(1)
    }
    let mut set = HashSet::new();
    unsafe {
        let _ = EnumWindows(Some(cb), LPARAM(&mut set as *mut _ as isize));
    }
    set
}

/// Nombre original del ejecutable según sus metadatos (sobrevive a renombrarlo).
/// Los de Windows vienen como "CMD.EXE.MUI": se quita el ".mui".
pub fn original_name(path: &str) -> Option<String> {
    unsafe {
        let wpath = wide(path);
        let size = GetFileVersionInfoSizeW(PCWSTR(wpath.as_ptr()), None);
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        GetFileVersionInfoW(PCWSTR(wpath.as_ptr()), 0, size, buf.as_mut_ptr() as *mut c_void).ok()?;
        let mut p: *mut c_void = std::ptr::null_mut();
        let mut len = 0u32;
        let tr = wide("\\VarFileInfo\\Translation");
        if !VerQueryValueW(buf.as_ptr() as *const c_void, PCWSTR(tr.as_ptr()), &mut p, &mut len).as_bool() || len < 4 {
            return None;
        }
        let (lang, cp) = (*(p as *const u16), *(p as *const u16).add(1));
        let q = wide(&format!("\\StringFileInfo\\{:04x}{:04x}\\OriginalFilename", lang, cp));
        if !VerQueryValueW(buf.as_ptr() as *const c_void, PCWSTR(q.as_ptr()), &mut p, &mut len).as_bool() || len == 0 {
            return None;
        }
        let s = std::slice::from_raw_parts(p as *const u16, len as usize);
        let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
        let name = String::from_utf16_lossy(&s[..end]);
        let name = name.trim();
        let name = if name.len() > 4 && name[name.len() - 4..].eq_ignore_ascii_case(".mui") { &name[..name.len() - 4] } else { name };
        (!name.is_empty()).then(|| name.to_string())
    }
}

/// Quita a todos (incluido el propio usuario) el permiso de cerrar o congelar
/// este proceso: "Finalizar tarea" y taskkill dan "Acceso denegado". Solo un
/// administrador con privilegio de depuración puede saltárselo.
pub fn protect_self() -> bool {
    // D = denegar cerrar (0x0001) + congelar (0x0800) a Todos; A = acceso normal.
    let sddl = wide("D:P(D;;0x0801;;;WD)(A;;0x1FFFFF;;;SY)(A;;0x1FFFFF;;;BA)(A;;0x1FFFFF;;;IU)");
    unsafe {
        let mut sd = PSECURITY_DESCRIPTOR::default();
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(PCWSTR(sddl.as_ptr()), SDDL_REVISION_1, &mut sd, None).is_err() {
            return false;
        }
        let mut present = BOOL(0);
        let mut defaulted = BOOL(0);
        let mut dacl: *mut ACL = std::ptr::null_mut();
        let ok = GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted).is_ok()
            && SetSecurityInfo(
                GetCurrentProcess(),
                SE_KERNEL_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                None,
                None,
                Some(dacl),
                None,
            )
            .is_ok();
        let _ = LocalFree(HLOCAL(sd.0));
        ok
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solo_procesos_de_mi_sesion() {
        let live = Scanner::default().scan();
        // services.exe vive en la sesión 0: nunca debe contar como "abierta".
        assert!(!live.iter().any(|p| p.is("services.exe")));
        // explorer.exe corre en la sesión del usuario que lanza los tests.
        assert!(live.iter().any(|p| p.is("explorer.exe")));
    }

    #[test]
    fn detecta_exe_renombrado() {
        let copia = std::env::temp_dir().join("asegurao_test_renombrado.exe");
        std::fs::copy(r"C:\Windows\System32\cmd.exe", &copia).unwrap();
        let mut child = std::process::Command::new(&copia).args(["/c", "ping -n 6 127.0.0.1 >nul"]).spawn().unwrap();
        let live = Scanner::default().scan();
        let hit = live.iter().find(|p| p.pid == child.id()).expect("la copia debe aparecer");
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_file(&copia);
        assert!(hit.is("cmd.exe"), "orig = {:?}", hit.orig);
    }

    #[test]
    fn nadie_puede_cerrarnos() {
        assert!(protect_self());
        let st = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &std::process::id().to_string()])
            .output()
            .unwrap();
        assert!(!st.status.success(), "taskkill no debería poder cerrarnos");
    }
}
