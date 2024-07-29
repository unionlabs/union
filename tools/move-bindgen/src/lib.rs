pub use aptos_rest_client;
pub use aptos_types;
pub use bcs;
pub use move_bindgen_derive::TypeTagged;
pub use move_core_types;
use move_core_types::{
    account_address::AccountAddress,
    ident_str,
    language_storage::{StructTag, TypeTag},
};
pub use serde;
use serde::{Deserialize, Serialize};
pub use serde_json;
pub use tracing;

pub trait TypeTagged {
    type Ctx;

    fn type_tag(ctx: Self::Ctx) -> TypeTag;
}

impl TypeTagged for bool {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::Bool
    }
}

impl TypeTagged for u8 {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::U8
    }
}

impl TypeTagged for u16 {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::U16
    }
}

impl TypeTagged for u32 {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::U32
    }
}

impl TypeTagged for u64 {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::U64
    }
}

impl TypeTagged for aptos_rest_client::aptos_api_types::U64 {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::U64
    }
}

impl TypeTagged for u128 {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::U128
    }
}

impl TypeTagged for aptos_rest_client::aptos_api_types::U128 {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::U128
    }
}

// TODO: impl this in unionlabs
// impl TypeTagged for U256 {
//     fn type_tag(_ctx: Self::Ctx) -> TypeTag {
//         TypeTag::U128
//     }
// }

impl TypeTagged for String {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::Struct(Box::new(StructTag {
            address: AccountAddress::ONE,
            module: ident_str!("string").into(),
            name: ident_str!("String").into(),
            type_args: vec![],
        }))
    }
}

impl<T: TypeTagged> TypeTagged for Vec<T> {
    type Ctx = T::Ctx;

    fn type_tag(ctx: Self::Ctx) -> TypeTag {
        TypeTag::Vector(Box::new(T::type_tag(ctx)))
    }
}

impl TypeTagged for aptos_rest_client::aptos_api_types::HexEncodedBytes {
    type Ctx = ();

    fn type_tag(_ctx: Self::Ctx) -> TypeTag {
        TypeTag::Vector(Box::new(TypeTag::U8))
    }
}

impl<T: TypeTagged> TypeTagged for Option<T> {
    type Ctx = T::Ctx;

    fn type_tag(ctx: Self::Ctx) -> TypeTag {
        TypeTag::Struct(Box::new(StructTag {
            address: AccountAddress::ONE,
            module: ident_str!("option").into(),
            name: ident_str!("Option").into(),
            type_args: vec![T::type_tag(ctx)],
        }))
    }
}

pub trait IntoTypeTagged<T: TypeTagged> {
    fn into_type_tagged(self) -> (T, T::Ctx);
}

impl<T: TypeTagged<Ctx = ()>> IntoTypeTagged<T> for T {
    fn into_type_tagged(self) -> (T, <T as TypeTagged>::Ctx) {
        (self, ())
    }
}

impl IntoTypeTagged<Vec<u8>> for aptos_rest_client::aptos_api_types::HexEncodedBytes {
    fn into_type_tagged(self) -> (Vec<u8>, ()) {
        (self.into(), ())
    }
}

impl<T: TypeTagged> IntoTypeTagged<T> for (T, T::Ctx) {
    fn into_type_tagged(self) -> (T, T::Ctx) {
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveOption<T> {
    // TODO: Add verification here that this only contains one item, maybe through a custom `SingleItemVec`?
    pub vec: Vec<T>,
}

impl<T> MoveOption<T> {
    pub fn into_option(mut self) -> Option<T> {
        self.vec.pop()
    }
}
