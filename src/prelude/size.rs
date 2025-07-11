// Size 用于表示终端或区域的宽高。

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}
