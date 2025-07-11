// Line 表示一行文本及其字素分片，支持插入、删除、拼接、分割等操作。

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use std::{
    collections::VecDeque,
    fmt::{self, Display},
    ops::{Deref, Range},
};
use crate::prelude::*;

mod graphemewidth;
use graphemewidth::GraphemeWidth;

mod textfragment;
use textfragment::TextFragment;

/// 行结构体，包含文本内容和分片信息
#[derive(Default, Clone)]
pub struct Line {
    fragments: Vec<TextFragment>, // fragments（文本片段向量）
    string: String, // string（字符串）
}

impl Line {
    /// 通过字符串构建一个 Line 实例
    pub fn from(line_str: &str) -> Self {
        debug_assert!(line_str.is_empty() || line_str.lines().count() == 1);
        let fragments = Self::str_to_fragments(line_str);
        Self {
            fragments,
            string: String::from(line_str),
        }
    }

    /// 字符串转换为文本片段的向量
    /// 每个片段包含 grapheme（字素）、rendered_width（渲染宽度）、start（开始位置）
    fn str_to_fragments(line_str: &str) -> Vec<TextFragment> {
        line_str
            .grapheme_indices(true)
            .map(|(byte_idx, grapheme)| {
                let (_replacement, rendered_width) = Self::get_replacement_character(grapheme)
                    .map_or_else(
                        || {
                            let unicode_width = grapheme.width();
                            let rendered_width = match unicode_width {
                                0 | 1 => GraphemeWidth::Half,
                                _ => GraphemeWidth::Full,
                            };
                            (None, rendered_width)
                        },
                        |replacement| (Some(replacement), GraphemeWidth::Half),
                    );

                TextFragment {
                    grapheme: grapheme.to_string(),
                    rendered_width,
                    start: byte_idx,
                }
            })
            .collect()
    }
   
    /// 重新构建分片信息
    fn rebuild_fragments(&mut self) {
        self.fragments = Self::str_to_fragments(&self.string);
    }

    /// 根据输入字符串返回一个替代字符，用于表示特定的控制字符或空白字符
    fn get_replacement_character(for_str: &str) -> Option<char> {
        let width = for_str.width();
        match for_str {
            " " => None,
            "\t" => Some(' '),
            _ if width > 0 && for_str.trim().is_empty() => Some('␣'),
            _ if width == 0 => {
                let mut chars = for_str.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        return Some('▯');
                    }
                }
                Some('·')
            }
            _ => None,
        }
    }

    /// 获取给定列索引中可见的字素。
    /// 只保留基础字符串截取
    pub fn get_visible_graphemes(&self, range: Range<ColIdx>) -> String {
        // 假设 range.start/end 是字素索引，直接用 grapheme_indices 截取
        let graphemes: Vec<&str> = self.string.graphemes(true).collect();
        let start = range.start.min(graphemes.len());
        let end = range.end.min(graphemes.len());
        graphemes[start..end].concat()
    }

    /// 返回行中的字素数量
    pub fn grapheme_count(&self) -> GraphemeIdx {
        self.fragments.len()
    }

    /// 计算直到指定字素的列宽
    pub fn width_until(&self, grapheme_idx: GraphemeIdx) -> ColIdx {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| match fragment.rendered_width {
                GraphemeWidth::Half => 1,
                GraphemeWidth::Full => 2,
            })
            .sum()
    }

    /// 返回整行的列宽
    pub fn width(&self) -> ColIdx {
        self.width_until(self.grapheme_count())
    }

    /// 在指定字素索引处插入字符
    /// 将一个字符插入到行中，或者如果 at == grapheme_count + 1，则将其附加到行尾
    pub fn insert_char(&mut self, character: char, at: GraphemeIdx) {
        debug_assert!(at.saturating_sub(1) <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            self.string.insert(fragment.start, character);
        } else {
            self.string.push(character);
        }
        self.rebuild_fragments();
    }

    /// 追加字符
    pub fn append_char(&mut self, character: char) {
        self.insert_char(character, self.grapheme_count());
    }

    /// 删除指定字素索引处的字符
    pub fn delete(&mut self, at: GraphemeIdx) {
        debug_assert!(at <= self.grapheme_count());
        if let Some(fragment) = self.fragments.get(at) {
            let start = fragment.start;
            let end = fragment.start.saturating_add(fragment.grapheme.len());
            self.string.drain(start..end);
            self.rebuild_fragments();
        }
    }

    /// 删除行末尾的字符
    pub fn delete_last(&mut self) {
        self.delete(self.grapheme_count().saturating_sub(1));
    }

    /// 将另一行的内容附加到当前行，并更新 fragments
    pub fn append(&mut self, other: &Self) {
        self.string.push_str(&other.string);
        self.rebuild_fragments();
    }

    /// 在指定字素索引处拆分行，并返回拆分后的剩余部分
    pub fn split(&mut self, at: GraphemeIdx) -> Self {
        if let Some(fragment) = self.fragments.get(at) {
            let remainder = self.string.split_off(fragment.start);
            self.rebuild_fragments();
            Self::from(&remainder)
        } else {
            Self::default()
        }
    }

    /// 将字节索引转换为字素索引
    fn byte_idx_to_grapheme_idx(&self, byte_idx: ByteIdx) -> Option<GraphemeIdx> {
        if byte_idx > self.string.len() {
            return None;
        }
        self.fragments
            .iter()
            .position(|fragment| fragment.start >= byte_idx)
    }

    /// 将字素索引转换为字节索引
    fn grapheme_idx_to_byte_idx(&self, grapheme_idx: GraphemeIdx) -> ByteIdx {
        debug_assert!(grapheme_idx <= self.grapheme_count());
        if grapheme_idx == 0 || self.grapheme_count() == 0 {
            return 0;
        }
        self.fragments.get(grapheme_idx).map_or_else(
            || {
                #[cfg(debug_assertions)]
                {
                    panic!("Fragment not found for grapheme index: {grapheme_idx:?}");
                }
                #[cfg(not(debug_assertions))]
                {
                    0
                }
            },
            |fragment| fragment.start,
        )
    }
}

impl Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.string)
    }
}

impl Deref for Line {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.string
    }
}