// 负责文本内容的显示、编辑、滚动和光标管理。


use std::{cmp::min, io::Error};

use crate::editor::RowIdx;
use crate::prelude::*;

use crate::editor::{
    command::{Edit, Move},
    DocumentStatus, Line, Terminal,
};
use super::UIComponent;

mod buffer;
use buffer::Buffer;

mod fileinfo;
use fileinfo::FileInfo;

/// 编辑区主视图，管理文本缓冲区、滚动、光标等
#[derive(Default)]
pub struct View {
    buffer: Buffer,           // 文本缓冲区
    needs_redraw: bool,       // 是否需要重绘
    size: Size,               // 视图区尺寸
    text_location: Location,  // 当前文本位置（行、字素）
    scroll_offset: Position,  // 当前滚动偏移
}

impl View {
    /// 获取当前文档状态（文件名、行数、修改状态等）
    pub fn get_status(&self) -> DocumentStatus {
        let file_info = self.buffer.get_file_info();
        DocumentStatus {
            total_lines: self.buffer.height(),
            current_line_idx: self.text_location.line_idx,
            file_name: format!("{file_info}"),
            is_modified: self.buffer.is_dirty(),
        }
    }

    /// 判断是否已加载文件
    pub const fn is_file_loaded(&self) -> bool {
        self.buffer.is_file_loaded()
    }

