use imgui_inspect_derive::Inspect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Inspect)]
pub enum PlainEnum {
    A,
    B,
    C,
}

fn main() {}
