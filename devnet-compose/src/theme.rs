use cliclack::{Theme, ThemeState};
use console::Style;

pub struct UnionTheme;

impl Theme for UnionTheme {
    fn bar_color(&self, state: &ThemeState) -> Style {
        match state {
            ThemeState::Active => Style::new().cyan(),
            ThemeState::Error(_) => Style::new().red(),
            _ => Style::new().cyan().dim(),
        }
    }

    fn state_symbol_color(&self, _state: &ThemeState) -> Style {
        Style::new().cyan()
    }

    fn info_symbol(&self) -> String {
        "⚙".into()
    }
}
