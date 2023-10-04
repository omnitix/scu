use std::fs;
use std::path::Path;
use std::process::Command;

pub fn get_device_model() -> Option<String> {
    let mut result: String = String::new();
    let mut brand: String;
    let model: String;

    if Path::new("/system/app").exists() && Path::new("/system/priv-app").exists() {
        brand = String::from_utf8(
            Command::new("getprop")
                .args(["ro.product.brand"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        if brand == "google" {
            brand = "Google".to_string();
        }
        model = String::from_utf8(
            Command::new("getprop")
                .args(["ro.product.model"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();

        result = format!("{brand} {model}");
    } else if Path::new("/sys/devices/virtual/dmi/id/board_vendor").exists()
        && Path::new("/sys/devices/virtual/dmi/id/product_name").exists()
        && Path::new("/sys/devices/virtual/dmi/id/product_version").exists()
    {
        brand = fs::read_to_string("/sys/devices/virtual/dmi/id/board_vendor").unwrap();
        model = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").unwrap();
        let version = fs::read_to_string("/sys/devices/virtual/dmi/id/product_version").unwrap();

        result = format!("{brand} {model} {version}");
    } else if Path::new("/sys/devices/virtual/dmi/id/product_name").exists()
        && Path::new("/sys/devices/virtual/dmi/id/product_version").exists()
    {
        let name = fs::read_to_string("/sys/devices/virtual/dmi/id/product_name").unwrap();
        let version = fs::read_to_string("/sys/devices/virtual/dmi/id/product_version").unwrap();

        result = format!("{name} {version}");
    } else if Path::new("/sys/devices/virtual/dmi/id/board_vendor").exists()
        && Path::new("/sys/devices/virtual/dmi/id/board_name").exists()
    {
        brand = fs::read_to_string("/sys/devices/virtual/dmi/id/board_vendor").unwrap();
        model = fs::read_to_string("/sys/devices/virtual/dmi/id/board_name").unwrap();

        result = format!("{brand} {model}");
    }

    if result.is_empty() {
        return None;
    }

    Some(result.replace(['\0', '\n'], "").to_string())
}
