use console::Style;

/// Centralized terminal palette. Violet/magenta brand accents; semantic greens/reds;
/// yellow reserved for warnings and paused state.
#[derive(Clone, Copy)]
pub struct Palette {
    pub enabled: bool,
}

impl Palette {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Brand/accent violet — 256-color on capable terminals, magenta fallback.
    fn primary_color() -> console::Color {
        if console::Term::stdout().features().colors_supported() {
            console::Color::Color256(141)
        } else {
            console::Color::Magenta
        }
    }

    fn style_primary(&self, bold: bool) -> Style {
        if self.enabled {
            let mut s = Style::new().fg(Self::primary_color());
            if bold {
                s = s.bold();
            }
            s
        } else {
            Style::new()
        }
    }

    fn style(&self, color: console::Color, bold: bool) -> Style {
        if self.enabled {
            let mut s = Style::new().fg(color);
            if bold {
                s = s.bold();
            }
            s
        } else {
            Style::new()
        }
    }

    pub fn brand(&self) -> Style {
        self.style_primary(true)
    }

    pub fn accent(&self) -> Style {
        self.style_primary(false)
    }

    pub fn active(&self) -> Style {
        self.style(console::Color::Green, true)
    }

    pub fn tripped(&self) -> Style {
        self.style(console::Color::Red, true)
    }

    pub fn paused(&self) -> Style {
        self.style(console::Color::Yellow, true)
    }

    pub fn warn(&self) -> Style {
        self.style(console::Color::Yellow, false)
    }

    pub fn dim(&self) -> Style {
        if self.enabled {
            Style::new().dim()
        } else {
            Style::new()
        }
    }

    pub fn ok(&self) -> Style {
        self.style(console::Color::Green, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_palette_emits_no_ansi_codes() {
        let palette = Palette::new(false);
        let styled = palette.brand().apply_to("Hawk.Sol").to_string();
        assert!(!styled.contains('\u{1b}'), "plain palette must not embed escape codes");
    }
}
