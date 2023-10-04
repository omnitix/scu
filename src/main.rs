#![allow(dead_code, unused_must_use)]
mod hardware;
pub mod pci_ids;
mod software;

mod logo;
mod utils;

use std::collections::BTreeMap;
use std::fmt::Write;

fn get_len(str: &String) -> usize {
    str.chars().count()
}

fn get_max_len(arr: Vec<String>) -> usize {
    let mut result: usize = 0;
    arr.iter().for_each(|elem| {
        let _len = get_len(elem);
        if _len > result {
            result = _len;
        }
    });
    result
}

fn drive_size_to_string(size: utils::converter::MemorySize) -> String {
    let mut _size: f64 = 0_f64;
    let mut suffix = "";
    if size.gb == 0 {
        _size = size.mb as f64;
        suffix = "MiB";
    } else if size.gb < 1024 {
        _size = size.gb as f64;
        suffix = "GiB";
    } else if size.gb > 1024 {
        _size = size.gb as f64 / 1024_f64;
        suffix = "TiB";
    }

    format!("{:.1}{}", _size, suffix)
}

fn get_info() -> BTreeMap<String, Vec<String>> {
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut buf = String::new();

    // System
    let device_name = hardware::device::get_device_model();
    let distro_name = software::os::get_name();
    let uptime = software::os::get_uptime();
    let hostname = software::os::get_hostname();
    let username = utils::whoami::username().unwrap();
    let shell = software::os::get_shell();
    let kernel_version = software::kernel::get_version();
    let init_system = software::init_system::detect();
    let terminal = software::terminal::get_name();

    write!(buf, "Hostname: {hostname}\0");
    write!(buf, "Username: {username}\0");
    write!(buf, "Distro: {distro_name}\0");
    if device_name.is_some() {
        let device_name = device_name.unwrap();
        write!(buf, "Device: {device_name}\0");
    }
    buf.push_str(format!("Kernel: {}\0", kernel_version).as_str());

    if let Some(init_system) = init_system {
        buf.push_str(
            format!(
                "Init system: {}\0┗Services: {}\0",
                init_system.name, init_system.count_services
            )
            .as_str(),
        );
    }
    write!(buf, "Terminal: {terminal}\0");
    write!(buf, "Shell: {shell}\0");
    if let Some(mut uptime) = uptime {
        buf.push_str("Uptime: ");
        if uptime.hours > 24 {
            buf.push_str(format!("{}d", uptime.hours / 24).as_str());
        }
        uptime.hours %= 24;
        buf.push_str(
            format!(
                " {}:{}:{}\0",
                format!(
                    "{}{}",
                    if uptime.hours < 10 { "0" } else { "" },
                    uptime.hours
                ),
                format!(
                    "{}{}",
                    if uptime.minutes < 10 { "0" } else { "" },
                    uptime.minutes
                ),
                format!(
                    "{}{}",
                    if uptime.seconds < 10 { "0" } else { "" },
                    uptime.seconds
                )
            )
            .as_str(),
        );
    }

    result.insert(
        "System".to_string(),
        buf.split('\0').map(|s| s.to_string()).collect(),
    );
    buf.clear();

    // Packages
    let pkg_info = software::packages::get_info();
    if !pkg_info.is_empty() {
        for manager in pkg_info {
            buf.push_str(format!("{}: {}\0", manager.manager, manager.count_of_packages).as_str());
        }
        result.insert(
            "Packages".to_string(),
            buf.split('\0').map(|s| s.to_string()).collect(),
        );
        buf.clear();
    }

    // Processor
    let cpu_info = hardware::cpu::get_info();
    buf.push_str(format!("Model: {}\0", cpu_info.model).as_str());
    buf.push_str(format!("Frequency: {:.2}GHz\0", cpu_info.freq.ghz).as_str());
    if cpu_info.cores > 0 {
        buf.push_str(
            format!(
                "Computing units: {} Cores / {} Threads\0",
                cpu_info.cores, cpu_info.threads
            )
            .as_str(),
        );
    }
    if cpu_info.temperature > 0.0 {
        buf.push_str(format!("Temperature: {}°C\0", cpu_info.temperature).as_str());
    }

    result.insert(
        "Processor".to_string(),
        buf.split('\0').map(|s| s.to_string()).collect(),
    );
    buf.clear();

    // Memory
    let mem_info = hardware::ram::get_info();
    buf.push_str(format!("RAM: {}MiB / {}MiB\0", mem_info.used.mb, mem_info.total.mb).as_str());
    if mem_info.swap_enabled {
        buf.push_str(
            format!(
                "Swap: {}MiB / {}MiB\0",
                mem_info.swap_used.mb, mem_info.swap_total.mb
            )
            .as_str(),
        );
    }

    result.insert(
        "Memory".to_string(),
        buf.split('\0').map(|s| s.to_string()).collect(),
    );
    buf.clear();

    // Battery
    let battery = hardware::battery::get_battery_info();
    if let Some(battery) = battery {
        buf.push_str(
            format!(
                "Model: {}\0Technology: {}\0Capacity: {}%\0Status: {}\0",
                battery.model, battery.technology, battery.capacity, battery.status
            )
            .as_str(),
        );
        result.insert(
            "Battery".to_string(),
            buf.split('\0').map(|s| s.to_string()).collect(),
        );
        buf.clear();
    }

    // Drives
    let drives = hardware::drive::scan_drives();
    if let Some(drives) = drives {
        if !drives.is_empty() {
            for drive in drives {
                buf.push_str(
                    format!("{}: {}\0", drive.model, drive_size_to_string(drive.size)).as_str(),
                );
            }
            result.insert(
                "Drives".to_string(),
                buf.split('\0').map(|s| s.to_string()).collect(),
            );
            buf.clear();
        }
    }

    // Graphics
    let gpus = hardware::gpu::get_info();
    if let Some(gpus) = gpus {
        let count_gpus = gpus.len();
        for entry in gpus {
            let gpu_id = entry.0;
            let gpu_info = entry.1;
            let mut sub_info: Vec<String> = Vec::new();
            if gpu_info.driver != "Unknown" {
                sub_info.push(format!("Driver: {}", gpu_info.driver));
            }
            if gpu_info.temperature > 0.0 {
                sub_info.push(format!("Temperature: {}°C", gpu_info.temperature));
            }
            buf.push_str(
                format!(
                    "GPU{}: {}\0",
                    if count_gpus > 1 {
                        format!(" #{}", gpu_id)
                    } else {
                        String::from("")
                    },
                    gpu_info.model,
                )
                .as_str(),
            );
            if !sub_info.is_empty() {
                sub_info.iter().for_each(|line| {
                    buf.push_str(
                        format!(
                            "{}{}\0",
                            if line == sub_info.iter().next_back().unwrap() {
                                "┗"
                            } else {
                                "┣"
                            },
                            line
                        )
                        .as_str(),
                    );
                })
            }
        }
    }
    let session_type = software::graphics::get_session_type();
    if let Some(session_type) = session_type {
        write!(buf, "Session type: {session_type}\0");
    }
    let de = software::graphics::detect_de();
    if let Some(de) = de {
        write!(buf, "Environment: {de}\0");
    }
    let wm = software::graphics::detect_wm();
    if let Some(wm) = wm {
        write!(buf, "Window manager: {wm}\0");
    }
    if !buf.is_empty() {
        result.insert(
            "Graphics".to_string(),
            buf.split('\0').map(|s| s.to_string()).collect(),
        );
    }
    buf.clear();

    result
}

