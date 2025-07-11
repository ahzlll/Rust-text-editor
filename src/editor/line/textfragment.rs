// TextFragment 表示一行中的单个字素及其渲染属性

use crate::prelude::*;

use super::GraphemeWidth;

/// 文本片段，包含单个字素及其渲染信息
#[derive(Clone, Debug)]
pub struct TextFragment {
    pub grapheme: String,           // 当前字素内容（如字符、emoji等）
    pub rendered_width: GraphemeWidth, // 渲染宽度（终端显示宽度）
    pub start: ByteIdx,             // 在原始字符串中的起始字节位置
}