    // 文件输入输出
    /// 加载文件内容到缓冲区
    pub fn load(&mut self, file_name: &str) -> Result<(), Error> {
        let buffer = Buffer::load(file_name)?;
        self.buffer = buffer;
        self.set_needs_redraw(true);
        Ok(())
    }
    /// 保存当前缓冲区内容到文件
    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()?;
        self.set_needs_redraw(true);
        Ok(())
    }
    /// 另存为新文件
    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        self.buffer.save_as(file_name)?;
        self.set_needs_redraw(true);
        Ok(())
    }

    // 命令处理
    /// 处理编辑命令（插入、删除、换行等）
    pub fn handle_edit_command(&mut self, command: Edit) {
        match command {
            Edit::Insert(character) => self.insert_char(character),
            Edit::Delete => self.delete(),
            Edit::DeleteBackward => self.delete_backward(),
            Edit::InsertNewline => self.insert_newline(),
        }
    }
    /// 处理移动命令（上下左右、翻页、行首行尾等）
    pub fn handle_move_command(&mut self, command: Move) {
        let Size { height, .. } = self.size;
        // 此匹配移动位置，但不检查所有边界。
        // 最终的边界检查发生在匹配语句之后。
        match command {
            Move::Up => self.move_up(1),
            Move::Down => self.move_down(1),
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::PageUp => self.move_up(height.saturating_sub(1)),
            Move::PageDown => self.move_down(height.saturating_sub(1)),
            Move::StartOfLine => self.move_to_start_of_line(),
            Move::EndOfLine => self.move_to_end_of_line(),
        }
        self.scroll_text_location_into_view();
    }

    // 文本编辑
    /// 插入换行
    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.text_location);
        self.handle_move_command(Move::Right);
        self.set_needs_redraw(true);
    }
    /// 向后删除字符
    fn delete_backward(&mut self) {
        if self.text_location.line_idx != 0 || self.text_location.grapheme_idx != 0 {
            self.handle_move_command(Move::Left);
            self.delete();
        }
    }
    /// 删除当前位置字符
    fn delete(&mut self) {
        self.buffer.delete(self.text_location);
        self.set_needs_redraw(true);
    }
    /// 插入字符
    fn insert_char(&mut self, character: char) {
        let old_len = self.buffer.grapheme_count(self.text_location.line_idx);
        self.buffer.insert_char(character, self.text_location);
        let new_len = self.buffer.grapheme_count(self.text_location.line_idx);
        let grapheme_delta = new_len.saturating_sub(old_len);
        if grapheme_delta > 0 {
            // 为添加的字符向右移动（应该是常规情况）
            self.handle_move_command(Move::Right);
        }
        self.set_needs_redraw(true);
    }

    // 渲染
    /// 渲染单行文本到指定行
    fn render_line(at: RowIdx, line_text: &str) -> Result<(), Error> {
        Terminal::print_row(at, line_text)
    }

    // 滚动
    /// 垂直滚动到指定行
    fn scroll_vertically(&mut self, to: RowIdx) {
        let Size { height, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };
        if offset_changed {
            self.set_needs_redraw(true);
        }
    }
    /// 水平滚动到指定列
    fn scroll_horizontally(&mut self, to: ColIdx) {
        let Size { width, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            true
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };
        if offset_changed {
            self.set_needs_redraw(true);
        }
    }
    /// 保证光标位置在可视区域内
    fn scroll_text_location_into_view(&mut self) {
        let Position { row, col } = self.text_location_to_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }
    /// 将光标居中
    fn center_text_location(&mut self) {
        let Size { height, width } = self.size;
        let Position { row, col } = self.text_location_to_position();
        let vertical_mid = height.div_ceil(2);
        let horizontal_mid = width.div_ceil(2);
        self.scroll_offset.row = row.saturating_sub(vertical_mid);
        self.scroll_offset.col = col.saturating_sub(horizontal_mid);
        self.set_needs_redraw(true);
    }

    // 位置和坐标处理
    /// 获取光标在终端中的实际位置
    pub fn caret_position(&self) -> Position {
        self.text_location_to_position()
            .saturating_sub(self.scroll_offset)
    }
    /// 将文本位置转换为终端坐标
    fn text_location_to_position(&self) -> Position {
        let row = self.text_location.line_idx;
        debug_assert!(row.saturating_sub(1) <= self.buffer.height());
        let col = self
            .buffer
            .width_until(row, self.text_location.grapheme_idx);
        Position { col, row }
    }

    // 文本位置移动
    /// 向上移动指定行数
    fn move_up(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }
    /// 向下移动指定行数
    fn move_down(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }
    /// 向右移动一个字素
    fn move_right(&mut self) {
        let grapheme_count = self.buffer.grapheme_count(self.text_location.line_idx);
        if self.text_location.grapheme_idx < grapheme_count {
            self.text_location.grapheme_idx += 1;
        } else {
            self.move_to_start_of_line();
            self.move_down(1);
        }
    }
    /// 向左移动一个字素
    fn move_left(&mut self) {
        if self.text_location.grapheme_idx > 0 {
            self.text_location.grapheme_idx -= 1;
        } else if self.text_location.line_idx > 0 {
            self.move_up(1);
            self.move_to_end_of_line();
        }
    }
    /// 移动到行首
    fn move_to_start_of_line(&mut self) {
        self.text_location.grapheme_idx = 0;
    }
    /// 移动到行尾
    fn move_to_end_of_line(&mut self) {
        self.text_location.grapheme_idx = self.buffer.grapheme_count(self.text_location.line_idx);
    }

    // 保证光标位置和行号有效
    /// 校正光标字素索引到有效范围
    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_idx = min(
            self.text_location.grapheme_idx,
            self.buffer.grapheme_count(self.text_location.line_idx),
        );
    }
    /// 校正行号到有效范围
    fn snap_to_valid_line(&mut self) {
        self.text_location.line_idx = min(self.text_location.line_idx, self.buffer.height());
    }
}

impl UIComponent for View {
    /// 设置是否需要重绘
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    /// 判断是否需要重绘
    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }
    /// 设置视图区尺寸
    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.scroll_text_location_into_view();
    }
    /// 绘制编辑区内容
    fn draw(&mut self, origin_row: RowIdx) -> Result<(), Error> {
        let Size { height, width } = self.size;
        let end_y = origin_row.saturating_add(height);
        let scroll_top = self.scroll_offset.row;

        for current_row in origin_row..end_y {
            let line_idx = current_row
                .saturating_sub(origin_row)
                .saturating_add(scroll_top);
            let left = self.scroll_offset.col;
            let right = self.scroll_offset.col.saturating_add(width);
            if let Some(line) = self.buffer.get_line(line_idx) {
                let text = line.get_visible_graphemes(left..right);
                Self::render_line(current_row, &text)?;
            } else {
                Self::render_line(current_row, "_")?;
            }
        }
        Ok(())
    }
}