// Location 用于表示文本中的行和字素位置

use super::{GraphemeIdx, LineIdx};

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub grapheme_idx: GraphemeIdx,
    pub line_idx: LineIdx,
}
