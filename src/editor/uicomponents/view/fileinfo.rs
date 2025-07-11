// FileInfo 用于管理和显示当前编辑文件的路径和名称。

use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
};

/// 文件信息结构体，保存文件路径
#[derive(Default, Debug)]
pub struct FileInfo {
    path: Option<PathBuf>, // 文件路径
}

impl FileInfo {
    /// 通过文件名创建 FileInfo
    pub fn from(file_name: &str) -> Self {
        let path = PathBuf::from(file_name);
        Self {
            path: Some(path),
        }
    }
    /// 获取文件路径
    pub fn get_path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    /// 判断是否有有效路径
    pub const fn has_path(&self) -> bool {
        self.path.is_some()
    }
}

impl Display for FileInfo {
    /// 显示文件名（无文件则显示[default]）
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .get_path()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("[default]");
        write!(formatter, "{name}")
    }
}
