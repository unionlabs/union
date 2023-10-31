use crate::syntax::improper::ImproperCtype;
use crate::syntax::instantiate::ImplKey;
use crate::syntax::map::{OrderedMap, UnorderedMap};
use crate::syntax::report::Errors;
use crate::syntax::resolve::Resolution;
use crate::syntax::set::{OrderedSet, UnorderedSet};
use crate::syntax::trivial::{self, TrivialReason};
use crate::syntax::visit::{self, Visit};
use crate::syntax::{
    toposort, Api, Atom, Enum, EnumRepr, ExternType, Impl, Lifetimes, Pair, Struct, Type, TypeAlias,
};
use proc_macro2::Ident;
use quote::ToTokens;

pub struct Types<'a> {
    pub all: OrderedSet<&'a Type>,
    pub structs: UnorderedMap<&'a Ident, &'a Struct>,
    pub enums: UnorderedMap<&'a Ident, &'a Enum>,
    pub cxx: UnorderedSet<&'a Ident>,
    pub rust: UnorderedSet<&'a Ident>,
    pub aliases: UnorderedMap<&'a Ident, &'a TypeAlias>,
    pub untrusted: UnorderedMap<&'a Ident, &'a ExternType>,
    pub required_trivial: UnorderedMap<&'a Ident, Vec<TrivialReason<'a>>>,
    pub impls: OrderedMap<ImplKey<'a>, Option<&'a Impl>>,
    pub resolutions: UnorderedMap<&'a Ident, Resolution<'a>>,
    pub struct_improper_ctypes: UnorderedSet<&'a Ident>,
    pub toposorted_structs: Vec<&'a Struct>,
}

impl<'a> Types<'a> {
    pub fn collect(cx: &mut Errors, apis: &'a [Api]) -> Self {
        let mut all = OrderedSet::new();
        let mut structs = UnorderedMap::new();
        let mut enums = UnorderedMap::new();
        let mut cxx = UnorderedSet::new();
        let mut rust = UnorderedSet::new();
        let mut aliases = UnorderedMap::new();
        let mut untrusted = UnorderedMap::new();
        let mut impls = OrderedMap::new();
        let mut resolutions = UnorderedMap::new();
        let struct_improper_ctypes = UnorderedSet::new();
        let toposorted_structs = Vec::new();

        fn visit<'a>(all: &mut OrderedSet<&'a Type>, ty: &'a Type) {
            struct CollectTypes<'s, 'a>(&'s mut OrderedSet<&'a Type>);

            impl<'s, 'a> Visit<'a> for CollectTypes<'s, 'a> {
                fn visit_type(&mut self, ty: &'a Type) {
                    self.0.insert(ty);
                    visit::visit_type(self, ty);
                }
            }

