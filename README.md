# Rust-text-editor

`Rust-text-editor` 是一个以 Rust 编写的终端纯文本编辑器，专注于安全、清晰的模块化设计。它通过 `crossterm` 提供跨平台终端控制，强调零 `unsafe`、严格边界检查与可扩展的组件划分，让学习者与进阶用户都能快速理解并二次开发。

## ✨ 核心特性

- **完整的文本编辑体验**：支持字符插入、删除、换行与多步游标移动（方向键、行首行尾、分页）。
- **可靠的文件操作**：打开、保存、另存为与脏标记追踪，退出前贴心提示。
- **可组合的命令系统**：`EditCommand` / `MoveCommand` / `SystemCommand` 三分法，逻辑清晰易扩展。
- **富信息 UI**：状态栏、命令栏、消息栏与主视图协同提供实时反馈。
- **终端控制抽象**：封装初始化、清屏、游标控制、备用屏幕、窗口尺寸等操作。
- **多字节字符友好**：`graphemewidth` 模块精准处理 Unicode 字素宽度，确保中英文混排展示准确。

## 🧱 项目结构速览

```
src
├── main.rs
├── prelude/           # RowIdx、ColIdx、Location、Size 等常用类型
└── editor/
    ├── command/       # edit.rs、movecommand.rs、system.rs
    ├── line/          # Line、TextFragment、graphemewidth
    ├── terminal/      # 终端操作封装
    ├── uicomponents/
    │   ├── statusbar.rs、commandbar.rs、messagebar.rs
    │   └── view/      # buffer.rs、fileinfo.rs、mod.rs
    └── documentstatus.rs
```

高度模块化使每个组件都保持单一职责、低耦合：在任何子模块内的修复或特性迭代都不会波及整体架构，也让后续扩展（例如语法高亮、插件化）更从容。

## 🚀 快速开始

```bash
# 1. 获取代码
git clone https://github.com/ahzlll/Rust-text-editor.git
cd Rust-text-editor

# 2. 下载依赖并运行（新建空白文档）
cargo run

# 3. 打开示例文件 `test.txt`
cargo run -- test.txt

# 4. 发布模式（可选，性能更好）
cargo run --release -- path/to/file
```

运行后即可进入全屏终端编辑器。若需要退出，请使用内建命令（如 `:q` 或 `Ctrl-C`，取决于你在命令模块中的绑定）。

## 🧭 使用方式速查

| 操作 | 键位 / 命令 | 说明 |
| --- | --- | --- |
| 保存当前文件 | `Ctrl + S` | 若文件尚未命名，会弹出命令栏输入文件名，回车确认，`Esc` 取消 |
| 退出编辑器 | `Ctrl + Q` | 脏缓冲区会触发 3 次确认：信息栏提示剩余次数 |
| 取消当前提示 | `Esc` | 适用于保存提示等 |
| 插入字符/换行 | 直接输入 / `Enter` | 所有可打印字符与 Tab 均可插入 |
| 删除 | `Backspace` / `Delete` | 前向删除或后向删除一个字符 |
| 光标移动 | 方向键 / `Home` / `End` / `PageUp` / `PageDown` | 支持行首、行尾及整页跳转 |

提示栏在启动时会显示快捷键摘要（`Ctrl + S = 保存 | Ctrl + Q = 退出`），方便新用户记忆。

> 仓库根目录下的 `test.txt` 是一个用于展示混合中英文字宽与 UI 反馈的示例文档，运行 `cargo run -- test.txt` 即可直接体验。

## 🕹️ 交互指南（示例）

- 插入/删除：直接输入字符；使用删除键或命令触发 `DeleteChar`
- 移动：方向键、`Home`/`End`、`PageUp`/`PageDown`
- 文件：`:o <path>` 打开、`:w` 保存、`:w <path>` 另存、`:q` 退出
- 状态栏实时显示：文件名、行列位置、总行数、脏状态

> 具体按键与命令绑定可在 `src/editor/command` 与 `uicomponents` 目录内自定义。

## 🦀 Rust 安全性的全面实践

- **所有权与借用**：`Buffer`、`FileInfo`、`Line` 等结构体严格遵守可变/不可变借用规则，例如：

```startLine:endLine:src/editor/uicomponents/view/buffer.rs
pub fn insert_char(&mut self, character: char, at: Location) {
    debug_assert!(at.line_idx <= self.height());
    if at.line_idx == self.height() {
        self.lines.push(Line::from(&character.to_string()));
    } else if let Some(line) = self.lines.get_mut(at.line_idx) {
        line.insert_char(character, at.grapheme_idx);
    }
    self.dirty = true;
}
```

- **零 `unsafe`**：全项目均未使用 `unsafe`，借助 Rust 编译期保证杜绝悬垂指针、缓冲区溢出等风险。
- **类型别名与边界检查**：`RowIdx`、`ColIdx`、`Location` 等类型配合 `saturating_add/sub` 确保索引安全。
- **Result-based 错误处理**：文件 I/O 等操作使用 `Result<T, Error>` 与 `?` 运算符，保证资源自动释放。

## 🔁 与其他编辑器的对比

| 项目        | 特性侧重                                   | 适用人群             |
|-------------|--------------------------------------------|----------------------|
| text-editor-rust | 轻量、教学友好、零 unsafe、结构极清晰           | 学习者、系统编程初学者 |
| xi-editor   | 多线程、前后端分离、插件化                  | 高性能需求开发者     |
| helix       | 现代 UI、LSP、语法高亮                     | 高阶终端爱好者       |
| kakoune     | C++、多光标、交互式                        | Vim 流派探索者       |

本项目启动快、占用低，却保留了 Rust 在内存/线程安全上的全部优势，是练习系统级软件工程的理想素材。

## 🗺️ 后续规划

- 语法高亮与主题系统
- LSP/LSP-like 提示与自动补全
- 文件树/多缓冲区支持
- 插件接口与命令脚本化

欢迎提交 Issue/PR，一起将 `Rust-text-editor` 打磨成教学与实践并重的优秀示例。


