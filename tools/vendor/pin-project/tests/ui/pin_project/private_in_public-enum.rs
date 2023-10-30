// Even if allows private_in_public, these are errors.

#![allow(private_in_public)]

pub enum PublicEnum {
    V(PrivateEnum), //~ ERROR E0446
}

enum PrivateEnum {
    V(u8),
}

mod foo {
    pub(crate) enum CrateEnum {
        V(PrivateEnum), //~ ERROR E0446
    }

    enum PrivateEnum {
        V(u8),
    }
}

fn main() {}
