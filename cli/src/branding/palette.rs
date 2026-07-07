use console::Style;

/// Centralized terminal palette. Amber/gold accents; semantic greens/reds/yellows.
#[derive(Clone, Copy)]
pub struct Palette {
    pub enabled: bool,
}

impl Palette {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
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
        self.style(console::Color::Yellow, true)
    }

    pub fn accent(&self) -> Style {
        self.style(console::Color::Yellow, false)
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
