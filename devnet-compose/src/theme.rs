use cliclack::{Theme, ThemeState};
use console::Style;

pub struct UnionTheme;

impl Theme for UnionTheme {
    fn bar_color(&self, state: &ThemeState) -> Style {
        let mut style = Style::new().cyan();
        if !matches!(state, ThemeState::Active) {
            style = style.dim();
        }
        if let ThemeState::Error(_) = state {
            style = Style::new().red();
        }
        style
    }

    fn state_symbol_color(&self, _state: &ThemeState) -> Style {
        static CYAN_STYLE: Style = Style::new().cyan();
        CYAN_STYLE
    }

    fn info_symbol(&self) -> String {
        const INFO_SYMBOL: &str = "⚙";
        INFO_SYMBOL.to_string()
    }
}
