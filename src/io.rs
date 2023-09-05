use crate::device::*;

use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const FILE_FOUND_DEVICES: &'static str = "devices_found.txt";

// Save the IP address to a file
// That way, discovery via UDP broadcast can be skipped
// File Format:
// RTSP: URL for device streaming ONVIF: URL for Onvif commands
pub fn file_save(devices: &Vec<Device>) -> Result<()> {
    if devices.len() == 0 {
        return Err(anyhow!(
            "[OnvifClient][file_save] Provided empty list of devices"
        ));
    }

    let path = Path::new(FILE_FOUND_DEVICES);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = File::create(&path)?;

    let mut contents = String::new();
    for device in devices {
        let url_rtsp = match device.url_rtsp.as_ref() {
            Some(url) => url.to_string(),
            None => String::new(),
        };

        let device_line = format!("IP: {} ONVIF: {}", url_rtsp, device.url_onvif);
        contents = format!("{contents}{device_line}\n");
    }

    file.write_all(contents.as_bytes())?;

    Ok(())
}

pub fn file_load() -> Result<Vec<Device>> {
    let open = Path::new(FILE_FOUND_DEVICES);
    let path = open.display();
    let mut contents_str = String::new();

    // Open a file in read-only mode, returns `io::Result<File>`
    let mut file = File::open(&open)?;
    let contents_size = file.read_to_string(&mut contents_str)?;

    if contents_size == 0 {
        return Err(anyhow!(
            "[OnvifClient][file_check] File found at {path}, but empty"
        ));
    }
    if !contents_str.contains("IP") {
        return Err(anyhow!(
            "[OnvifClient][file_check] File found at {path}, but no devices"
        ));
    }

    let vec_devices: Vec<Device> = contents_str
        .lines()
        .map(|line| line.split(' ').collect::<Vec<&str>>())
        .map(|line| {
            line.iter()
                .enumerate()
                .filter(|(i, _)| i % 2 == 1)
                .map(|(_, val)| *val)
                .collect::<Vec<&str>>()
        })
        .map(|vals| {
            let url_rtsp = match vals[0].is_empty() {
                true => None,
                false => Some(
                    vals[0]
                        .parse()
                        .expect("[OnvifClient][file_check] Parse error on IP"),
                ),
            };

            let mut device = Device::new();
            device.url_rtsp = url_rtsp;
            device.url_onvif = vals[1]
                .parse()
                .expect("[OnvifClient][file_check] Parse error on onvif url");

            device
        })
        .collect();

    if vec_devices.len() == 0 {
        return Err(anyhow!(
            "[OnvifClient][file_check] Error parsing devices at {path}."
        ));
    }

    Ok(vec_devices)
}
