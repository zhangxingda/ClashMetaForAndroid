#[cfg(windows)]
mod utils {
    pub mod dirs {
        pub fn app_home_dir() -> anyhow::Result<std::path::PathBuf> {
            Ok(std::env::temp_dir().join(format!("verge-device-check-{}", std::process::id())))
        }
    }
}

#[cfg(windows)]
mod subscription_device {
    include!("../../subscription_device.rs");
}

#[cfg(all(test, windows))]
mod tests {
    use super::{subscription_device, utils};
    use reqwest::Url;

    #[test]
    fn subscription_metadata_persists_and_changes_with_computer() {
        let root = utils::dirs::app_home_dir().unwrap();
        let _ = std::fs::remove_dir_all(&root);
        let mut unrelated = Url::parse("https://example.com/sub?token=customer").unwrap();
        subscription_device::attach(&mut unrelated).unwrap();
        assert!(!root.exists());
        assert_eq!(unrelated.as_str(), "https://example.com/sub?token=customer");
        let mut insecure = Url::parse("http://vip1.959621.xyz/sub?token=customer").unwrap();
        subscription_device::attach(&mut insecure).unwrap();
        assert!(!root.exists());
        let mut first = Url::parse("https://vip1.959621.xyz/sub?token=customer&extra=one&extra=two&device_id=old#fragment").unwrap();
        subscription_device::attach(&mut first).unwrap();
        let pairs: Vec<_> = first.query_pairs().collect();
        let id = pairs.iter().find(|(k, _)| k == "device_id").unwrap().1.to_string();
        assert_eq!(id.len(), 36);
        assert_eq!(&id[14..15], "4");
        assert_eq!(pairs.iter().filter(|(k, _)| k == "device_id").count(), 1);
        assert_eq!(pairs.iter().filter(|(k, _)| k == "extra").count(), 2);
        assert!(pairs.iter().any(|(k, v)| k == "token" && v == "customer"));
        assert!(pairs.iter().any(|(k, v)| k == "device_brand" && !v.is_empty()));
        assert!(pairs.iter().any(|(k, v)| k == "device_model" && !v.is_empty()));
        assert_eq!(first.fragment(), Some("fragment"));
        for host in ["vip2027-1.pages.dev", "vip1.959621.xyz", "vip2-x3w.pages.dev", "vip2.959621.xyz", "vip4-2jc.pages.dev", "vip4.959621.xyz", "vip-5.pages.dev", "vip5.959621.xyz", "vip7-aiz.pages.dev", "vip7.959621.xyz", "v-ip9.pages.dev", "vip9.959621.xyz"] {
            let mut url = Url::parse(&format!("https://{host}/sub?token=customer")).unwrap();
            subscription_device::attach(&mut url).unwrap();
            assert_eq!(url.query_pairs().find(|(k, _)| k == "device_id").unwrap().1, id);
        }
        let path = root.join("subscription-device.json");
        let mut record: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert!(!record["machine"].as_str().unwrap().is_empty(), "Windows machine identity must be available");
        record["machine"] = serde_json::json!("another-computer");
        std::fs::write(&path, serde_json::to_vec(&record).unwrap()).unwrap();
        subscription_device::attach(&mut first).unwrap();
        assert_ne!(first.query_pairs().find(|(k, _)| k == "device_id").unwrap().1, id);
        std::fs::remove_dir_all(root).unwrap();
    }
}
