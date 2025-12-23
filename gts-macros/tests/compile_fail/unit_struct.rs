//! Test: Unit structs are not supported

use gts_macros::struct_to_gts_schema;

#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.app.entities.empty.v1~",
    description = "Empty entity",
    properties = ""
)]
pub struct Empty;

fn main() {}
