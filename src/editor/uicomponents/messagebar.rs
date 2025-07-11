// MessageBar 用于在编辑器底部显示临时提示信息。

use std::{
    io::Error,
    time::{Duration, Instant},
};

use crate::prelude::*;
use super::super::Terminal;
use super::UIComponent;

/// 默认消息显示时长（ 10秒）
const DEFAULT_DURATION: Duration = Duration::new(10, 0);


struct Message {
    text: String,   // 消息内容
    time: Instant,  // 消息生成时间
}
impl Default for Message {
    fn default() -> Self {
        Self {
            text: String::new(),
            time: Instant::now(),
        }
    }
}

impl Message {
    /// 判断消息是否已过期
    fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.time) > DEFAULT_DURATION
    }
}

/// 消息栏组件
#[derive(Default)]
pub struct MessageBar {
    current_message: Message,   // 当前显示的消息
    needs_redraw: bool,        // 是否需要重绘
    cleared_after_expiry: bool, // 确保过期消息被正确清除
}

impl MessageBar {
    /// 更新消息栏内容，并重置计时
    pub fn update_message(&mut self, new_message: &str) {
        self.current_message = Message {
            text: new_message.to_string(),
            time: Instant::now(),
        };
        self.cleared_after_expiry = false;
        self.set_needs_redraw(true);
    }
}

impl UIComponent for MessageBar {
    /// 设置是否需要重绘
    fn set_needs_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }
    /// 判断是否需要重绘（消息过期或主动请求重绘时）
    fn needs_redraw(&self) -> bool {
        (!self.cleared_after_expiry && self.current_message.is_expired()) || self.needs_redraw
    }
    /// 消息栏无需调整尺寸，空实现
    fn set_size(&mut self, _size: Size) {}
    /// 绘制消息栏内容，过期时清空
    fn draw(&mut self, origin: RowIdx) -> Result<(), Error> {
        if self.current_message.is_expired() {
            self.cleared_after_expiry = true; // 过期时，写出 "" 一次以清除消息
        }
        let message = if self.current_message.is_expired() {
            ""
        } else {
            &self.current_message.text
        };

        Terminal::print_row(origin, message)
    }
}