fn get_map_max_len(map: BTreeMap<String, Vec<String>>) -> usize {
    let mut result: usize = 0;
    if map.is_empty() {
        return result;
    }

    for key in map.keys() {
        for line in map.get(key).unwrap() {
            let _len = get_len(line);
            if _len > result {
                result = _len;
            }
        }
    }

    result
}

fn get_param_max_len(map: BTreeMap<String, Vec<String>>) -> usize {
    let mut result: usize = 0;
    if map.is_empty() {
        return result;
    }

    for key in map.keys() {
        for line in map.get(key).unwrap() {
            let _len = get_len(&String::from(line.split(": ").next().unwrap()));
            if _len > result {
                result = _len;
            }
        }
    }

    result
}

fn format_info(map: BTreeMap<String, Vec<String>>) -> BTreeMap<String, Vec<String>> {
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();

    let max_param_len = get_param_max_len(map.clone());

    for category in map.keys() {
        let mut buf: Vec<String> = Vec::new();
        map.get(category.as_str())
            .unwrap()
            .iter()
            .for_each(|info_line| {
                if !info_line.is_empty() {
                    let mut line = info_line.split(": ");
                    let line_param = line.next().unwrap();
                    let param_len = get_len(&line_param.to_string());
                    let line_val = line.next().unwrap().trim().to_string();
                    if &line_val != "Unknown" && &line_val != "0" {
                        buf.push(format!(
                            "{}:{}{}",
                            line_param,
                            " ".repeat(max_param_len + 2 - param_len),
                            line_val
                        ));
                    }
                }
            });
        if !buf.is_empty() {
            result.insert(category.to_string(), buf);
        }
    }

    result
}

