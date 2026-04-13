use bytesize::ByteSize;
use iocraft::prelude::*;
use std::collections::BTreeMap;
use veneer::{Error, VdevState};

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

const BLOCKS: &'static [char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

#[derive(Default, Props)]
struct SparklineProps {
    value: u64,
}

struct Sparkline {
    prev: Option<u64>,
    history: Vec<u64>,
}

impl Default for Sparkline {
    fn default() -> Self {
        Sparkline {
            prev: None,
            history: std::iter::repeat(0).take(256).collect(),
            //..Default::default()
        }
    }
}

impl Component for Sparkline {
    type Props<'a> = SparklineProps;

    fn new(_props: &Self::Props<'_>) -> Self {
        Self::default()
    }

    fn update(
        &mut self,
        props: &mut Self::Props<'_>,
        _hooks: Hooks,
        updater: &mut ComponentUpdater,
    ) {
        let diff = self.prev.map(|v| props.value - v).unwrap_or(0);

        self.prev.replace(props.value);

        self.history.remove(0);
        self.history.push(diff);

        updater.set_measure_func(Box::new(move |known_size, available_space, _style| {
            // return is a taffy::geometry::Size<f32>, which we don't have named access to.
            // `known_size` however is a taffy::geometry::Size<Option<f32>> and has deriving
            // constructors, so we can work around it with this nonsense
            let w = if available_space.width.is_definite() {
                available_space.width.unwrap().min(256.0)
            } else if available_space.width.compute_free_space(1.0) < 1.0 {
                // MinContent
                8.0 as _
            } else {
                // MaxContent
                256.0 as _
            };
            known_size
                .map_width(|_| Some(w))
                .map_height(|_| Some(1 as _))
                .map(|v| v.unwrap())
        }));
    }

    fn draw(&mut self, drawer: &mut ComponentDrawer<'_>) {
        let width = drawer.layout().size.width as usize;

        let (_, w) = self.history.split_at(self.history.len() - width);

        let sf = (*(w.iter().max().unwrap_or(&0)) as f64) / ((BLOCKS.len() - 1) as f64);

        let content: String = w
            .iter()
            .map(|&v| ((v as f64) / sf) as usize)
            .map(|v| {
                assert!(v < 9);
                BLOCKS[v]
            })
            .collect();

        drawer
            .canvas()
            .set_text(0, 0, &content, CanvasTextStyle::default());
    }
}

#[derive(Default, Props)]
struct DashMeterProps {
    value: u64,
    color: Option<Color>,
}

#[component]
fn DashMeter(props: &DashMeterProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut prev = hooks.use_ref(|| props.value);
    let diff = props.value - prev.get();
    prev.set(props.value);

    element! {
        Text(
            content: format!("{}", ByteSize::b(diff)),
            color: props.color,
            align: TextAlign::Center,
            wrap: TextWrap::NoWrap
        )
    }
}

#[derive(Default, Props)]
struct DashBoxProps<'a> {
    children: Vec<AnyElement<'a>>,
    title: String,
}

#[component]
fn DashBox(props: &mut DashBoxProps<'static>, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();

    element! {
        View(
            background_color: palette.bg,
            border_style: BorderStyle::Single,
            border_color: palette.accent,
            flex_direction: FlexDirection::Column,
            padding_left: 1,
            padding_right: 1,
        ) {
            View(margin_top: -1) {
                Text(
                    content: format!(" {} ", &props.title),
                    color: palette.accent,
                    wrap: TextWrap::NoWrap,
                )
            }
            #(std::mem::take(&mut props.children))
        }
    }
}

#[derive(Clone, Debug, Default)]
struct PoolData {
    name: String,
    state: VdevState,
    size: u64,
    alloc: u64,
    read: u64,
    write: u64,
    _wat: String,
}

#[derive(Clone, Debug, Default)]
struct DashData {
    pools: BTreeMap<String, PoolData>,
}

