use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct FileOperations;

impl FileOperations {
    /// 读取文件内容
    pub async fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        let content = fs::read_to_string(path)?;
        Ok(content)
    }

    /// 写入文件内容
    pub async fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        fs::write(path, content)?;
        Ok(())
    }

    /// 检查文件是否存在
    pub async fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        Path::new(path.as_ref()).exists()
    }

    /// 创建目录
    pub async fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    /// 复制文件
    pub async fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        fs::copy(from, to)?;
        Ok(())
    }

    /// 删除文件
    pub async fn delete_file<P: AsRef<Path>>(path: P) -> Result<()> {
        fs::remove_file(path)?;
        Ok(())
    }
}