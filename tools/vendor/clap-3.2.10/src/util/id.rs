use crate::util::fnv::Key;

use std::{
    fmt::{Debug, Formatter, Result},
    hash::{Hash, Hasher},
    ops::Deref,
};

#[derive(Clone, Eq, Default)]
#[cfg_attr(not(debug_assertions), repr(transparent))]
pub(crate) struct Id {
    #[cfg(debug_assertions)]
    name: String,
    id: u64,
}

macro_rules! precomputed_hashes {
    ($($fn_name:ident, $const:expr, $name:expr;)*) => {
        impl Id {
            $(
                pub(crate) fn $fn_name() -> Self {
                    Id {
                        #[cfg(debug_assertions)]
                        name: $name.into(),
                        id: $const,
                    }
                }
            )*
        }
    };
}

// precompute some common values
precomputed_hashes! {
    empty_hash,   0x1C9D_3ADB_639F_298E, "";
    help_hash,    0x5963_6393_CFFB_FE5F, "help";
    version_hash, 0x30FF_0B7C_4D07_9478, "version";
}

impl Id {
    pub(crate) fn from_ref<T: Key>(val: T) -> Self {
        Id {
            #[cfg(debug_assertions)]
            name: val.to_string(),
            id: val.key(),
        }
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut Formatter) -> Result {
        #[cfg(debug_assertions)]
        write!(f, "{}", self.name)?;
        #[cfg(not(debug_assertions))]
        write!(f, "[hash: {:X}]", self.id)?;

        Ok(())
    }
}

impl Deref for Id {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl<T: Key> From<T> for Id {
    fn from(val: T) -> Self {
        Id {
            #[cfg(debug_assertions)]
            name: val.to_string(),
            id: val.key(),
        }
    }
}

impl Hash for Id {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state)
    }
}

impl PartialEq for Id {
    fn eq(&self, other: &Id) -> bool {
        self.id == other.id
    }
}
