use crate::syntax::namespace::Namespace;
use crate::syntax::Api;

impl Api {
    pub fn namespace(&self) -> &Namespace {
        match self {
            Api::CxxFunction(efn) | Api::RustFunction(efn) => &efn.name.namespace,
            Api::CxxType(ety) | Api::RustType(ety) => &ety.name.namespace,
            Api::Enum(enm) => &enm.name.namespace,
            Api::Struct(strct) => &strct.name.namespace,
            Api::Impl(_) | Api::Include(_) | Api::TypeAlias(_) => Default::default(),
        }
    }
}
