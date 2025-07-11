// 主要功能：
//   - 终端初始化与清理
//   - 主事件循环与输入处理
//   - 文件的加载、保存、退出
//   - 状态栏、消息栏、命令栏的统一管理
//   - 编辑区的渲染与状态刷新

use crossterm::event::{read, Event, KeyEvent, KeyEventKind};
use std::{
    env,
    io::Error,
    panic::{set_hook, take_hook},
};
use crate::prelude::*;

mod command;
use command::{
    Command::{self, Edit, Move, System},
    Edit::InsertNewline,
    Move::{Down, Left, Right, Up},
    System::{Dismiss, Quit, Resize, Save},
};

mod line;
use line::Line;

mod terminal;
use terminal::Terminal;

mod uicomponents;
use uicomponents::{View, CommandBar, MessageBar, StatusBar, UIComponent};

mod documentstatus;
use documentstatus::DocumentStatus;

const QUIT_TIMES: u8 = 3;

/// 编辑器提示类型（仅支持保存提示）
#[derive(Eq, PartialEq, Default)]
enum PromptType {
    Save,
    #[default]
    None,
}

impl PromptType {
    /// 判断当前是否为提示模式
    fn is_prompt(&self) -> bool {
        matches!(self, Self::Save)
    }
}

/// 编辑器主结构体，包含所有核心组件和状态
#[derive(Default)]
pub struct Editor {
    should_quit: bool,      // 是否应退出
    view: View,             // 编辑区视图
    status_bar: StatusBar,  // 状态栏
    message_bar: MessageBar,// 消息栏
    command_bar: CommandBar,// 命令栏
    prompt_type: PromptType,// 当前提示类型
    terminal_size: Size,    // 终端尺寸
    title: String,          // 终端标题
    quit_times: u8,         // 退出确认计数
}

impl Editor {
    /// 初始化 panic hook，确保异常时终端能正确恢复
    fn initialize_panic_hook() {
        let current_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
    }

    /// 初始化编辑器，包括终端、文件加载、状态栏等
    pub fn new() -> Result<Self, Error> {
        Self::initialize_panic_hook();
        // 初始化终端
        Terminal::initialize()?;

        let mut editor = Self::default();
        let size = Terminal::size().unwrap_or_default();
        editor.handle_resize_command(size);
        editor.update_message("Ctrl + S = 保存 | Ctrl + Q = 退出");

        let args: Vec<String> = env::args().collect();
        if let Some(file_name) = args.get(1) {
            debug_assert!(!file_name.is_empty());
            if editor.view.load(file_name).is_err() {
                editor.update_message(&format!("ERROR: 无法打开文件: {file_name}"));
            }
        }
        editor.refresh_status();
        Ok(editor)
    }

