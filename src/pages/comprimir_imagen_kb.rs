use resuma::prelude::*;

use crate::landing::{self, Tool};

pub fn page(_req: FlowRequest) -> View {
    landing::page(Tool::Compress)
}