impl DashData {
    async fn fetch() -> Result<Self, Error> {
        let z = veneer::open()?;

        let mut pools = BTreeMap::default();
        for pool in z.pools()? {
            let root = pool.root_vdev()?;
            let stats = root.stats()?;

            let data = PoolData {
                name: pool.name(),
                state: stats.state,
                size: stats.space,
                alloc: stats.alloc,
                read: stats.bytes[1],
                write: stats.bytes[2],
                _wat: format!("{:?}", stats),
            };

            pools.insert(pool.name(), data);
        }

        Ok(DashData { pools })
    }
}

#[derive(Default, Props)]
struct PoolDataViewProps {
    data: PoolData,
}

#[component]
fn PoolDataView(props: &PoolDataViewProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();

    element! {
        DashBox(title: &props.data.name) {
            View(
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Stretch,
                gap: 1,
            ) {
                View(
                    //background_color: Color::Rgb { r: 128, g: 0, b: 0 },
                    flex_direction: FlexDirection::Column,
                ) {
                    Text(content: "Read", wrap: TextWrap::NoWrap)
                    Text(content: "Write", wrap: TextWrap::NoWrap)
                }
                View(
                    //background_color: Color::Rgb { r: 0, g: 128, b: 0 },
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    //overflow_x: Overflow::Hidden,
                ) {
                    View(background_color: palette.selection) {
                        Sparkline(value: props.data.read)
                    }
                    View(background_color: palette.selection) {
                        Sparkline(value: props.data.write)
                    }
                }
                View(
                    //background_color: Color::Rgb { r: 0, g: 0, b: 128 },
                    flex_direction: FlexDirection::Column,
                    min_width: 9,
                ) {
                    DashMeter(value: props.data.read, color: palette.fg)
                    DashMeter(value: props.data.write, color: palette.fg)
                }
                View(
                    //background_color: Color::Rgb { r: 128, g: 0, b: 128 },
                    flex_direction: FlexDirection::Column,
                ) {
                    Text(
                        content: props.data.state.to_string(),
                        color: palette.fg,
                        wrap: TextWrap::NoWrap)
                    Text(
                        content: format!("Size: {}", ByteSize::b(props.data.size)),
                        color: palette.fg,
                        wrap: TextWrap::NoWrap,
                    )
                    Text(
                        content: format!("Used: {}", ByteSize::b(props.data.alloc)),
                        color: palette.fg,
                        wrap: TextWrap::NoWrap,
                    )
                }
            }
            //Text(content: format!("{:#?}", &props.data._wat), color: palette.secondary)
        }
    }
}

enum DashState {
    Init,
    Loading,
    Loaded(Result<DashData, Error>),
}

#[component]
fn Dash(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut theme = hooks.use_state(|| Theme::default());

    let mut state = hooks.use_state(|| DashState::Init);
    hooks.use_future(async move {
        loop {
            if !matches!(*state.read(), DashState::Loading) {
                state.set(DashState::Loading);
                state.set(DashState::Loaded(DashData::fetch().await));
            }
            smol::Timer::after(std::time::Duration::from_secs(1)).await;
        }
    });

    let mut exit = hooks.use_state(|| false);
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
            height,
            background_color: theme.get().palette().bg,
            flex_direction: FlexDirection::Column,
        ) {
            #(match &*state.read() {
                DashState::Loaded(Ok(data)) => element! {
                    Fragment {
                        #(data.pools.iter().map(|(name,data)| {
                            element! {
                                ContextProvider(value: Context::owned(theme.get())) {
                                    PoolDataView(key: name.clone(), data: data.clone())
                                }
                            }
                        }))
                    }
                }.into_any(),
                DashState::Loaded(Err(err)) => element! {
                    View(
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        width: 100pct,
                        height: 100pct,
                        padding: 2,
                    ) {
                        Text(content: "Error!", weight: Weight::Bold, color: Color::Red)
                        Text(content: format!("{:#}", err))
                    }
                }.into_any(),
                _ => element! {
                    Text(content: "loading")
                }.into_any(),
            })
        }
    }
}

fn main() {
    smol::block_on(element!(Dash).fullscreen()).unwrap();
}
