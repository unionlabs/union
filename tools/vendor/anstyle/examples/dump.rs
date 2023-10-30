use std::io::Write;

fn main() -> Result<(), lexopt::Error> {
    let args = Args::parse()?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    for fixed in 0..16 {
        let style = style(fixed, args.layer, args.effects);
        let _ = print_number(&mut stdout, fixed, style);
        if fixed == 7 || fixed == 15 {
            let _ = writeln!(&mut stdout);
        }
    }

    for r in 0..6 {
        let _ = writeln!(stdout);
        for g in 0..6 {
            for b in 0..6 {
                let fixed = r * 36 + g * 6 + b + 16;
                let style = style(fixed, args.layer, args.effects);
                let _ = print_number(&mut stdout, fixed, style);
            }
            let _ = writeln!(stdout);
        }
    }

    for c in 0..24 {
        if 0 == c % 8 {
            let _ = writeln!(stdout);
        }
        let fixed = 232 + c;
        let style = style(fixed, args.layer, args.effects);
        let _ = print_number(&mut stdout, fixed, style);
    }

    Ok(())
}

fn style(fixed: u8, layer: Layer, effects: anstyle::Effects) -> anstyle::Style {
    let color = anstyle::Ansi256Color(fixed).into();
    (match layer {
        Layer::Fg => anstyle::Style::new().fg_color(Some(color)),
        Layer::Bg => anstyle::Style::new().bg_color(Some(color)),
        Layer::Underline => anstyle::Style::new().underline_color(Some(color)),
    }) | effects
}

fn print_number(
    stdout: &mut std::io::StdoutLock<'_>,
    fixed: u8,
    style: anstyle::Style,
) -> std::io::Result<()> {
    write!(
        stdout,
        "{}{:>4}{}",
        style.render(),
        fixed,
        anstyle::Reset.render()
    )
}

#[derive(Default)]
struct Args {
    effects: anstyle::Effects,
    layer: Layer,
}

#[derive(Copy, Clone)]
enum Layer {
    Fg,
    Bg,
    Underline,
}

impl Default for Layer {
    fn default() -> Self {
        Layer::Fg
    }
}

impl Args {
    fn parse() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;

        let mut res = Args::default();

        let mut args = lexopt::Parser::from_env();
        while let Some(arg) = args.next()? {
            match arg {
                Long("layer") => {
                    res.layer = args.value()?.parse_with(|s| match s {
                        "fg" => Ok(Layer::Fg),
                        "bg" => Ok(Layer::Bg),
                        "underline" => Ok(Layer::Underline),
                        _ => Err("expected values fg, bg, underline"),
                    })?;
                }
                Long("effect") => {
                    const EFFECTS: [(&str, anstyle::Effects); 12] = [
                        ("bold", anstyle::Effects::BOLD),
                        ("dimmed", anstyle::Effects::DIMMED),
                        ("italic", anstyle::Effects::ITALIC),
                        ("underline", anstyle::Effects::UNDERLINE),
                        ("double_underline", anstyle::Effects::DOUBLE_UNDERLINE),
                        ("curly_underline", anstyle::Effects::CURLY_UNDERLINE),
                        ("dotted_underline", anstyle::Effects::DOTTED_UNDERLINE),
                        ("dashed_underline", anstyle::Effects::DASHED_UNDERLINE),
                        ("blink", anstyle::Effects::BLINK),
                        ("invert", anstyle::Effects::INVERT),
                        ("hidden", anstyle::Effects::HIDDEN),
                        ("strikethrough", anstyle::Effects::STRIKETHROUGH),
                    ];
                    let effect = args.value()?.parse_with(|s| {
                        EFFECTS
                            .into_iter()
                            .find(|(name, _)| *name == s)
                            .map(|(_, effect)| effect)
                            .ok_or_else(|| {
                                format!(
                                    "expected one of {}",
                                    EFFECTS
                                        .into_iter()
                                        .map(|(n, _)| n)
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            })
                    })?;
                    res.effects = res.effects.insert(effect);
                }
                _ => return Err(arg.unexpected()),
            }
        }
        Ok(res)
    }
}
