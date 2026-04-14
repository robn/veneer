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
