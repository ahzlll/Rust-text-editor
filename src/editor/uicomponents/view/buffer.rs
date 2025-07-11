// Buffer 负责管理编辑区的所有文本内容、文件信息和脏标记。


use super::FileInfo;
use super::Line;
use crate::prelude::*;
use std::fs::{read_to_string, File};
use std::io::Error;
use std::io::Write;

/// 文本缓冲区，管理所有文本行、文件信息和脏标记
#[derive(Default)]
pub struct Buffer {
    lines: Vec<Line>,      // 文本行集合
    file_info: FileInfo,   // 文件信息
    dirty: bool,           // 是否有未保存修改
}

impl Buffer {
    /// 判断缓冲区是否有未保存修改
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }
    /// 获取文件信息
    pub const fn get_file_info(&self) -> &FileInfo {
        &self.file_info
    }
    /// 获取指定行的字素数
    pub fn grapheme_count(&self, idx: LineIdx) -> GraphemeIdx {
        self.lines.get(idx).map_or(0, Line::grapheme_count)
    }
    /// 获取指定行到某字素的宽度
    pub fn width_until(&self, idx: LineIdx, until: GraphemeIdx) -> GraphemeIdx {
        self.lines
            .get(idx)
            .map_or(0, |line| line.width_until(until))
    }
    /// 加载文件内容到缓冲区
    pub fn load(file_name: &str) -> Result<Self, Error> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        Ok(Self {
            lines,
            file_info: FileInfo::from(file_name),
            dirty: false,
        })
    }
    /// 保存内容到指定文件
    fn save_to_file(&self, file_info: &FileInfo) -> Result<(), Error> {
        if let Some(file_path) = &file_info.get_path() {
            let mut file = File::create(file_path)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        } else {
            #[cfg(debug_assertions)]
            {
                panic!("Attempting to save with no file path present");
            }
        }
        Ok(())
    }
    /// 另存为新文件
    pub fn save_as(&mut self, file_name: &str) -> Result<(), Error> {
        let file_info = FileInfo::from(file_name);
        self.save_to_file(&file_info)?;
        self.file_info = file_info;
        self.dirty = false;
        Ok(())
    }
    /// 保存到当前文件
    pub fn save(&mut self) -> Result<(), Error> {
        self.save_to_file(&self.file_info)?;
        self.dirty = false;
        Ok(())
    }
    /// 判断缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    /// 判断是否已加载文件
    pub const fn is_file_loaded(&self) -> bool {
        self.file_info.has_path()
    }
    /// 获取文本行数
    pub fn height(&self) -> LineIdx {
        self.lines.len()
    }
    /// 在指定位置插入字符
    pub fn insert_char(&mut self, character: char, at: Location) {
        debug_assert!(at.line_idx <= self.height());
        if at.line_idx == self.height() {
            self.lines.push(Line::from(&character.to_string()));
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            line.insert_char(character, at.grapheme_idx);
            self.dirty = true;
        }
    }
    /// 在指定位置删除字符或合并行
    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.grapheme_idx >= line.grapheme_count()
                && self.height() > at.line_idx.saturating_add(1)
            {
                let next_line = self.lines.remove(at.line_idx.saturating_add(1));
                self.lines[at.line_idx].append(&next_line);
                self.dirty = true;
            } else if at.grapheme_idx < line.grapheme_count() {
                self.lines[at.line_idx].delete(at.grapheme_idx);
                self.dirty = true;
            }
        }
    }
    /// 在指定位置插入换行
    pub fn insert_newline(&mut self, at: Location) {
        if at.line_idx == self.height() {
            self.lines.push(Line::default());
            self.dirty = true;
        } else if let Some(line) = self.lines.get_mut(at.line_idx) {
            let new = line.split(at.grapheme_idx);
            self.lines.insert(at.line_idx.saturating_add(1), new);
            self.dirty = true;
        }
    }
    /// 获取指定行的引用
    pub fn get_line(&self, idx: usize) -> Option<&Line> {
        self.lines.get(idx)
    }
}
