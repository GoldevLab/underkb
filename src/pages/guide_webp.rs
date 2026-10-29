use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    crate::guides::render("/guides/jpg-to-webp")
}
