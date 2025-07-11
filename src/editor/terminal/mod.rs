// Terminal 负责终端的初始化、清理、光标控制、行列输出、标题设置等底层操作。


use crossterm::{
    cursor::{Hide, MoveTo, Show},
    style::{
        Attribute::{Reset, Reverse},
        Print,
    },
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
        EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
    queue, Command,
};
use std::io::{stdout, Error, Write};
use crate::prelude::*;


pub struct Terminal;

impl Terminal {
    /// 终端清理与退出，恢复原始状态
    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::enable_line_wrap()?;
        Self::show_caret()?;
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }
    
    /// 初始化终端，进入备用屏幕、禁用自动换行、清屏
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::execute()?;
        Ok(())
    }

    /// 清空整个屏幕
    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))?;
        Ok(())
    }

    /// 清空当前行
    pub fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    /// 将插入符号移动到指定位置。
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))?;
        Ok(())
    }

    /// 进入备用屏幕缓冲区
    pub fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(EnterAlternateScreen)?;
        Ok(())
    }

    /// 离开备用屏幕缓冲区
    pub fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(LeaveAlternateScreen)?;
        Ok(())
    }

    /// 隐藏光标
    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)?;
        Ok(())
    }

    /// 显示光标
    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)?;
        Ok(())
    }

    /// 禁用自动换行
    pub fn disable_line_wrap() -> Result<(), Error> {
        Self::queue_command(DisableLineWrap)?;
        Ok(())
    }

    /// 启用自动换行
    pub fn enable_line_wrap() -> Result<(), Error> {
        Self::queue_command(EnableLineWrap)?;
        Ok(())
    }

    /// 设置终端标题
    pub fn set_title(title: &str) -> Result<(), Error> {
        Self::queue_command(SetTitle(title))?;
        Ok(())
    }

    /// 输出字符串到终端
    pub fn print(string: &str) -> Result<(), Error> {
        Self::queue_command(Print(string))?;
        Ok(())
    }

    /// 在指定行输出一行文本
    pub fn print_row(row: RowIdx, line_text: &str) -> Result<(), Error> {
        Self::move_caret_to(Position { row, col: 0 })?;
        Self::clear_line()?;
        Self::print(line_text)?;
        Ok(())
    }

    /// 在指定行输出反色文本（用于状态栏等）
    pub fn print_inverted_row(row: RowIdx, line_text: &str) -> Result<(), Error> {
        let width = Self::size()?.width;
        Self::print_row(row, &format!("{Reverse}{line_text:width$.width$}{Reset}"))
    }

    /// 获取当前终端尺寸（行数和列数）
    pub fn size() -> Result<Size, Error> {
        let (width_u16, height_u16) = size()?;
        let height = height_u16 as usize;
        let width = width_u16 as usize;
        Ok(Size { height, width })
    }
    /// 刷新终端输出缓冲区
    pub fn execute() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    /// 内部方法：将命令加入输出队列
    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)?;
        Ok(())
    }
}