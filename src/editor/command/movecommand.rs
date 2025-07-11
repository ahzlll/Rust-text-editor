// Move 处理上下左右等移动操作。

use crossterm::event::{
    KeyCode::{Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};

/// 光标移动命令枚举，表示各种方向和范围的移动
#[derive(Clone, Copy)]
pub enum Move {
    PageUp,       // 向上翻页
    PageDown,     // 向下翻页
    StartOfLine,  // 移动到行首
    EndOfLine,    // 移动到行尾
    Up,           // 向上移动一行
    Left,         // 向左移动一列
    Right,        // 向右移动一列
    Down,         // 向下移动一行
}

impl TryFrom<KeyEvent> for Move {
    type Error = String;
    /// 将 KeyEvent 转换为 Move 移动命令
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::NONE {
            match code {
                Up => Ok(Self::Up),
                Down => Ok(Self::Down),
                Left => Ok(Self::Left),
                Right => Ok(Self::Right),
                PageDown => Ok(Self::PageDown),
                PageUp => Ok(Self::PageUp),
                Home => Ok(Self::StartOfLine),
                End => Ok(Self::EndOfLine),
                _ => Err(format!("Unsupported code: {code:?}")),
            }
        } else {
            Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ))
        }
    }
}