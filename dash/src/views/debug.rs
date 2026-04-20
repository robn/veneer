use crate::components::Panel;
use iocraft::{component, element, AnyElement};

#[component]
pub(crate) fn DebugView() -> impl Into<AnyElement<'static>> {
    element! {
        Panel(title: "debug") {
        }
    }
}
