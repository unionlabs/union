pub mod ast;
pub mod attr;

mod ctxt;
pub use self::ctxt::Ctxt;

mod receiver;
pub use self::receiver::replace_receiver;

mod case;
mod check;
mod respan;
mod symbol;

use syn::Type;

#[derive(Copy, Clone)]
pub enum Derive {
    Serialize,
    Deserialize,
}

pub fn ungroup(mut ty: &Type) -> &Type {
    while let Type::Group(group) = ty {
        ty = &group.elem;
    }
    ty
}
