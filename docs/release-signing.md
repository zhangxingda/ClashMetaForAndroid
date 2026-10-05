# 固定 APK 签名

打包工作流使用四个 GitHub Actions Secrets：`SIGNING_KEYSTORE_BASE64`、`SIGNING_STORE_PASSWORD`、`SIGNING_KEY_ALIAS`、`SIGNING_KEY_PASSWORD`。同一套密钥必须长期保持不变。不要将私钥或密码提交到仓库，也不要把私钥作为 Actions 构建附件上传。

`python3 scripts/prepare-release-signing.py` 校验密钥和别名，生成忽略的 `release-signing.keystore` 与 `signing.properties`，保留仓库原有 `release.keystore`。配置脚本不会输出凭据。手动/推送构建缺少 Secrets 时直接失败，避免悄悄使用临时 debug 签名。PR 构建不读取 Secrets，不用于正式发布。任务完成后删除生成的签名文件。

版本名更新为 2.11.36；签名构建使用从 2020-01-01 UTC 起的秒数作为 Android versionCode。同一时间点的重试不会降低版本号；后续时间点的构建会递增。版本名可继续按实际功能更新。

第一次从旧 debug 签名切换到正式密钥可能需要卸载重装，设备 ID 随之变化；之后应一直使用相同的应用 ID、签名密钥与递增的 versionCode。密钥与密码必须在独立安全存储中备份；GitHub Secrets 不能导出恢复，不能作为唯一备份。

本任务生成的密钥及恢复资料保存在云工作区外于源码目录的受限路径 `/workspace/.signing/ClashMetaForAndroid`，目录权限 0700，文件权限 0600。该路径只存在于此云环境，尚未验证独立备份或新环境恢复。不要删除或重置环境后才考虑备份。
