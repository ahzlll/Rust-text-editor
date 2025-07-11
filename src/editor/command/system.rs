// System 处理保存、退出、调整大小、取消等系统级命令。

use crate::prelude::*;
use crossterm::event::{
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};

/// 系统命令枚举，表示保存、调整大小、退出、取消等操作
#[derive(Clone, Copy)]
pub enum System {
    Save,         // 保存文件
    Resize(Size), // 调整终端大小
    Quit,         // 退出编辑器
    Dismiss,      // 取消/关闭当前操作
}

impl TryFrom<KeyEvent> for System {
    type Error = String;
    /// 将键盘事件转换为系统命令
    fn try_from(event: KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;

        if modifiers == KeyModifiers::CONTROL {
            match code {
                Char('q') => Ok(Self::Quit),   // Ctrl+Q 退出
                Char('s') => Ok(Self::Save),   // Ctrl+S 保存
                _ => Err(format!("Unsupported CONTROL+{code:?} combination")),
            }
        } else if modifiers == KeyModifiers::NONE && matches!(code, KeyCode::Esc) {
            Ok(Self::Dismiss) // Esc 取消
        } else {
            Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ))
        }
    }
}