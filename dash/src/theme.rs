use catppuccin::{ColorName, FlavorName};
use iocraft::Color;

pub(crate) struct Palette {
    // general background (whole window)
    pub background: Color,

    // general text */
    pub text: Color,

    // background for moving/focus elements (eg gutters)
    pub gutter: Color,

    // panel borders, frames
    pub border: Color,
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

        // https://github.com/catppuccin/catppuccin/blob/main/docs/style-guide.md
        Palette {
            background: color(f, ColorName::Base), // General -> Background Pane
            gutter: color(f, ColorName::Surface0), // General -> Surface Elements
            text: color(f, ColorName::Text),       // General -> Body Copy
            border: color(f, ColorName::Lavender), // Terminal -> Active Border
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
