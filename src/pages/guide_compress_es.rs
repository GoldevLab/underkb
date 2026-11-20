use resuma::prelude::*;

pub fn page(_req: FlowRequest) -> View {
    crate::guides::render("/guides/comprimir-imagen-a-200kb")
}
