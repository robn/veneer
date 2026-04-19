// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use iocraft::taffy::{AvailableSpace, Size};
use iocraft::{CanvasTextStyle, Color, Component, ComponentDrawer, ComponentUpdater, Hooks, Props};

const BLOCKS: &'static [char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

#[derive(Default, Props)]
pub(crate) struct SparklineProps {
    pub value: u64,
    pub color: Option<Color>,
}

pub(crate) struct Sparkline {
    prev: Option<u64>,
    history: Vec<u64>,
    style: CanvasTextStyle,
}

impl Sparkline {
    const MIN_WIDTH: usize = 8;
    const MAX_WIDTH: usize = 256;
}

impl Default for Sparkline {
    fn default() -> Self {
        Sparkline {
            prev: None,
            history: std::iter::repeat(0).take(Self::MAX_WIDTH).collect(),
            style: CanvasTextStyle::default(),
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

        self.style.color = props.color;

        updater.set_measure_func(Box::new(move |_, avail, _| {
            let w = match avail.width {
                AvailableSpace::Definite(s) => s.min(Self::MAX_WIDTH as _),
                AvailableSpace::MinContent => Self::MIN_WIDTH as _,
                AvailableSpace::MaxContent => Self::MAX_WIDTH as _,
            };

            Size {
                width: w,
                height: 1.0,
            }
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

        drawer.canvas().set_text(0, 0, &content, self.style);
    }
}
