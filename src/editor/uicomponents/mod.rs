// 统一管理编辑器的各个 UI 组件，便于主程序调用。

mod view;
/// 编辑区主视图组件
pub use view::View;

mod commandbar;
/// 命令栏组件（显示快捷键信息）
pub use commandbar::CommandBar;

mod messagebar;
/// 消息栏组件（用于显示临时提示信息）
pub use messagebar::MessageBar;

mod statusbar;
/// 状态栏组件（显示文件名、行数、光标位置等）
pub use statusbar::StatusBar;

mod uicomponent;
/// UI 组件通用 trait
pub use uicomponent::UIComponent;
