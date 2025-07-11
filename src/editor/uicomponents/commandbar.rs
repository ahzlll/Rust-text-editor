// CommandBar 用于处理保存文件时的文件名输入等命令行交互。

use std::{cmp::min, io::Error};
use unicode_width::UnicodeWidthStr;

use crate::prelude::*;

use super::super::{command::Edit, Line, Terminal};
use super::UIComponent;

/// 处理底部命令输入（如保存文件名）
#[derive(Default)]
pub struct CommandBar {
    prompt: String,    // 提示符内容
    value: Line,       // 用户输入内容
    needs_redraw: bool,// 是否需要重绘
    size: Size,        // 组件尺寸
}

impl CommandBar {
    /// 处理编辑命令（插入、删除等）
    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(character) => self.value.append_char(character),
            Edit::Delete | Edit::InsertNewline => {}
            Edit::DeleteBackward => self.value.delete_last(),
        }
        self.set_needs_redraw(true);
    }

    /// 获取光标在命令栏中的列位置
    pub fn caret_position_col(&self) -> ColIdx {
        let prompt_width = UnicodeWidthStr::width(self.prompt.as_str());
        let value_width = UnicodeWidthStr::width(self.value.to_string().as_str());
        // 计算提示符和输入值的实际显示宽度
        let max_width = prompt_width + value_width;
        // 限制光标位置在可显示宽度范围内
        min(max_width, self.size.width)
    }

    /// 获取当前输入的字符串
    pub fn value(&self) -> String {
        self.value.to_string()
    }

    /// 设置命令栏提示符内容
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
        self.set_needs_redraw(true);
    }

    /// 清空命令栏输入内容
    pub fn clear_value(&mut self) {
        self.value = Line::default();
        self.set_needs_redraw(true);
    }
}

impl UIComponent for CommandBar {
    /// 设置是否需要重绘
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    /// 判断是否需要重绘
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    /// 设置命令栏尺寸
    fn set_size(&mut self, size: Size) {
        self.size = size;
    }
    /// 绘制命令栏内容
    fn draw(&mut self, origin: RowIdx) -> Result<(), Error> {
        let area_for_value = self.size.width.saturating_sub(self.prompt.len()); 
        let value_end = self.value.width(); 
        let value_start = value_end.saturating_sub(area_for_value); 
    
        let visible_value = self.value.get_visible_graphemes(value_start..value_end);
    
        let message = format!("{}{}", self.prompt, visible_value);
        let to_print = if message.len() <= self.size.width {
            message
        } else {
            // 如果提示符和值的组合长度超过了可显示区域的宽度，只显示提示符
            format!("{}{}", self.prompt, &visible_value)
        };
    
        Terminal::print_row(origin, &to_print)
    }    
}
