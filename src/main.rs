use colored::*;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::env;
use std::ffi::OsString;
use std::io::{stdout, Write};
use std::os::windows::ffi::OsStringExt;
use std::time::{Duration, Instant};
use windows_sys::Win32::Graphics::Gdi::{GetDC, GetDeviceCaps, HORZRES, VERTRES};
use windows_sys::Win32::System::Console::{
    GetStdHandle, GetConsoleMode, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
};
use windows_sys::Win32::System::Registry::{
    RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, KEY_READ,
};
use windows_sys::Win32::System::SystemInformation::{
    GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX,
};

/// Forces Windows Console to process ANSI escape sequences (fixes raw \x1b[1;36m output)
fn enable_ansi_support() {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle != 0 && handle != -1isize {
            let mut mode = 0u32;
            if GetConsoleMode(handle, &mut mode) != 0 {
                SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

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
    let name = get_reg_string("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", "ProductName");
    let build = get_reg_string("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion", "CurrentBuild");
    format!("{} x86_64 (Build {})", name, build)
}

fn get_motherboard_info() -> String {
    let vendor = get_reg_string("HARDWARE\\DESCRIPTION\\System\\BIOS", "BaseBoardManufacturer");
    let product = get_reg_string("HARDWARE\\DESCRIPTION\\System\\BIOS", "BaseBoardProduct");
    format!("{} {}", vendor, product)
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
            let pct = mem_status.dwMemoryLoad;
            format!("{:.2} GiB / {:.2} GiB ({}%)", used_gb, total_gb, pct)
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

fn get_uptime() -> String {
    unsafe {
        let ms = GetTickCount64();
        let secs = ms / 1000;
        let mins = (secs / 60) % 60;
        let hours = (secs / 3600) % 24;
        let days = secs / 86400;
        if days > 0 {
            format!("{} days, {} hours, {} mins", days, hours, mins)
        } else {
            format!("{} hours, {} mins", hours, mins)
        }
    }
}

fn run_dih_matrix() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

    let (cols, rows) = crossterm::terminal::size()?;
    let mut rng = rand::thread_rng();

    let mut drops: Vec<i16> = (0..cols).map(|_| rng.gen_range(-20..0)).collect();
    let chars = ['d', 'i', 'h', 'D', 'I', 'H', '1', '0'];

    loop {
        if event::poll(Duration::from_millis(40))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc || key.code == KeyCode::Char('c') {
                    break;
                }
            }
        }

        for x in 0..cols {
            let y = drops[x as usize];
            if y >= 0 && y < rows as i16 {
                let ch = chars[rng.gen_range(0..chars.len())];
                execute!(stdout, cursor::MoveTo(x, y as u16))?;
                if rng.gen_bool(0.1) {
                    print!("{}", ch.to_string().bold().white());
                } else {
                    print!("{}", ch.to_string().bold().green());
                }
            }

            let tail = y - 12;
            if tail >= 0 && tail < rows as i16 {
                execute!(stdout, cursor::MoveTo(x, tail as u16))?;
                print!(" ");
            }

            drops[x as usize] += 1;
            if drops[x as usize] > rows as i16 + 15 {
                drops[x as usize] = rng.gen_range(-10..0);
            }
        }
        stdout.flush()?;
    }

    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    disable_raw_mode()?;
    Ok(())
}

fn main() {
    // Enable VT processing immediately for clean colors
    enable_ansi_support();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "dih" {
        if let Err(e) = run_dih_matrix() {
            eprintln!("Error playing matrix: {}", e);
        }
        return;
    }

    let start = Instant::now();

    let username = env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    let computername = env::var("COMPUTERNAME").unwrap_or_else(|_| "pc".to_string());

    let logo = vec![
        format!("{}  {}", "█████████".red().bold(), "█████████".green().bold()),
        format!("{}  {}", "█████████".red().bold(), "█████████".green().bold()),
        format!("{}  {}", "█████████".red().bold(), "█████████".green().bold()),
        "                       ".to_string(),
        format!("{}  {}", "█████████".blue().bold(), "█████████".yellow().bold()),
        format!("{}  {}", "█████████".blue().bold(), "█████████".yellow().bold()),
        format!("{}  {}", "█████████".blue().bold(), "█████████".yellow().bold()),
    ];

    let header = format!("{}@{}", username.cyan().bold(), computername.cyan().bold());
    let border = "-".repeat(username.len() + computername.len() + 1);

    let colors_primary = format!(
        "{}{}{}{}{}{}{}{}",
        "  ".on_black(),
        "  ".on_red(),
        "  ".on_green(),
        "  ".on_yellow(),
        "  ".on_blue(),
        "  ".on_magenta(),
        "  ".on_cyan(),
        "  ".on_white()
    );

    let info = vec![
        header,
        border,
        format!("{}: {}", "OS".cyan().bold(), get_os_info()),
        format!("{}: {}", "Host".cyan().bold(), get_motherboard_info()),
        format!("{}: {}", "Uptime".cyan().bold(), get_uptime()),
        format!("{}: {}", "Resolution".cyan().bold(), get_resolution()),
        format!("{}: {}", "CPU".cyan().bold(), get_cpu_info()),
        format!("{}: {}", "GPU".cyan().bold(), get_gpu_info()),
        format!("{}: {}", "Memory".cyan().bold(), get_ram_info()),
        format!("{}: {}", "Shell".cyan().bold(), env::var("ComSpec").unwrap_or_default()),
        "".to_string(),
        colors_primary,
    ];

    let max_lines = logo.len().max(info.len());
    for i in 0..max_lines {
        let left = if i < logo.len() {
            &logo[i]
        } else {
            "                       "
        };
        let right = if i < info.len() { &info[i] } else { "" };
        println!("{}   {}", left, right);
    }

    let duration = start.elapsed();
    println!("\n{}", format!("Fetched in {:.2?}", duration).bright_black().italic());
}