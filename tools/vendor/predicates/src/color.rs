#[derive(Copy, Clone, Debug, Default)]
pub(crate) struct Palette {
    pub(crate) description: styled::Style,
    pub(crate) var: styled::Style,
    pub(crate) expected: styled::Style,
}

impl Palette {
    #[cfg(feature = "color")]
    pub(crate) fn current() -> Self {
        if concolor::get(concolor::Stream::Either).ansi_color() {
            Self {
                description: styled::Style(yansi::Style::new(yansi::Color::Blue).bold()),
                var: styled::Style(yansi::Style::new(yansi::Color::Red).bold()),
                expected: styled::Style(yansi::Style::new(yansi::Color::Green).bold()),
            }
        } else {
            Self::default()
        }
    }

    #[cfg(not(feature = "color"))]
    pub(crate) fn current() -> Self {
        Self::default()
    }
}

#[cfg(feature = "color")]
mod styled {
    #[derive(Copy, Clone, Debug, Default)]
    pub(crate) struct Style(pub(crate) yansi::Style);

    impl Style {
        pub(crate) fn paint<T: std::fmt::Display>(self, item: T) -> impl std::fmt::Display {
            self.0.paint(item)
        }
    }
}

#[cfg(not(feature = "color"))]
mod styled {
    #[derive(Copy, Clone, Debug, Default)]
    pub(crate) struct Style;

    impl Style {
        pub(crate) fn paint<T: std::fmt::Display>(self, item: T) -> impl std::fmt::Display {
            item
        }
    }
}
