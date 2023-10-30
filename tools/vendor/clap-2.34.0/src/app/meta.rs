#[doc(hidden)]
#[allow(missing_debug_implementations)]
#[derive(Default, Clone)]
pub struct AppMeta<'b> {
    pub name: String,
    pub bin_name: Option<String>,
    pub author: Option<&'b str>,
    pub version: Option<&'b str>,
    pub long_version: Option<&'b str>,
    pub about: Option<&'b str>,
    pub long_about: Option<&'b str>,
    pub more_help: Option<&'b str>,
    pub pre_help: Option<&'b str>,
    pub aliases: Option<Vec<(&'b str, bool)>>, // (name, visible)
    pub usage_str: Option<&'b str>,
    pub usage: Option<String>,
    pub help_str: Option<&'b str>,
    pub disp_ord: usize,
    pub term_w: Option<usize>,
    pub max_w: Option<usize>,
    pub template: Option<&'b str>,
}

impl<'b> AppMeta<'b> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn with_name(s: String) -> Self {
        AppMeta {
            name: s,
            disp_ord: 999,
            ..Default::default()
        }
    }
}
