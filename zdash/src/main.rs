use iocraft::prelude::*;

// Minimal theme stuff ripped form ratatui_themes, because getting the traits
// right for converting Color from ratatui -> iocraft is more
// complicated that I care for right now (and a bit dumb, because they're
// all crossterm::style::Color in the end anyway

struct Palette {
    accent: Color,
    secondary: Color,
    bg: Color,
    fg: Color,
    muted: Color,
    selection: Color,
    error: Color,
    warning: Color,
    success: Color,
    info: Color,
}

#[derive(Clone, Copy, Default)]
enum Theme {
    #[default]
    Dark,
    Light,
}

impl Theme {
    const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color::Rgb { r, g, b }
    }

    const fn dark() -> Palette {
        // Catppuccin Mocha
        Palette {
            accent: Self::rgb(137, 180, 250),    // Blue
            secondary: Self::rgb(245, 194, 231), // Pink
            bg: Self::rgb(30, 30, 46),           // Base
            fg: Self::rgb(205, 214, 244),        // Text
            muted: Self::rgb(108, 112, 134),     // Overlay0
            selection: Self::rgb(49, 50, 68),    // Surface0
            error: Self::rgb(243, 139, 168),     // Red
            warning: Self::rgb(249, 226, 175),   // Yellow
            success: Self::rgb(166, 227, 161),   // Green
            info: Self::rgb(148, 226, 213),      // Teal
        }
    }

    const fn light() -> Palette {
        // Catppuccin Latte
        Palette {
            accent: Self::rgb(30, 102, 245),     // Blue
            secondary: Self::rgb(234, 118, 203), // Pink
            bg: Self::rgb(239, 241, 245),        // Base
            fg: Self::rgb(76, 79, 105),          // Text
            muted: Self::rgb(140, 143, 161),     // Overlay0
            selection: Self::rgb(204, 208, 218), // Surface0
            error: Self::rgb(210, 15, 57),       // Red
            warning: Self::rgb(223, 142, 29),    // Yellow
            success: Self::rgb(64, 160, 43),     // Green
            info: Self::rgb(23, 146, 153),       // Teal
        }
    }

    pub fn palette(&self) -> Palette {
        match self {
            Theme::Dark => Self::dark(),
            Theme::Light => Self::light(),
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}

#[derive(Default, Props)]
struct DashBoxProps {
    title: String,
}

#[component]
fn DashBox(props: &DashBoxProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();

    element! {
        View(
            background_color: palette.bg,
            border_style: BorderStyle::DoubleLeftRight,
            border_color: palette.accent,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
        ) {
            View(margin_top: -1) {
                Text(
                    content: format!(" {} ", &props.title),
                    color: palette.accent,
                    wrap: TextWrap::NoWrap,
                )
            }
            View(
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                height: 3,
                flex_grow: 1.0,
            ) {
                Text(
                    content: "oh no",
                    color: palette.fg,
                    weight: Weight::Bold,
                )
            }
        }
    }
}

#[component]
fn Dash(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let mut theme = hooks.use_state(|| Theme::default());

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('q') => exit.set(true),
                    KeyCode::Char('t') => theme.set(theme.get().toggle()),
                    _ => {}
                }
            }
            _ => {}
        }
    });

    if exit.get() {
        system.exit();
    }

    element! {
        View(
            width,
            height: height,
            background_color: theme.get().palette().bg,
            flex_direction: FlexDirection::Column,
            gap: 1,
        ) {
            View(
                flex_grow: 1.0,
            ) {
                View(
                    flex_grow: 1.0,
                ) {
                    ContextProvider(value: Context::owned(theme.get())) {
                        DashBox(title: "pool")
                    }
                }
            }
        }
    }
}

fn main() {
    smol::block_on(element!(Dash).fullscreen()).unwrap();
}
