use crate::ssh::{SSHConnection, SSHTunnel};
use crate::settings::AppConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    pub connections: HashMap<String, SSHConnection>,
    pub tunnels: HashMap<String, SSHTunnel>,
    pub settings: AppConfig,
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
            tunnels: HashMap::new(),
            settings: AppConfig::default(),
        }
    }
}

pub struct DataManager {
    data_path: PathBuf,
}

impl DataManager {
    pub fn new() -> Result<Self, String> {
        // 使用固定的数据目录路径
        let data_path = dirs::data_dir()
            .ok_or("Failed to get data directory")?
            .join("vesper");

        Ok(Self {
            data_path,
        })
    }

    fn get_data_file_path(&self) -> PathBuf {
        self.data_path.join("data.json")
    }

    // Synchronous version for blocking operations
    fn load_data_sync(&self) -> Result<AppData, String> {
        let file_path = self.get_data_file_path();

        if !file_path.exists() {
            // 如果文件不存在，返回默认数据
            return Ok(AppData::default());
        }

        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read data file: {}", e))?;

        let data: AppData = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse data file: {}", e))?;

        Ok(data)
    }

    // Synchronous version for blocking operations
    fn save_data_sync(&self, data: &AppData) -> Result<(), String> {
        let file_path = self.get_data_file_path();
        let backup_path = self.data_path.join("data.json.bak");

        // 创建数据目录（如果不存在）
        if !self.data_path.exists() {
            fs::create_dir_all(&self.data_path)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        // 创建备份（如果主文件存在）
        if file_path.exists() {
            if let Err(e) = fs::copy(&file_path, &backup_path) {
                eprintln!("Warning: Failed to create backup: {}", e);
            }
        }

        let content = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize data: {}", e))?;

        // 写入临时文件，然后原子性移动
        let temp_path = file_path.with_extension("tmp");
        fs::write(&temp_path, content)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        // 原子性移动
        fs::rename(&temp_path, &file_path)
            .map_err(|e| format!("Failed to move temp file: {}", e))?;

        // 删除备份
        let _ = fs::remove_file(backup_path);

        Ok(())
    }

    // Async wrapper that runs blocking operations in a separate thread
    pub async fn load_data(&self) -> Result<AppData, String> {
        let data_path = self.data_path.clone();
        tokio::task::spawn_blocking(move || {
            // Create a new DataManager in the blocking thread
            let manager = DataManager { data_path };
            manager.load_data_sync()
        })
        .await
        .map_err(|e| format!("Failed to join blocking task: {}", e))?
    }

    pub async fn save_data(&self, data: AppData) -> Result<(), String> {
        let data_path = self.data_path.clone();

        tokio::task::spawn_blocking(move || {
            // Create a new DataManager in the blocking thread
            let manager = DataManager { data_path };
            manager.save_data_sync(&data)
        })
        .await
        .map_err(|e| format!("Failed to join blocking task: {}", e))?
    }

    // 便利方法：直接保存连接和隧道
    pub async fn save_connections_and_tunnels(
        &self,
        connections: &HashMap<String, SSHConnection>,
        tunnels: &HashMap<String, SSHTunnel>,
    ) -> Result<(), String> {
        let mut data = self.load_data().await?;
        data.connections = connections.clone();
        data.tunnels = tunnels.clone();
        self.save_data(data).await
    }

    // 便利方法：加载连接和隧道
    pub async fn load_connections_and_tunnels(
        &self,
    ) -> Result<(HashMap<String, SSHConnection>, HashMap<String, SSHTunnel>), String> {
        let data = self.load_data().await?;
        Ok((data.connections, data.tunnels))
    }

    // 便利方法：保存设置
    pub async fn save_settings(&self, settings: &AppConfig) -> Result<(), String> {
        let mut data = self.load_data().await?;
        data.settings = settings.clone();
        self.save_data(data).await
    }

    // 便利方法：加载设置
    pub async fn load_settings(&self) -> Result<AppConfig, String> {
        let data = self.load_data().await?;
        Ok(data.settings)
    }
}