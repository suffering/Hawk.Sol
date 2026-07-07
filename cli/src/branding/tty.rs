use super::palette::Palette;

pub fn is_tty_stdout() -> bool {
    console::Term::stdout().is_term()
}

pub fn color_enabled(quiet: bool) -> bool {
    is_tty_stdout() && !quiet
}

pub fn terminal_palette(quiet: bool) -> Palette {
    Palette::new(color_enabled(quiet))
}