    /// 主事件循环，处理用户输入和界面刷新
    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => self.evaluate_event(event),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not read event: {err:?}");
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        // 错误提示
                        self.update_message("读取事件时发生错误，请重试。");
                    }
                }
            }
            self.refresh_status();
        }
    }

    /// 刷新整个屏幕，包括各 UI 组件
    fn refresh_screen(&mut self) {
        if self.terminal_size.height == 0 || self.terminal_size.width == 0 {
            return;
        }
        let bottom_bar_row = self.terminal_size.height.saturating_sub(1);
        let _ = Terminal::hide_caret();
        if self.in_prompt() {
            self.command_bar.render(bottom_bar_row);
        } else {
            self.message_bar.render(bottom_bar_row);
        }
        if self.terminal_size.height > 1 {
            self.status_bar
                .render(self.terminal_size.height.saturating_sub(2));
        }
        if self.terminal_size.height > 2 {
            self.view.render(0);
        }
        let new_caret_pos = if self.in_prompt() {
            Position {
                row: bottom_bar_row,
                col: self.command_bar.caret_position_col(),
            }
        } else {
            self.view.caret_position()
        };
        debug_assert!(new_caret_pos.col <= self.terminal_size.width);
        debug_assert!(new_caret_pos.row <= self.terminal_size.height);

        let _ = Terminal::move_caret_to(new_caret_pos);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }

    /// 刷新状态栏内容和终端标题
    fn refresh_status(&mut self) {
        let status = self.view.get_status();
        let title = format!("{} - {NAME}", status.file_name);
        self.status_bar.update_status(status);
        if title != self.title && matches!(Terminal::set_title(&title), Ok(())) {
            self.title = title;
        }
    }

    /// 处理输入事件，分发命令
    fn evaluate_event(&mut self, event: Event) {
        let should_process = match &event {
            Event::Key(KeyEvent { kind, .. }) => kind == &KeyEventKind::Press,
            Event::Resize(_, _) => true,
            _ => false,
        };

        if should_process {
            if let Ok(command) = Command::try_from(event) {
                self.process_command(command);
            }
        }
    }

    /// 处理命令分发，根据当前模式调用不同处理逻辑
    fn process_command(&mut self, command: Command) {
        match command {
            System(Resize(size)) => self.handle_resize_command(size),
            _ => match self.prompt_type {
                PromptType::Save => self.process_command_during_save(command),
                PromptType::None => self.process_command_no_prompt(command),
            }
        }
    }

    /// 非提示模式下的命令处理
    fn process_command_no_prompt(&mut self, command: Command) {
        if matches!(command, System(Quit)) {
            self.handle_quit_command();
            return;
        }
        self.reset_quit_times(); // 重置退出计数

        match command {
            System(Quit | Resize(_) | Dismiss) => {}, // 退出和调整大小已经在上面处理，其他不适用
            System(Save) => self.handle_save_command(),
            Edit(edit_command) => self.view.handle_edit_command(edit_command),
            Move(move_command) => self.view.handle_move_command(move_command),
        }
    }

    /// 处理调整终端大小命令
    fn handle_resize_command(&mut self, size: Size) {
        self.terminal_size = size;
        self.view.resize(Size {
            height: size.height.saturating_sub(2),
            width: size.width,
        });
        let bar_size = Size {
            height: 1,
            width: size.width,
        };
        self.message_bar.resize(bar_size);
        self.status_bar.resize(bar_size);
        self.command_bar.resize(bar_size);
    }

    /// 处理退出命令，支持多次确认
    fn handle_quit_command(&mut self) {
        if !self.view.get_status().is_modified || self.quit_times + 1 == QUIT_TIMES {
            self.should_quit = true;
        } else if self.view.get_status().is_modified {
            self.update_message(&format!(
                "WARNING! 文件有未保存的更改。再按 Ctrl-Q {} 次以退出。",
                QUIT_TIMES - self.quit_times - 1
            ));

            self.quit_times += 1;
        }
    }
    /// 重置退出计数
    fn reset_quit_times(&mut self) {
        if self.quit_times > 0 {
            self.quit_times = 0;
            self.update_message("");
        }
    }
    
    /// 处理保存命令
    fn handle_save_command(&mut self) {
        if self.view.is_file_loaded() {
            self.save(None);
        } else {
            self.set_prompt(PromptType::Save);
        }
    }

    /// 保存模式下的命令处理
    fn process_command_during_save(&mut self, command: Command) {
        match command {
            System(Quit | Resize(_) | Save) | Move(_) => {}, // 保存过程中不适用，调整大小已经在此阶段处理
            System(Dismiss) => {
                self.set_prompt(PromptType::None);
                self.update_message("保存已取消。");
            }
            Edit(InsertNewline) => {
                let file_name = self.command_bar.value();
                self.save(Some(&file_name));
                self.set_prompt(PromptType::None);
            }
            Edit(edit_command) => self.command_bar.handle_edit_command(edit_command),
        }
    }
    
    /// 保存文件，支持另存为
    fn save(&mut self, file_name: Option<&str>) {
        let result = if let Some(name) = file_name {
            self.view.save_as(name)
        } else {
            self.view.save()
        };
        if result.is_ok() {
            self.update_message("文件保存成功！");
        } else {
            self.update_message("文件写入失败！");
        }
    }

    /// 查找模式下的命令处理（已禁用，直接退出）
    fn process_command_during_search(&mut self, _command: Command) {
        // 纯文本编辑器不再支持查找，直接退出查找模式
        self.set_prompt(PromptType::None);
    }

    /// 更新消息栏内容
    fn update_message(&mut self, new_message: &str) {
        self.message_bar.update_message(new_message);
    }

    /// 判断当前是否为提示模式
    fn in_prompt(&self) -> bool {
        self.prompt_type.is_prompt()
    }

    /// 设置提示模式
    fn set_prompt(&mut self, prompt_type: PromptType) {
        match prompt_type {
            PromptType::None => self.message_bar.set_needs_redraw(true), // 确保消息栏在下一个重绘周期中正确绘制
            PromptType::Save => self.command_bar.set_prompt("保存为（Esc 取消）: "),
        }
        self.command_bar.clear_value();
        self.prompt_type = prompt_type;
    }
}

impl Drop for Editor {
    /// 退出时终端清理
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("欢迎下次使用。\r\n");
        }
    }
}
