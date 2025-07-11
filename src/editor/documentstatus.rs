// DocumentStatus 记录文档的总行数、当前行、是否已修改、文件名，并提供格式化显示方法。

use crate::prelude::*;

/// 文档状态信息
#[derive(Default, Eq, PartialEq, Debug)]
pub struct DocumentStatus {
    /// 文档总行数
    pub total_lines: usize,
    /// 当前行号
    pub current_line_idx: LineIdx,
    /// 是否已修改
    pub is_modified: bool,
    /// 文件名
    pub file_name: String,
}

impl DocumentStatus {
    /// 返回“(modified)”或空字符串，指示文档是否被修改
    pub fn modified_indicator_to_string(&self) -> String {
        if self.is_modified {
            "(modified)".to_string()
        } else {
            String::new()
        }
    }
    /// 返回“xx lines”格式的总行数字符串
    pub fn line_count_to_string(&self) -> String {
        format!("{} lines", self.total_lines)
    }
    /// 返回“当前行/总行”格式的光标位置字符串
    pub fn position_indicator_to_string(&self) -> String {
        format!(
            "{}/{}",
            self.current_line_idx.saturating_add(1),
            self.total_lines
        )
    }
}