            CollectTypes(all).visit_type(ty);
        }

        let mut add_resolution = |name: &'a Pair, generics: &'a Lifetimes| {
            resolutions.insert(&name.rust, Resolution { name, generics });
        };

        let mut type_names = UnorderedSet::new();
        let mut function_names = UnorderedSet::new();
        for api in apis {
            // The same identifier is permitted to be declared as both a shared
            // enum and extern C++ type, or shared struct and extern C++ type.
            // That indicates to not emit the C++ enum/struct definition because
            // it's defined by the included headers already.
            //
            // All other cases of duplicate identifiers are reported as an error.
            match api {
                Api::Include(_) => {}
                Api::Struct(strct) => {
                    let ident = &strct.name.rust;
                    if !type_names.insert(ident)
                        && (!cxx.contains(ident)
                            || structs.contains_key(ident)
                            || enums.contains_key(ident))
                    {
                        // If already declared as a struct or enum, or if
                        // colliding with something other than an extern C++
                        // type, then error.
                        duplicate_name(cx, strct, ident);
                    }
                    structs.insert(&strct.name.rust, strct);
                    for field in &strct.fields {
                        visit(&mut all, &field.ty);
                    }
                    add_resolution(&strct.name, &strct.generics);
                }
                Api::Enum(enm) => {
                    match &enm.repr {
                        EnumRepr::Native { atom: _, repr_type } => {
                            all.insert(repr_type);
                        }
                        #[cfg(feature = "experimental-enum-variants-from-header")]
                        EnumRepr::Foreign { rust_type: _ } => {}
                    }
                    let ident = &enm.name.rust;
                    if !type_names.insert(ident)
                        && (!cxx.contains(ident)
                            || structs.contains_key(ident)
                            || enums.contains_key(ident))
                    {
                        // If already declared as a struct or enum, or if
                        // colliding with something other than an extern C++
                        // type, then error.
                        duplicate_name(cx, enm, ident);
                    }
                    enums.insert(ident, enm);
                    if enm.variants_from_header {
                        // #![variants_from_header] enums are implicitly extern
                        // C++ type.
                        cxx.insert(&enm.name.rust);
                    }
                    add_resolution(&enm.name, &enm.generics);
                }
                Api::CxxType(ety) => {
                    let ident = &ety.name.rust;
                    if !type_names.insert(ident)
                        && (cxx.contains(ident)
                            || !structs.contains_key(ident) && !enums.contains_key(ident))
                    {
                        // If already declared as an extern C++ type, or if
                        // colliding with something which is neither struct nor
                        // enum, then error.
                        duplicate_name(cx, ety, ident);
                    }
                    cxx.insert(ident);
                    if !ety.trusted {
                        untrusted.insert(ident, ety);
                    }
                    add_resolution(&ety.name, &ety.generics);
                }
                Api::RustType(ety) => {
                    let ident = &ety.name.rust;
                    if !type_names.insert(ident) {
                        duplicate_name(cx, ety, ident);
                    }
                    rust.insert(ident);
                    add_resolution(&ety.name, &ety.generics);
                }
                Api::CxxFunction(efn) | Api::RustFunction(efn) => {
                    // Note: duplication of the C++ name is fine because C++ has
                    // function overloading.
                    if !function_names.insert((&efn.receiver, &efn.name.rust)) {
                        duplicate_name(cx, efn, &efn.name.rust);
                    }
                    for arg in &efn.args {
                        visit(&mut all, &arg.ty);
                    }
                    if let Some(ret) = &efn.ret {
                        visit(&mut all, ret);
                    }
                }
                Api::TypeAlias(alias) => {
                    let ident = &alias.name.rust;
                    if !type_names.insert(ident) {
                        duplicate_name(cx, alias, ident);
                    }
                    cxx.insert(ident);
                    aliases.insert(ident, alias);
                    add_resolution(&alias.name, &alias.generics);
                }
                Api::Impl(imp) => {
                    visit(&mut all, &imp.ty);
                    if let Some(key) = imp.ty.impl_key() {
                        impls.insert(key, Some(imp));
                    }
                }
            }
        }

        for ty in &all {
            let impl_key = match ty.impl_key() {
                Some(impl_key) => impl_key,
                None => continue,
            };
            let implicit_impl = match impl_key {
                ImplKey::RustBox(ident)
                | ImplKey::RustVec(ident)
                | ImplKey::UniquePtr(ident)
                | ImplKey::SharedPtr(ident)
                | ImplKey::WeakPtr(ident)
                | ImplKey::CxxVector(ident) => {
                    Atom::from(ident.rust).is_none() && !aliases.contains_key(ident.rust)
                }
            };
            if implicit_impl && !impls.contains_key(&impl_key) {
                impls.insert(impl_key, None);
            }
        }

        // All these APIs may contain types passed by value. We need to ensure
        // we check that this is permissible. We do this _after_ scanning all
        // the APIs above, in case some function or struct references a type
        // which is declared subsequently.
        let required_trivial =
            trivial::required_trivial_reasons(apis, &all, &structs, &enums, &cxx);

        let mut types = Types {
            all,
            structs,
            enums,
            cxx,
            rust,
            aliases,
            untrusted,
            required_trivial,
            impls,
            resolutions,
            struct_improper_ctypes,
            toposorted_structs,
        };

        types.toposorted_structs = toposort::sort(cx, apis, &types);

        let mut unresolved_structs = types.structs.keys();
        let mut new_information = true;
        while new_information {
            new_information = false;
            unresolved_structs.retain(|ident| {
                let mut retain = false;
                for var in &types.structs[ident].fields {
                    if match types.determine_improper_ctype(&var.ty) {
                        ImproperCtype::Depends(inner) => {
                            retain = true;
                            types.struct_improper_ctypes.contains(inner)
                        }
                        ImproperCtype::Definite(improper) => improper,
                    } {
                        types.struct_improper_ctypes.insert(ident);
                        new_information = true;
                        return false;
                    }
                }
                // If all fields definite false, remove from unresolved_structs.
                retain
            });
        }

        types
    }

    pub fn needs_indirect_abi(&self, ty: &Type) -> bool {
        match ty {
            Type::RustBox(_) | Type::UniquePtr(_) => false,
            Type::Array(_) => true,
            _ => !self.is_guaranteed_pod(ty),
        }
    }

    // Types that trigger rustc's default #[warn(improper_ctypes)] lint, even if
    // they may be otherwise unproblematic to mention in an extern signature.
    // For example in a signature like `extern "C" fn(*const String)`, rustc
    // refuses to believe that C could know how to supply us with a pointer to a
    // Rust String, even though C could easily have obtained that pointer
    // legitimately from a Rust call.
    pub fn is_considered_improper_ctype(&self, ty: &Type) -> bool {
        match self.determine_improper_ctype(ty) {
            ImproperCtype::Definite(improper) => improper,
            ImproperCtype::Depends(ident) => self.struct_improper_ctypes.contains(ident),
        }
    }

    // Types which we need to assume could possibly exist by value on the Rust
    // side.
    pub fn is_maybe_trivial(&self, ty: &Ident) -> bool {
        self.structs.contains_key(ty)
            || self.enums.contains_key(ty)
            || self.aliases.contains_key(ty)
    }
}

impl<'t, 'a> IntoIterator for &'t Types<'a> {
    type Item = &'a Type;
    type IntoIter = crate::syntax::set::Iter<'t, 'a, Type>;
    fn into_iter(self) -> Self::IntoIter {
        self.all.into_iter()
    }
}

fn duplicate_name(cx: &mut Errors, sp: impl ToTokens, ident: &Ident) {
    let msg = format!("the name `{}` is defined multiple times", ident);
    cx.error(sp, msg);
}
