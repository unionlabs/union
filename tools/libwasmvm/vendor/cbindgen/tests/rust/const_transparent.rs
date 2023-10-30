#[repr(transparent)]
struct Transparent { field: u8 }

pub const FOO: Transparent = Transparent { field: 0 };
