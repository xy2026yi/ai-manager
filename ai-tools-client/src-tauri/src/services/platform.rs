use anyhow::Result;

pub struct Platform;

impl Platform {
    /// 获取当前平台类型
    pub fn get_platform_type() -> String {
        #[cfg(target_os = "windows")]
        return "windows".to_string();

        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "freebsd"))]
        return "unix".to_string();
    }

    /// 判断是否为Windows平台
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// 判断是否为Unix-like平台
    pub fn is_unix() -> bool {
        cfg!(any(target_os = "macos", target_os = "linux", target_os = "freebsd"))
    }

    /// 获取用户主目录
    pub fn get_home_dir() -> Result<String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?
            .to_string_lossy()
            .to_string();
        Ok(home_dir)
    }

    /// 获取应用数据目录
    pub fn get_app_data_dir() -> Result<String> {
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取应用数据目录"))?
            .join("ai-tools-client")
            .to_string_lossy()
            .to_string();
        Ok(app_data_dir)
    }

    /// 获取配置目录
    pub fn get_config_dir() -> Result<String> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?
            .join("ai-tools-client")
            .to_string_lossy()
            .to_string();
        Ok(config_dir)
    }
}