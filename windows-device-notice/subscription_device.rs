use anyhow::{Context, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::{fs, os::windows::process::CommandExt, process::Command, sync::{Mutex, OnceLock}};

const HOSTS: &[&str] = &[
    "vip2027-1.pages.dev", "vip1.959621.xyz",
    "vip2-x3w.pages.dev", "vip2.959621.xyz",
    "vip4-2jc.pages.dev", "vip4.959621.xyz",
    "vip-5.pages.dev", "vip5.959621.xyz",
    "vip7-aiz.pages.dev", "vip7.959621.xyz",
    "v-ip9.pages.dev", "vip9.959621.xyz",
];

#[derive(Default, Deserialize)]
struct Computer {
    #[serde(default)]
    machine: String,
    #[serde(default)]
    brand: String,
    #[serde(default)]
    model: String,
}

#[derive(Deserialize, Serialize)]
struct Identity {
    machine: String,
    id: String,
}

fn computer() -> &'static Computer {
    static VALUE: OnceLock<Computer> = OnceLock::new();
    VALUE.get_or_init(|| {
        let output = Command::new("powershell.exe")
            .creation_flags(0x08000000)
            .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
                "$c=Get-CimInstance Win32_ComputerSystem; @{machine=(Get-ItemProperty 'HKLM:\\SOFTWARE\\Microsoft\\Cryptography').MachineGuid;brand=$c.Manufacturer;model=$c.Model}|ConvertTo-Json -Compress"])
            .output();
        output.ok().filter(|o| o.status.success())
            .and_then(|o| serde_json::from_slice(&o.stdout).ok()).unwrap_or_default()
    })
}

fn valid_id(id: &str) -> bool {
    id.len() == 36 && id.bytes().enumerate().all(|(i, b)| {
        if [8, 13, 18, 23].contains(&i) { b == b'-' } else { b.is_ascii_hexdigit() }
    })
}

fn identity() -> Result<String> {
    static LOCK: Mutex<()> = Mutex::new(());
    let _guard = LOCK.lock().map_err(|_| anyhow::anyhow!("Device identity lock failed"))?;
    let root = crate::utils::dirs::app_home_dir()?;
    fs::create_dir_all(&root)?;
    let path = root.join("subscription-device.json");
    let machine = &computer().machine;
    match fs::read(&path) {
        Ok(bytes) => {
            let saved: Identity = serde_json::from_slice(&bytes).context("Invalid device identity file")?;
            anyhow::ensure!(valid_id(&saved.id), "Invalid saved device UUID");
            if machine.is_empty() || saved.machine == *machine { return Ok(saved.id); }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| anyhow::anyhow!("Device identity generation failed: {e}"))?;
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let id = format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..]);
    let saved = Identity {machine: machine.clone(), id: id.clone()};
    fs::write(&path, serde_json::to_vec(&saved)?)?;
    Ok(id)
}

pub(crate) fn attach(url: &mut Url) -> Result<()> {
    if url.scheme() != "https" || !url.host_str().is_some_and(|host| HOSTS.contains(&host)) {
        return Ok(());
    }
    let id = identity()?;
    let computer = computer();
    let pairs: Vec<_> = url.query_pairs().filter(|(key, _)| {
        !["device_id", "device_brand", "device_model"].contains(&key.as_ref())
    }).map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
    let mut query = url.query_pairs_mut();
    query.clear().extend_pairs(pairs);
    query.append_pair("device_id", &id);
    query.append_pair("device_brand", if computer.brand.is_empty() {"Windows"} else {&computer.brand});
    query.append_pair("device_model", if computer.model.is_empty() {"Windows PC"} else {&computer.model});
    Ok(())
}
