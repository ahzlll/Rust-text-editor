// StatusBar 显示文件名、行数、修改状态和光标位置等信息。

use std::io::Error;
use crate::prelude::*;
use super::super::{DocumentStatus, Terminal};
use super::UIComponent;

/// 状态栏组件，负责显示文档状态信息
#[derive(Default)]
pub struct StatusBar {
    current_status: DocumentStatus, // 当前文档状态
    needs_redraw: bool,            // 是否需要重绘
    size: Size,                    // 状态栏尺寸
}

impl StatusBar {
    /// 更新状态栏显示的文档状态
    pub fn update_status(&mut self, new_status: DocumentStatus) {
        if new_status != self.current_status {
            self.current_status = new_status;
            self.set_needs_redraw(true);
        }
    }
}

impl UIComponent for StatusBar {
    /// 设置是否需要重绘
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    /// 判断是否需要重绘
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    /// 设置状态栏尺寸
    fn set_size(&mut self, size: Size) {
        self.size = size;
    }
    /// 绘制状态栏内容
    fn draw(&mut self, origin_row: RowIdx) -> Result<(), Error> {
        // 组装状态栏的第一部分
        let line_count = self.current_status.line_count_to_string();
        let modified_indicator = self.current_status.modified_indicator_to_string();

        let beginning = format!(
            "{} - {line_count} {modified_indicator}",
            self.current_status.file_name
        );

        // 组装后半部分
        let position_indicator = self.current_status.position_indicator_to_string();
        let back_part = format!("{position_indicator}");

        // 组装整个状态栏
        let remainder_len = self.size.width.saturating_sub(beginning.len());
        let status = format!("{beginning}{back_part:>remainder_len$}");

        // 仅在状态适合时打印状态。否则写出一个空字符串以确保清除行。
        let to_print = if status.len() <= self.size.width {
            status
        } else {
            String::new()
        };
        Terminal::print_inverted_row(origin_row, &to_print)?;

        Ok(())
    }
}
