import json
import pathlib
import shutil
import sys

root = pathlib.Path(sys.argv[1])
module = root / 'src-tauri/src/utils/mod.rs'
module.write_text(module.read_text() + '\n#[cfg(target_os = "windows")]\npub(crate) mod subscription_device;\n')
profile = root / 'src-tauri/src/config/prfitem.rs'
source = profile.read_text()
marker = '        let url = fix_dirty_url(url)?;'
assert source.count(marker) == 1, 'Upstream subscription entry changed'
source = source.replace(marker, '''        let url = fix_dirty_url(url)?;
        #[cfg(target_os = "windows")]
        let url = {
            let mut enriched = url.clone();
            let result = tokio::task::spawn_blocking(move || {
                crate::utils::subscription_device::attach(&mut enriched)?;
                Ok::<_, anyhow::Error>(enriched)
            }).await;
            match result {
                Ok(Ok(enriched)) => enriched,
                _ => url,
            }
        };''')
profile.write_text(source)
shutil.copyfile(pathlib.Path(__file__).with_name('subscription_device.rs'), root / 'src-tauri/src/utils/subscription_device.rs')
config = root / 'src-tauri/tauri.conf.json'
data = json.loads(config.read_text())
data['bundle']['createUpdaterArtifacts'] = False
data['plugins']['updater']['endpoints'] = []
config.write_text(json.dumps(data, indent=2) + '\n')
print('Windows subscription metadata patch applied; official auto-updater endpoints disabled')
