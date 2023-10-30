// vim: tw=80
use super::*;

use quote::ToTokens;
use std::collections::HashSet;

use crate::{
    mock_function::MockFunction,
    mock_trait::MockTrait
};

fn phantom_default_inits(generics: &Generics) -> Vec<TokenStream> {
    generics.params
    .iter()
    .enumerate()
    .map(|(count, _param)| {
        let phident = format_ident!("_t{}", count);
        quote!(#phident: ::std::marker::PhantomData)
    }).collect()
}

/// Generate any PhantomData field definitions
fn phantom_fields(generics: &Generics) -> Vec<TokenStream> {
    generics.params
    .iter()
    .enumerate()
    .filter_map(|(count, param)| {
        let phident = format_ident!("_t{}", count);
        match param {
            syn::GenericParam::Lifetime(l) => {
                if !l.bounds.is_empty() {
                    compile_error(l.bounds.span(),
                        "#automock does not yet support lifetime bounds on structs");
                }
                let lifetime = &l.lifetime;
                Some(
                quote!(#phident: ::std::marker::PhantomData<&#lifetime ()>)
                )
            },
            syn::GenericParam::Type(tp) => {
                let ty = &tp.ident;
                Some(
                quote!(#phident: ::std::marker::PhantomData<#ty>)
                )
            },
            syn::GenericParam::Const(_) => {
                compile_error(param.span(),
                    "#automock does not yet support generic constants");
                None
            }
        }
    }).collect()
}

/// Filter out multiple copies of the same trait, even if they're implemented on
/// different types.  But allow them if they have different attributes, which 
/// probably indicates that they aren't meant to be compiled together.
fn unique_trait_iter<'a, I: Iterator<Item = &'a MockTrait>>(i: I)
    -> impl Iterator<Item = &'a MockTrait>
{
    let mut hs = HashSet::<(Path, Vec<Attribute>)>::default();
    i.filter(move |mt| {
        let impl_attrs = AttrFormatter::new(&mt.attrs)
            .async_trait(false)
            .doc(false)
            .format();
        let key = (mt.trait_path.clone(), impl_attrs);
        if hs.contains(&key) {
            false
        } else {
            hs.insert(key);
            true
        }
    })
}

/// A collection of methods defined in one spot
struct Methods(Vec<MockFunction>);

impl Methods {
    /// Are all of these methods static?
    fn all_static(&self) -> bool {
        self.0.iter()
            .all(|meth| meth.is_static())
    }

    fn checkpoints(&self) -> Vec<impl ToTokens> {
        self.0.iter()
            .filter(|meth| !meth.is_static())
            .map(|meth| meth.checkpoint())
            .collect::<Vec<_>>()
    }

    /// Return a fragment of code to initialize struct fields during default()
    fn default_inits(&self) -> Vec<TokenStream> {
        self.0.iter()
            .filter(|meth| !meth.is_static())
            .map(|meth| {
                let name = meth.name();
                let attrs = AttrFormatter::new(&meth.attrs)
                    .doc(false)
                    .format();
                quote!(#(#attrs)* #name: Default::default())
            }).collect::<Vec<_>>()
    }

    fn field_definitions(&self, modname: &Ident) -> Vec<TokenStream> {
        self.0.iter()
            .filter(|meth| !meth.is_static())
            .map(|meth| meth.field_definition(Some(modname)))
            .collect::<Vec<_>>()
    }

    fn priv_mods(&self) -> Vec<impl ToTokens> {
        self.0.iter()
            .map(|meth| meth.priv_module())
            .collect::<Vec<_>>()
    }
}

pub(crate) struct MockItemStruct {
    attrs: Vec<Attribute>,
    consts: Vec<ImplItemConst>,
    generics: Generics,
    /// Should Mockall generate a Debug implementation?
    auto_debug: bool,
    /// Does the original struct have a `new` method?
    has_new: bool,
    /// Inherent methods of the mock struct
    methods: Methods,
    /// Name of the overall module that holds all of the mock stuff
    modname: Ident,
    name: Ident,
    /// Is this a whole MockStruct or just a substructure for a trait impl?
    traits: Vec<MockTrait>,
    vis: Visibility,
}

impl MockItemStruct {
    fn debug_impl(&self) -> impl ToTokens {
        if self.auto_debug {
            let (ig, tg, wc) = self.generics.split_for_impl();
            let struct_name = &self.name;
            let struct_name_str = format!("{}", self.name);
            quote!(
                impl #ig ::std::fmt::Debug for #struct_name #tg #wc {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>)
                        -> ::std::result::Result<(), std::fmt::Error>
                    {
                        f.debug_struct(#struct_name_str).finish()
                    }
                }
            )
        } else {
            quote!()
        }
    }

    fn new_method(&self) -> impl ToTokens {
        if self.has_new {
            TokenStream::new()
        } else {
            quote!(
                /// Create a new mock object with no expectations.
                ///
                /// This method will not be generated if the real struct
                /// already has a `new` method.  However, it *will* be
                /// generated if the struct implements a trait with a `new`
                /// method.  The trait's `new` method can still be called
                /// like `<MockX as TraitY>::new`
                pub fn new() -> Self {
                    Self::default()
                }
            )
        }
    }

    fn phantom_default_inits(&self) -> Vec<TokenStream> {
        phantom_default_inits(&self.generics)
    }

    fn phantom_fields(&self) -> Vec<TokenStream> {
        phantom_fields(&self.generics)
    }
}

impl From<MockableStruct> for MockItemStruct {
    fn from(mockable: MockableStruct) -> MockItemStruct {
        let auto_debug = mockable.derives_debug();
        let modname = gen_mod_ident(&mockable.name, None);
        let generics = mockable.generics.clone();
        let struct_name = &mockable.name;
        let vis = mockable.vis;
        let has_new = mockable.methods.iter()
            .any(|meth| meth.sig.ident == "new") ||
            mockable.impls.iter()
            .any(|impl_|
                impl_.items.iter()
                    .any(|ii| if let ImplItem::Method(iim) = ii {
                            iim.sig.ident == "new"
                        } else {
                            false
                        }
                    )
            );
        let methods = Methods(mockable.methods.into_iter()
            .map(|meth|
                mock_function::Builder::new(&meth.sig, &meth.vis)
                    .attrs(&meth.attrs)
                    .struct_(struct_name)
                    .struct_generics(&generics)
                    .levels(2)
                    .call_levels(0)
                    .build()
            ).collect::<Vec<_>>());
        let structname = &mockable.name;
        let traits = mockable.impls.into_iter()
            .map(|i| MockTrait::new(structname, &generics, i, &vis))
            .collect();

        MockItemStruct {
            attrs: mockable.attrs,
            auto_debug,
            consts: mockable.consts,
            generics,
            has_new,
            methods,
            modname,
            name: mockable.name,
            traits,
            vis
        }
    }
}

impl ToTokens for MockItemStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = AttrFormatter::new(&self.attrs)
            .async_trait(false)
            .format();
        let consts = &self.consts;
        let debug_impl = self.debug_impl();
        let struct_name = &self.name;
        let (ig, tg, wc) = self.generics.split_for_impl();
        let modname = &self.modname;
        let calls = self.methods.0.iter()
            .map(|meth| meth.call(Some(modname)))
            .collect::<Vec<_>>();
        let contexts = self.methods.0.iter()
            .filter(|meth| meth.is_static())
            .map(|meth| meth.context_fn(Some(modname)))
            .collect::<Vec<_>>();
        let expects = self.methods.0.iter()
            .filter(|meth| !meth.is_static())
            .map(|meth| meth.expect(modname, None))
            .collect::<Vec<_>>();
        let method_checkpoints = self.methods.checkpoints();
        let new_method = self.new_method();
        let priv_mods = self.methods.priv_mods();
        let substructs = unique_trait_iter(self.traits.iter())
            .map(|trait_| {
                MockItemTraitImpl {
                    attrs: trait_.attrs.clone(),
                    generics: self.generics.clone(),
                    fieldname: format_ident!("{}_expectations",
                                             trait_.ss_name()),
                    methods: Methods(trait_.methods.clone()),
                    modname: format_ident!("{}_{}", &self.modname,
                                           trait_.ss_name()),
                    name: format_ident!("{}_{}", &self.name, trait_.ss_name()),
                }
            }).collect::<Vec<_>>();
        let substruct_expectations = substructs.iter()
            .filter(|ss| !ss.all_static())
            .map(|ss| {
                let attrs = AttrFormatter::new(&ss.attrs)
                    .async_trait(false)
                    .doc(false)
                    .format();
                let fieldname = &ss.fieldname;
                quote!(#(#attrs)* self.#fieldname.checkpoint();)
            }).collect::<Vec<_>>();
        let mut field_definitions = substructs.iter()
            .filter(|ss| !ss.all_static())
            .map(|ss| {
                let attrs = AttrFormatter::new(&ss.attrs)
                    .async_trait(false)
                    .doc(false)
                    .format();
                let fieldname = &ss.fieldname;
                let tyname = &ss.name;
                quote!(#(#attrs)* #fieldname: #tyname #tg)
            }).collect::<Vec<_>>();
        field_definitions.extend(self.methods.field_definitions(modname));
        field_definitions.extend(self.phantom_fields());
        let mut default_inits = substructs.iter()
            .filter(|ss| !ss.all_static())
            .map(|ss| {
                let attrs = AttrFormatter::new(&ss.attrs)
                    .async_trait(false)
                    .doc(false)
                    .format();
                let fieldname = &ss.fieldname;
                quote!(#(#attrs)* #fieldname: Default::default())
            }).collect::<Vec<_>>();
        default_inits.extend(self.methods.default_inits());
        default_inits.extend(self.phantom_default_inits());
        let trait_impls = self.traits.iter()
            .map(|trait_| {
                let modname = format_ident!("{}_{}", &self.modname,
                                            trait_.ss_name());
                trait_.trait_impl(&modname)
            }).collect::<Vec<_>>();
        let vis = &self.vis;
        quote!(
            #[allow(non_snake_case)]
            #[allow(missing_docs)]
            pub mod #modname {
                use super::*;
                #(#priv_mods)*
            }
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            #[allow(missing_docs)]
            #(#attrs)*
            #vis struct #struct_name #ig #wc
            {
                #(#field_definitions),*
            }
            #debug_impl
            impl #ig ::std::default::Default for #struct_name #tg #wc {
                #[allow(clippy::default_trait_access)]
                fn default() -> Self {
                    Self {
                        #(#default_inits),*
                    }
                }
            }
            #(#substructs)*
            impl #ig #struct_name #tg #wc {
                #(#consts)*
                #(#calls)*
                #(#contexts)*
                #(#expects)*
                /// Validate that all current expectations for all methods have
                /// been satisfied, and discard them.
                pub fn checkpoint(&mut self) {
                    #(#substruct_expectations)*
                    #(#method_checkpoints)*
                }
                #new_method
            }
            #(#trait_impls)*
        ).to_tokens(tokens);
    }
}

pub(crate) struct MockItemTraitImpl {
    attrs: Vec<Attribute>,
    generics: Generics,
    /// Inherent methods of the mock struct
    methods: Methods,
    /// Name of the overall module that holds all of the mock stuff
    modname: Ident,
    name: Ident,
    /// Name of the field of this type in the parent's structure
    fieldname: Ident,
}

impl MockItemTraitImpl {
    /// Are all of this traits's methods static?
    fn all_static(&self) -> bool {
        self.methods.all_static()
    }

    fn phantom_default_inits(&self) -> Vec<TokenStream> {
        phantom_default_inits(&self.generics)
    }

    fn phantom_fields(&self) -> Vec<TokenStream> {
        phantom_fields(&self.generics)
    }
}

impl ToTokens for MockItemTraitImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attrs = AttrFormatter::new(&self.attrs)
            .async_trait(false)
            .doc(false)
            .format();
        let struct_name = &self.name;
        let (ig, tg, wc) = self.generics.split_for_impl();
        let modname = &self.modname;
        let method_checkpoints = self.methods.checkpoints();
        let mut default_inits = self.methods.default_inits();
        default_inits.extend(self.phantom_default_inits());
        let mut field_definitions = self.methods.field_definitions(modname);
        field_definitions.extend(self.phantom_fields());
        let priv_mods = self.methods.priv_mods();
        quote!(
            #[allow(non_snake_case)]
            #[allow(missing_docs)]
            #(#attrs)*
            pub mod #modname {
                use super::*;
                #(#priv_mods)*
            }
            #[allow(non_camel_case_types)]
            #[allow(non_snake_case)]
            #[allow(missing_docs)]
            #(#attrs)*
            struct #struct_name #ig #wc
            {
                #(#field_definitions),*
            }
            #(#attrs)*
            impl #ig ::std::default::Default for #struct_name #tg #wc {
                fn default() -> Self {
                    Self {
                        #(#default_inits),*
                    }
                }
            }
            #(#attrs)*
            impl #ig #struct_name #tg #wc {
                /// Validate that all current expectations for all methods have
                /// been satisfied, and discard them.
                pub fn checkpoint(&mut self) {
                    #(#method_checkpoints)*
                }
            }
        ).to_tokens(tokens);
    }
}
