// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use crate::{
    components::{Label, Meter, Panel, Sparkline},
    data, Theme,
};
use anystring::AnyString;
use bytesize::ByteSize;
use iocraft::{
    component,
    components::View,
    element,
    hooks::{UseContext, UseFuture, UseState},
    AnyElement, FlexDirection, Hooks, JustifyContent,
};

#[component]
pub(crate) fn PoolView(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();
    let name = hooks.use_context::<AnyString>().clone();

    let mut data = hooks.use_state(|| None);
    {
        let name = name.clone();
        hooks.use_future(async move {
            loop {
                data.set(data::get().pool_data.get(&name).map(|d| d.clone()));
                smol::Timer::after(std::time::Duration::from_secs(1)).await;
            }
        });
    }

    element! {
        Panel(title: &name) {
            #(match *data.read() {
                Some(ref pool_data) => element! {
                    View(
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Stretch,
                        gap: 1,
                    ) {
                        View(
                            //background_color: Color::Rgb { r: 128, g: 0, b: 0 },
                            flex_direction: FlexDirection::Column,
                        ) {
                            Label(content: "Read")
                            Label(content: "Write")
                        }
                        View(
                            //background_color: Color::Rgb { r: 0, g: 128, b: 0 },
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0_f32,
                            //overflow_x: Overflow::Hidden,
                        ) {
                            View(background_color: palette.gutter) {
                                Sparkline(value: pool_data.read, color: palette.text)
                            }
                            View(background_color: palette.gutter) {
                                Sparkline(value: pool_data.write, color: palette.text)
                            }
                        }
                        View(
                            //background_color: Color::Rgb { r: 0, g: 0, b: 128 },
                            flex_direction: FlexDirection::Column,
                            min_width: 9,
                        ) {
                            Meter(value: pool_data.read, color: palette.text)
                            Meter(value: pool_data.write, color: palette.text)
                        }
                        View(
                            //background_color: Color::Rgb { r: 128, g: 0, b: 128 },
                            flex_direction: FlexDirection::Column,
                        ) {
                            Label(content: pool_data.state.to_string())
                            Label(content: format!("Size: {}", ByteSize::b(pool_data.size)))
                            Label(content: format!("Used: {}", ByteSize::b(pool_data.alloc)))
                        }
                    }
                }.into_any(),
                None => element! {
                    Label(content: "loading")
                }.into_any(),
            })
            //Text(content: format!("{:#?}", &pool_data._wat), color: palette.secondary)
        }
    }
}
