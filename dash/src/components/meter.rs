// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use bytesize::ByteSize;
use iocraft::{
    component,
    components::{Text, TextAlign, TextWrap},
    element,
    hooks::UseRef,
    AnyElement, Color, Hooks, Props,
};

#[derive(Default, Props)]
pub(crate) struct MeterProps {
    pub value: u64,
    pub color: Option<Color>,
}

#[component]
pub(crate) fn Meter(props: &MeterProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
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
