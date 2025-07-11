// GraphemeWidth 用于区分半宽和全宽字符的显示宽度。


#[derive(Copy, Clone, Debug)]
/// 字素宽度：Half 表示半宽，Full 表示全宽
pub enum GraphemeWidth {
    Half, 
    Full, 
}
// 将 GraphemeWidth 转换为 usize 类型
impl From<GraphemeWidth> for usize {
    fn from(val: GraphemeWidth) -> Self {
        match val {
            GraphemeWidth::Half => 1,
            GraphemeWidth::Full => 2,
        }
    }
}