use crate::{
    components::{Label, Meter, Panel, Sparkline},
    PoolData, Theme,
};
use bytesize::ByteSize;
use iocraft::{
    component, components::View, element, hooks::UseContext, AnyElement, FlexDirection, Hooks,
    JustifyContent, Props,
};

#[derive(Default, Props)]
pub(crate) struct PoolViewProps {
    pub data: PoolData,
}

#[component]
pub(crate) fn PoolView(props: &PoolViewProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();

    element! {
        Panel(title: &props.data.name) {
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
                        Sparkline(value: props.data.read, color: palette.text)
                    }
                    View(background_color: palette.gutter) {
                        Sparkline(value: props.data.write, color: palette.text)
                    }
                }
                View(
                    //background_color: Color::Rgb { r: 0, g: 0, b: 128 },
                    flex_direction: FlexDirection::Column,
                    min_width: 9,
                ) {
                    Meter(value: props.data.read, color: palette.text)
                    Meter(value: props.data.write, color: palette.text)
                }
                View(
                    //background_color: Color::Rgb { r: 128, g: 0, b: 128 },
                    flex_direction: FlexDirection::Column,
                ) {
                    Label(content: props.data.state.to_string())
                    Label(content: format!("Size: {}", ByteSize::b(props.data.size)))
                    Label(content: format!("Used: {}", ByteSize::b(props.data.alloc)))
                }
            }
            //Text(content: format!("{:#?}", &props.data._wat), color: palette.secondary)
        }
    }
}