fn colorize(str: &str, r: u16, g: u16, b: u16) -> String {
    format!("\x1b[38;2;{r};{g};{b}m{str}\x1B[0m")
}

fn colorize_background(str: &str, r: u16, g: u16, b: u16) -> String {
    let mut result = format!("\x1b[48;2;{r};{g};{b}m{str}\x1B[0m");
    if (r + g + b) / 3 > 123 {
        result = colorize(&result, 0, 0, 0);
    }
    result
}

fn print_info() {
    let info = format_info(get_info());

    let max_len = get_map_max_len(info.clone());
    let mut to_display: Vec<String> = Vec::new();

    let distro_name = software::os::get_name();
    // TODO More logos
    let is_supported_logo = distro_name.trim().to_lowercase().contains("arch")
        || distro_name.trim().to_lowercase().contains("ubuntu");

    let logo_lines: Vec<String> = if is_supported_logo {
        if distro_name.trim().to_lowercase().contains("arch") {
            logo::ARCH_LOGO
                .trim()
                .lines()
                .map(|line| line.to_string())
                .collect()
        } else {
            logo::UBUNTU_LOGO
                .trim()
                .lines()
                .map(|line| line.to_string())
                .collect()
        }
    } else {
        Vec::new()
    };
    let logo_max_len = get_max_len(logo_lines.clone());
    let logo_width = logo_max_len + 2;

    let total_width = max_len + 2;

    let (padding_before, padding_after) = if is_supported_logo {
        let padding_before = (total_width - logo_width) / 2;
        let padding_after = total_width - logo_width - padding_before;
        (padding_before, padding_after)
    } else {
        (0, 0)
    };

    to_display.push(format!("┌{}┐", "─".repeat(total_width)));

    if is_supported_logo {
        for line in 0..logo_lines.len() {
            let logo_line = format!(
                "│ {}{}{} │",
                " ".repeat(padding_before),
                logo_lines[line],
                " ".repeat(padding_after)
            );
            to_display.push(logo_line);
        }

        to_display.push(format!("├{}┤", "─".repeat(total_width)));
    }

    for category in info.keys().rev() {
        to_display.push(format!(
            "{}─┤ {} ├{}{}",
            if Some(category) == info.keys().next_back() {
                "├"
            } else {
                "├"
            },
            category,
            "─".repeat(max_len - get_len(category) - 3),
            if Some(category) == info.keys().next_back() {
                "┤"
            } else {
                "┤"
            }
        ));
        info.get(category.as_str())
            .unwrap()
            .iter()
            .for_each(|info_line| {
                to_display.push(format!(
                    "│ {}{}│",
                    info_line,
                    " ".repeat(max_len - get_len(info_line) + 1)
                ))
            });
    }

    to_display.push(format!("└{}┘", "─".repeat(total_width)));
    to_display.iter().for_each(|info_line| {
        println!("{}", info_line);
    });
}

fn main() {
    print_info();
}
