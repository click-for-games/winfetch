use colored::*;
use std::env;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, HORZRES, VERTRES};
use windows_sys::Win32::System::Registry::{
    RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, KEY_READ,
};
use windows_sys::Win32::System::SystemInformation::{
    GlobalMemoryStatusEx, MEMORYSTATUSEX,
};

fn get_reg_string(key_path: &str, value_name: &str) -> String {
    let subkey: Vec<u16> = key_path.encode_utf16().chain(std::iter::once(0)).collect();
    let value: Vec<u16> = value_name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut hkey = 0isize;

    unsafe {
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_READ, &mut hkey) == 0 {
            let mut buf_type = 0u32;
            let mut buf_len = 512u32;
            let mut buf = vec![0u8; 512];
            if RegQueryValueExW(
                hkey,
                value.as_ptr(),
                std::ptr::null_mut(),
                &mut buf_type,
                buf.as_mut_ptr(),
                &mut buf_len,
            ) == 0 {
                let u16_slice = std::slice::from_raw_parts(
                    buf.as_ptr() as *const u16,
                    (buf_len / 2) as usize,
                );
                let os_str = OsString::from_wide(u16_slice);
                return os_str.to_string_lossy().trim_matches('\0').trim().to_string();
            }
        }
    }
    "Unknown".to_string()
}

fn get_cpu_info() -> String {
    get_reg_string(
        "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
        "ProcessorNameString",
    )
}

fn get_os_info() -> String {
    get_reg_string(
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
        "ProductName",
    )
}

fn get_gpu_info() -> String {
    get_reg_string(
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}\\0000",
        "DriverDesc",
    )
}

fn get_ram_info() -> String {
    unsafe {
        let mut mem_status: MEMORYSTATUSEX = std::mem::zeroed();
        mem_status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(&mut mem_status) != 0 {
            let used_gb = (mem_status.ullTotalPhys - mem_status.ullAvailPhys) as f64 / 1024.0 / 1024.0 / 1024.0;
            let total_gb = mem_status.ullTotalPhys as f64 / 1024.0 / 1024.0 / 1024.0;
            format!("{:.2} GB / {:.2} GB ({})", used_gb, total_gb, format!("{}%", mem_status.dwMemoryLoad).cyan())
        } else {
            "Unknown".to_string()
        }
    }
}

fn get_resolution() -> String {
    unsafe {
        let hdc = GetDC(0);
        if hdc != 0 {
            let width = GetDeviceCaps(hdc, HORZRES as i32);
            let height = GetDeviceCaps(hdc, VERTRES as i32);
            format!("{}x{}", width, height)
        } else {
            "Unknown".to_string()
        }
    }
}

fn main() {
    let username = env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    let computername = env::var("COMPUTERNAME").unwrap_or_else(|_| "pc".to_string());

    let logo = vec![
        "  ████████   ████████  ".cyan().bold(),
        "  ████████   ████████  ".cyan().bold(),
        "  ████████   ████████  ".cyan().bold(),
        "                       ".normal(),
        "  ████████   ████████  ".cyan().bold(),
        "  ████████   ████████  ".cyan().bold(),
        "  ████████   ████████  ".cyan().bold(),
    ];

    let header = format!("{}@{}", username.cyan().bold(), computername.cyan().bold());
    let border = "-".repeat(username.len() + computername.len() + 1);

    let colors = format!(
        "{}{}{}{}{}{}{}{}",
        "██".black(),
        "██".red(),
        "██".green(),
        "██".yellow(),
        "██".blue(),
        "██".magenta(),
        "██".cyan(),
        "██".white()
    );

    let info = vec![
        header,
        border,
        format!("{}: {}", "OS".bold().cyan(), get_os_info()),
        format!("{}: {}", "Resolution".bold().cyan(), get_resolution()),
        format!("{}: {}", "CPU".bold().cyan(), get_cpu_info()),
        format!("{}: {}", "GPU".bold().cyan(), get_gpu_info()),
        format!("{}: {}", "Memory".bold().cyan(), get_ram_info()),
        format!("{}: {}", "Shell".bold().cyan(), env::var("ComSpec").unwrap_or_default()),
        "".to_string(),
        colors,
    ];

    let max_lines = logo.len().max(info.len());
    for i in 0..max_lines {
        let left = if i < logo.len() {
            logo[i].to_string()
        } else {
            "                       ".to_string()
        };
        let right = if i < info.len() { &info[i] } else { "" };
        println!("{}   {}", left, right);
    }
}