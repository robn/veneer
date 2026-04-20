// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use crate::{
    components::{Label, Meter, Panel, Sparkline},
    data::PoolData,
    Theme,
};
use anystring::AnyString;
use bytesize::ByteSize;
use iocraft::{
    component, components::View, element, hooks::UseContext, AnyElement, FlexDirection, Hooks,
    JustifyContent, Props,
};

#[derive(Default, Props)]
pub(crate) struct PoolViewProps {
    pub name: AnyString,
}

#[component]
pub(crate) fn PoolView(props: &PoolViewProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();
    let pool_data = PoolData::default();

    element! {
        Panel(title: &props.name) {
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
                    flex_grow: 1.0,
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
            //Text(content: format!("{:#?}", &pool_data._wat), color: palette.secondary)
        }
    }
}
