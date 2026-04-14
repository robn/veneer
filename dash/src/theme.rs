use catppuccin::{ColorName, FlavorName};
use iocraft::Color;

pub(crate) struct Palette {
    pub accent: Color,
    pub bg: Color,
    pub fg: Color,
    pub selection: Color,
}

#[derive(Clone, Copy, Default)]
pub(crate) enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    const fn palette_for(theme: Theme) -> Palette {
        const fn color(f: FlavorName, c: ColorName) -> Color {
            let rgb = catppuccin::PALETTE.get_flavor(f).get_color(c).rgb;
            Color::Rgb {
                r: rgb.r,
                g: rgb.g,
                b: rgb.b,
            }
        }

        let f = match theme {
            Theme::Dark => FlavorName::Mocha,
            Theme::Light => FlavorName::Latte,
        };

        Palette {
            accent: color(f, ColorName::Blue),
            bg: color(f, ColorName::Base),
            fg: color(f, ColorName::Text),
            selection: color(f, ColorName::Surface0),
        }
    }

    pub fn palette(&self) -> Palette {
        Self::palette_for(*self)
    }

    pub fn toggle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}
