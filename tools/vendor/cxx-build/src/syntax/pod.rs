use crate::syntax::atom::Atom::{self, *};
use crate::syntax::{derive, Trait, Type, Types};

impl<'a> Types<'a> {
    pub fn is_guaranteed_pod(&self, ty: &Type) -> bool {
        match ty {
            Type::Ident(ident) => {
                let ident = &ident.rust;
                if let Some(atom) = Atom::from(ident) {
                    match atom {
                        Bool | Char | U8 | U16 | U32 | U64 | Usize | I8 | I16 | I32 | I64
                        | Isize | F32 | F64 => true,
                        CxxString | RustString => false,
                    }
                } else if let Some(strct) = self.structs.get(ident) {
                    derive::contains(&strct.derives, Trait::Copy)
                        || strct
                            .fields
                            .iter()
                            .all(|field| self.is_guaranteed_pod(&field.ty))
                } else {
                    self.enums.contains_key(ident)
                }
            }
            Type::RustBox(_)
            | Type::RustVec(_)
            | Type::UniquePtr(_)
            | Type::SharedPtr(_)
            | Type::WeakPtr(_)
            | Type::CxxVector(_)
            | Type::Void(_) => false,
            Type::Ref(_) | Type::Str(_) | Type::Fn(_) | Type::SliceRef(_) | Type::Ptr(_) => true,
            Type::Array(array) => self.is_guaranteed_pod(&array.inner),
        }
    }
}
