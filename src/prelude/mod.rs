// prelude 统一导出常用类型和常量，便于全局使用

pub type GraphemeIdx = usize;
pub type LineIdx = usize;
pub type ByteIdx = usize;
pub type ColIdx = usize;
pub type RowIdx = usize;

mod location;
pub use location::Location;

mod position;
pub use position::Position;

mod size;
pub use size::Size;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
