use crate::ast;
use crate::encode;
use crate::util::ShortHash;
use crate::Diagnostic;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{quote, ToTokens};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use syn;
use wasm_bindgen_shared as shared;

/// A trait for converting AST structs into Tokens and adding them to a TokenStream,
/// or providing a diagnostic if conversion fails.
pub trait TryToTokens {
    /// Attempt to convert a `Self` into tokens and add it to the `TokenStream`
    fn try_to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostic>;

    /// Attempt to convert a `Self` into a new `TokenStream`
    fn try_to_token_stream(&self) -> Result<TokenStream, Diagnostic> {
        let mut tokens = TokenStream::new();
        self.try_to_tokens(&mut tokens)?;
        Ok(tokens)
    }
}

impl TryToTokens for ast::Program {
    // Generate wrappers for all the items that we've found
    fn try_to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostic> {
        let mut errors = Vec::new();
        for export in self.exports.iter() {
            if let Err(e) = export.try_to_tokens(tokens) {
                errors.push(e);
            }
        }
        for s in self.structs.iter() {
            s.to_tokens(tokens);
        }
        let mut types = HashMap::new();
        for i in self.imports.iter() {
            if let ast::ImportKind::Type(t) = &i.kind {
                types.insert(t.rust_name.to_string(), t.rust_name.clone());
            }
        }
        for i in self.imports.iter() {
            DescribeImport { kind: &i.kind }.to_tokens(tokens);

            // If there is a js namespace, check that name isn't a type. If it is,
            // this import might be a method on that type.
            if let Some(nss) = &i.js_namespace {
                // When the namespace is `A.B`, the type name should be `B`.
                if let Some(ns) = nss.last().and_then(|t| types.get(t)) {
                    if i.kind.fits_on_impl() {
                        let kind = match i.kind.try_to_token_stream() {
                            Ok(kind) => kind,
                            Err(e) => {
                                errors.push(e);
                                continue;
                            }
                        };
                        (quote! {
                            #[automatically_derived]
                            impl #ns { #kind }
                        })
                        .to_tokens(tokens);
                        continue;
                    }
                }
            }

            if let Err(e) = i.kind.try_to_tokens(tokens) {
                errors.push(e);
            }
        }
        for e in self.enums.iter() {
            e.to_tokens(tokens);
        }

        Diagnostic::from_vec(errors)?;

        // Generate a static which will eventually be what lives in a custom section
        // of the wasm executable. For now it's just a plain old static, but we'll
        // eventually have it actually in its own section.

        static CNT: AtomicUsize = AtomicUsize::new(0);

        let generated_static_name = format!(
            "__WASM_BINDGEN_GENERATED_{}",
            ShortHash(CNT.fetch_add(1, Ordering::SeqCst)),
        );
        let generated_static_name = Ident::new(&generated_static_name, Span::call_site());

        // See comments in `crates/cli-support/src/lib.rs` about what this
        // `schema_version` is.
        let prefix_json = format!(
            r#"{{"schema_version":"{}","version":"{}"}}"#,
            shared::SCHEMA_VERSION,
            shared::version()
        );
        let encoded = encode::encode(self)?;
        let mut bytes = Vec::new();
        bytes.push((prefix_json.len() >> 0) as u8);
        bytes.push((prefix_json.len() >> 8) as u8);
        bytes.push((prefix_json.len() >> 16) as u8);
        bytes.push((prefix_json.len() >> 24) as u8);
        bytes.extend_from_slice(prefix_json.as_bytes());
        bytes.extend_from_slice(&encoded.custom_section);

        let generated_static_length = bytes.len();
        let generated_static_value = syn::LitByteStr::new(&bytes, Span::call_site());

        // We already consumed the contents of included files when generating
        // the custom section, but we want to make sure that updates to the
        // generated files will cause this macro to rerun incrementally. To do
        // that we use `include_str!` to force rustc to think it has a
        // dependency on these files. That way when the file changes Cargo will
        // automatically rerun rustc which will rerun this macro. Other than
        // this we don't actually need the results of the `include_str!`, so
        // it's just shoved into an anonymous static.
        let file_dependencies = encoded.included_files.iter().map(|file| {
            let file = file.to_str().unwrap();
            quote! { include_str!(#file) }
        });

        (quote! {
            #[cfg(target_arch = "wasm32")]
            #[automatically_derived]
            #[link_section = "__wasm_bindgen_unstable"]
            #[doc(hidden)]
            pub static #generated_static_name: [u8; #generated_static_length] = {
                static _INCLUDED_FILES: &[&str] = &[#(#file_dependencies),*];

                *#generated_static_value
            };

        })
        .to_tokens(tokens);

        Ok(())
    }
}

impl ToTokens for ast::Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &self.rust_name;
        let name_str = self.js_name.to_string();
        let name_len = name_str.len() as u32;
        let name_chars = name_str.chars().map(|c| c as u32);
        let new_fn = Ident::new(&shared::new_function(&name_str), Span::call_site());
        let free_fn = Ident::new(&shared::free_function(&name_str), Span::call_site());
        let free_fn_const = Ident::new(&format!("{}__const", free_fn), free_fn.span());
        (quote! {
            #[automatically_derived]
            impl wasm_bindgen::describe::WasmDescribe for #name {
                fn describe() {
                    use wasm_bindgen::__wbindgen_if_not_std;
                    __wbindgen_if_not_std! {
                        compile_error! {
                            "exporting a class to JS requires the `std` feature to \
                             be enabled in the `wasm-bindgen` crate"
                        }
                    }
                    use wasm_bindgen::describe::*;
                    inform(RUST_STRUCT);
                    inform(#name_len);
                    #(inform(#name_chars);)*
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::IntoWasmAbi for #name {
                type Abi = u32;

                fn into_abi(self) -> u32 {
                    use wasm_bindgen::__rt::std::boxed::Box;
                    use wasm_bindgen::__rt::WasmRefCell;
                    Box::into_raw(Box::new(WasmRefCell::new(self))) as u32
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::FromWasmAbi for #name {
                type Abi = u32;

                unsafe fn from_abi(js: u32) -> Self {
                    use wasm_bindgen::__rt::std::boxed::Box;
                    use wasm_bindgen::__rt::{assert_not_null, WasmRefCell};

                    let ptr = js as *mut WasmRefCell<#name>;
                    assert_not_null(ptr);
                    let js = Box::from_raw(ptr);
                    (*js).borrow_mut(); // make sure no one's borrowing
                    js.into_inner()
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::__rt::core::convert::From<#name> for
                wasm_bindgen::JsValue
            {
                fn from(value: #name) -> Self {
                    let ptr = wasm_bindgen::convert::IntoWasmAbi::into_abi(value);

                    #[link(wasm_import_module = "__wbindgen_placeholder__")]
                    #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
                    extern "C" {
                        fn #new_fn(ptr: u32) -> u32;
                    }

                    #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
                    unsafe fn #new_fn(_: u32) -> u32 {
                        panic!("cannot convert to JsValue outside of the wasm target")
                    }

                    unsafe {
                        <wasm_bindgen::JsValue as wasm_bindgen::convert::FromWasmAbi>
                            ::from_abi(#new_fn(ptr))
                    }
                }
            }

            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            #[automatically_derived]
            const #free_fn_const: () = {
                #[no_mangle]
                #[doc(hidden)]
                pub unsafe extern "C" fn #free_fn(ptr: u32) {
                    drop(<#name as wasm_bindgen::convert::FromWasmAbi>::from_abi(ptr));
                }
            };

            #[automatically_derived]
            impl wasm_bindgen::convert::RefFromWasmAbi for #name {
                type Abi = u32;
                type Anchor = wasm_bindgen::__rt::Ref<'static, #name>;

                unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                    let js = js as *mut wasm_bindgen::__rt::WasmRefCell<#name>;
                    wasm_bindgen::__rt::assert_not_null(js);
                    (*js).borrow()
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::RefMutFromWasmAbi for #name {
                type Abi = u32;
                type Anchor = wasm_bindgen::__rt::RefMut<'static, #name>;

                unsafe fn ref_mut_from_abi(js: Self::Abi) -> Self::Anchor {
                    let js = js as *mut wasm_bindgen::__rt::WasmRefCell<#name>;
                    wasm_bindgen::__rt::assert_not_null(js);
                    (*js).borrow_mut()
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::OptionIntoWasmAbi for #name {
                #[inline]
                fn none() -> Self::Abi { 0 }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::OptionFromWasmAbi for #name {
                #[inline]
                fn is_none(abi: &Self::Abi) -> bool { *abi == 0 }
            }
        })
        .to_tokens(tokens);

        for field in self.fields.iter() {
            field.to_tokens(tokens);
        }
    }
}

impl ToTokens for ast::StructField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let rust_name = &self.rust_name;
        let struct_name = &self.struct_name;
        let ty = &self.ty;
        let getter = &self.getter;
        let setter = &self.setter;

        let maybe_assert_copy = if self.getter_with_clone {
            quote! {}
        } else {
            quote! { assert_copy::<#ty>() }
        };
        let maybe_assert_copy = respan(maybe_assert_copy, ty);

        let maybe_clone = if self.getter_with_clone {
            quote! { .clone() }
        } else {
            quote! {}
        };

        let getter_const = Ident::new(&format!("{}__const", getter), getter.span());

        (quote! {
            #[automatically_derived]
            const #getter_const: () = {
                #[cfg_attr(all(target_arch = "wasm32", not(target_os = "emscripten")), no_mangle)]
                #[doc(hidden)]
                pub unsafe extern "C" fn #getter(js: u32)
                    -> <#ty as wasm_bindgen::convert::IntoWasmAbi>::Abi
                {
                    use wasm_bindgen::__rt::{WasmRefCell, assert_not_null};
                    use wasm_bindgen::convert::IntoWasmAbi;

                    fn assert_copy<T: Copy>(){}
                    #maybe_assert_copy;

                    let js = js as *mut WasmRefCell<#struct_name>;
                    assert_not_null(js);
                    let val = (*js).borrow().#rust_name#maybe_clone;
                    <#ty as IntoWasmAbi>::into_abi(val)
                }
            };
        })
        .to_tokens(tokens);

        Descriptor {
            ident: &getter,
            inner: quote! {
                <#ty as WasmDescribe>::describe();
            },
            attrs: vec![],
        }
        .to_tokens(tokens);

        if self.readonly {
            return;
        }

        let setter_const = Ident::new(&format!("{}__const", setter), setter.span());

        (quote! {
            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            #[automatically_derived]
            const #setter_const: () = {
                #[no_mangle]
                #[doc(hidden)]
                pub unsafe extern "C" fn #setter(
                    js: u32,
                    val: <#ty as wasm_bindgen::convert::FromWasmAbi>::Abi,
                ) {
                    use wasm_bindgen::__rt::{WasmRefCell, assert_not_null};
                    use wasm_bindgen::convert::FromWasmAbi;

                    let js = js as *mut WasmRefCell<#struct_name>;
                    assert_not_null(js);
                    let val = <#ty as FromWasmAbi>::from_abi(val);
                    (*js).borrow_mut().#rust_name = val;
                }
            };
        })
        .to_tokens(tokens);
    }
}

impl TryToTokens for ast::Export {
    fn try_to_tokens(self: &ast::Export, into: &mut TokenStream) -> Result<(), Diagnostic> {
        let generated_name = self.rust_symbol();
        let export_name = self.export_name();
        let mut args = vec![];
        let mut arg_conversions = vec![];
        let mut converted_arguments = vec![];
        let ret = Ident::new("_ret", Span::call_site());

        let offset = if self.method_self.is_some() {
            args.push(quote! { me: u32 });
            1
        } else {
            0
        };

        let name = &self.rust_name;
        let receiver = match self.method_self {
            Some(ast::MethodSelf::ByValue) => {
                let class = self.rust_class.as_ref().unwrap();
                arg_conversions.push(quote! {
                    let me = unsafe {
                        <#class as wasm_bindgen::convert::FromWasmAbi>::from_abi(me)
                    };
                });
                quote! { me.#name }
            }
            Some(ast::MethodSelf::RefMutable) => {
                let class = self.rust_class.as_ref().unwrap();
                arg_conversions.push(quote! {
                    let mut me = unsafe {
                        <#class as wasm_bindgen::convert::RefMutFromWasmAbi>
                            ::ref_mut_from_abi(me)
                    };
                    let me = &mut *me;
                });
                quote! { me.#name }
            }
            Some(ast::MethodSelf::RefShared) => {
                let class = self.rust_class.as_ref().unwrap();
                arg_conversions.push(quote! {
                    let me = unsafe {
                        <#class as wasm_bindgen::convert::RefFromWasmAbi>
                            ::ref_from_abi(me)
                    };
                    let me = &*me;
                });
                quote! { me.#name }
            }
            None => match &self.rust_class {
                Some(class) => quote! { #class::#name },
                None => quote! { #name },
            },
        };

        let mut argtys = Vec::new();
        for (i, arg) in self.function.arguments.iter().enumerate() {
            argtys.push(&arg.ty);
            let i = i + offset;
            let ident = Ident::new(&format!("arg{}", i), Span::call_site());
            let ty = &arg.ty;
            match &*arg.ty {
                syn::Type::Reference(syn::TypeReference {
                    mutability: Some(_),
                    elem,
                    ..
                }) => {
                    args.push(quote! {
                        #ident: <#elem as wasm_bindgen::convert::RefMutFromWasmAbi>::Abi
                    });
                    arg_conversions.push(quote! {
                        let mut #ident = unsafe {
                            <#elem as wasm_bindgen::convert::RefMutFromWasmAbi>
                                ::ref_mut_from_abi(#ident)
                        };
                        let #ident = &mut *#ident;
                    });
                }
                syn::Type::Reference(syn::TypeReference { elem, .. }) => {
                    args.push(quote! {
                        #ident: <#elem as wasm_bindgen::convert::RefFromWasmAbi>::Abi
                    });
                    arg_conversions.push(quote! {
                        let #ident = unsafe {
                            <#elem as wasm_bindgen::convert::RefFromWasmAbi>
                                ::ref_from_abi(#ident)
                        };
                        let #ident = &*#ident;
                    });
                }
                _ => {
                    args.push(quote! {
                        #ident: <#ty as wasm_bindgen::convert::FromWasmAbi>::Abi
                    });
                    arg_conversions.push(quote! {
                        let #ident = unsafe {
                            <#ty as wasm_bindgen::convert::FromWasmAbi>
                                ::from_abi(#ident)
                        };
                    });
                }
            }
            converted_arguments.push(quote! { #ident });
        }
        let syn_unit = syn::Type::Tuple(syn::TypeTuple {
            elems: Default::default(),
            paren_token: Default::default(),
        });
        let syn_ret = self.function.ret.as_ref().unwrap_or(&syn_unit);
        if let syn::Type::Reference(_) = syn_ret {
            bail_span!(syn_ret, "cannot return a borrowed ref with #[wasm_bindgen]",)
        }

        // For an `async` function we always run it through `future_to_promise`
        // since we're returning a promise to JS, and this will implicitly
        // require that the function returns a `Future<Output = Result<...>>`
        let (ret_ty, inner_ret_ty, ret_expr) = if self.function.r#async {
            if self.start {
                (
                    quote! { () },
                    quote! { () },
                    quote! {
                        wasm_bindgen_futures::spawn_local(async move {
                            <#syn_ret as wasm_bindgen::__rt::Start>::start(#ret.await);
                        })
                    },
                )
            } else {
                (
                    quote! { wasm_bindgen::JsValue },
                    quote! { #syn_ret },
                    quote! {
                        wasm_bindgen_futures::future_to_promise(async move {
                            <#syn_ret as wasm_bindgen::__rt::IntoJsResult>::into_js_result(#ret.await)
                        }).into()
                    },
                )
            }
        } else if self.start {
            (
                quote! { () },
                quote! { () },
                quote! { <#syn_ret as wasm_bindgen::__rt::Start>::start(#ret) },
            )
        } else {
            (quote! { #syn_ret }, quote! { #syn_ret }, quote! { #ret })
        };

        let projection = quote! { <#ret_ty as wasm_bindgen::convert::ReturnWasmAbi> };
        let convert_ret = quote! { #projection::return_abi(#ret_expr) };
        let describe_ret = quote! {
            <#ret_ty as WasmDescribe>::describe();
            <#inner_ret_ty as WasmDescribe>::describe();
        };
        let nargs = self.function.arguments.len() as u32;
        let attrs = &self.function.rust_attrs;

        let start_check = if self.start {
            quote! { const _ASSERT: fn() = || -> #projection::Abi { loop {} }; }
        } else {
            quote! {}
        };

        let generated_name_const =
            Ident::new(&format!("{}__const", generated_name), generated_name.span());
        (quote! {
            #[automatically_derived]
            const #generated_name_const: () = {
                #(#attrs)*
                #[cfg_attr(
                    all(target_arch = "wasm32", not(target_os = "emscripten")),
                    export_name = #export_name,
                )]
                pub extern "C" fn #generated_name(#(#args),*) -> #projection::Abi {
                    #start_check
                    // Scope all local variables to be destroyed after we call
                    // the function to ensure that `#convert_ret`, if it panics,
                    // doesn't leak anything.
                    let #ret = {
                        #(#arg_conversions)*
                        #receiver(#(#converted_arguments),*)
                    };
                    #convert_ret
                }
            };
        })
        .to_tokens(into);

        // In addition to generating the shim function above which is what
        // our generated JS will invoke, we *also* generate a "descriptor"
        // shim. This descriptor shim uses the `WasmDescribe` trait to
        // programmatically describe the type signature of the generated
        // shim above. This in turn is then used to inform the
        // `wasm-bindgen` CLI tool exactly what types and such it should be
        // using in JS.
        //
        // Note that this descriptor function is a purely an internal detail
        // of `#[wasm_bindgen]` and isn't intended to be exported to anyone
        // or actually part of the final was binary. Additionally, this is
        // literally executed when the `wasm-bindgen` tool executes.
        //
        // In any case, there's complications in `wasm-bindgen` to handle
        // this, but the tl;dr; is that this is stripped from the final wasm
        // binary along with anything it references.
        let export = Ident::new(&export_name, Span::call_site());
        Descriptor {
            ident: &export,
            inner: quote! {
                inform(FUNCTION);
                inform(0);
                inform(#nargs);
                #(<#argtys as WasmDescribe>::describe();)*
                #describe_ret
            },
            attrs: attrs.clone(),
        }
        .to_tokens(into);

        Ok(())
    }
}

impl TryToTokens for ast::ImportKind {
    fn try_to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostic> {
        match *self {
            ast::ImportKind::Function(ref f) => f.try_to_tokens(tokens)?,
            ast::ImportKind::Static(ref s) => s.to_tokens(tokens),
            ast::ImportKind::Type(ref t) => t.to_tokens(tokens),
            ast::ImportKind::Enum(ref e) => e.to_tokens(tokens),
        }

        Ok(())
    }
}

impl ToTokens for ast::ImportType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.vis;
        let rust_name = &self.rust_name;
        let attrs = &self.attrs;
        let doc_comment = match &self.doc_comment {
            None => "",
            Some(comment) => comment,
        };
        let const_name = format!("__wbg_generated_const_{}", rust_name);
        let const_name = Ident::new(&const_name, Span::call_site());
        let instanceof_shim = Ident::new(&self.instanceof_shim, Span::call_site());

        let internal_obj = match self.extends.first() {
            Some(target) => {
                quote! { #target }
            }
            None => {
                quote! { wasm_bindgen::JsValue }
            }
        };

        let description = if let Some(typescript_type) = &self.typescript_type {
            let typescript_type_len = typescript_type.len() as u32;
            let typescript_type_chars = typescript_type.chars().map(|c| c as u32);
            quote! {
                use wasm_bindgen::describe::*;
                inform(NAMED_EXTERNREF);
                inform(#typescript_type_len);
                #(inform(#typescript_type_chars);)*
            }
        } else {
            quote! {
                JsValue::describe()
            }
        };

        let is_type_of = self.is_type_of.as_ref().map(|is_type_of| {
            quote! {
                #[inline]
                fn is_type_of(val: &JsValue) -> bool {
                    let is_type_of: fn(&JsValue) -> bool = #is_type_of;
                    is_type_of(val)
                }
            }
        });

        let no_deref = self.no_deref;

        (quote! {
            #[automatically_derived]
            #(#attrs)*
            #[doc = #doc_comment]
            #[repr(transparent)]
            #vis struct #rust_name {
                obj: #internal_obj
            }

            #[automatically_derived]
            const #const_name: () = {
                use wasm_bindgen::convert::{IntoWasmAbi, FromWasmAbi};
                use wasm_bindgen::convert::{OptionIntoWasmAbi, OptionFromWasmAbi};
                use wasm_bindgen::convert::RefFromWasmAbi;
                use wasm_bindgen::describe::WasmDescribe;
                use wasm_bindgen::{JsValue, JsCast, JsObject};
                use wasm_bindgen::__rt::core;

                impl WasmDescribe for #rust_name {
                    fn describe() {
                        #description
                    }
                }

                impl IntoWasmAbi for #rust_name {
                    type Abi = <JsValue as IntoWasmAbi>::Abi;

                    #[inline]
                    fn into_abi(self) -> Self::Abi {
                        self.obj.into_abi()
                    }
                }

                impl OptionIntoWasmAbi for #rust_name {
                    #[inline]
                    fn none() -> Self::Abi {
                        0
                    }
                }

                impl<'a> OptionIntoWasmAbi for &'a #rust_name {
                    #[inline]
                    fn none() -> Self::Abi {
                        0
                    }
                }

                impl FromWasmAbi for #rust_name {
                    type Abi = <JsValue as FromWasmAbi>::Abi;

                    #[inline]
                    unsafe fn from_abi(js: Self::Abi) -> Self {
                        #rust_name {
                            obj: JsValue::from_abi(js).into(),
                        }
                    }
                }

                impl OptionFromWasmAbi for #rust_name {
                    #[inline]
                    fn is_none(abi: &Self::Abi) -> bool { *abi == 0 }
                }

                impl<'a> IntoWasmAbi for &'a #rust_name {
                    type Abi = <&'a JsValue as IntoWasmAbi>::Abi;

                    #[inline]
                    fn into_abi(self) -> Self::Abi {
                        (&self.obj).into_abi()
                    }
                }

                impl RefFromWasmAbi for #rust_name {
                    type Abi = <JsValue as RefFromWasmAbi>::Abi;
                    type Anchor = core::mem::ManuallyDrop<#rust_name>;

                    #[inline]
                    unsafe fn ref_from_abi(js: Self::Abi) -> Self::Anchor {
                        let tmp = <JsValue as RefFromWasmAbi>::ref_from_abi(js);
                        core::mem::ManuallyDrop::new(#rust_name {
                            obj: core::mem::ManuallyDrop::into_inner(tmp).into(),
                        })
                    }
                }

                // TODO: remove this on the next major version
                impl From<JsValue> for #rust_name {
                    #[inline]
                    fn from(obj: JsValue) -> #rust_name {
                        #rust_name { obj: obj.into() }
                    }
                }

                impl AsRef<JsValue> for #rust_name {
                    #[inline]
                    fn as_ref(&self) -> &JsValue { self.obj.as_ref() }
                }

                impl AsRef<#rust_name> for #rust_name {
                    #[inline]
                    fn as_ref(&self) -> &#rust_name { self }
                }


                impl From<#rust_name> for JsValue {
                    #[inline]
                    fn from(obj: #rust_name) -> JsValue {
                        obj.obj.into()
                    }
                }

                impl JsCast for #rust_name {
                    fn instanceof(val: &JsValue) -> bool {
                        #[link(wasm_import_module = "__wbindgen_placeholder__")]
                        #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
                        extern "C" {
                            fn #instanceof_shim(val: u32) -> u32;
                        }
                        #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
                        unsafe fn #instanceof_shim(_: u32) -> u32 {
                            panic!("cannot check instanceof on non-wasm targets");
                        }
                        unsafe {
                            let idx = val.into_abi();
                            #instanceof_shim(idx) != 0
                        }
                    }

                    #is_type_of

                    #[inline]
                    fn unchecked_from_js(val: JsValue) -> Self {
                        #rust_name { obj: val.into() }
                    }

                    #[inline]
                    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
                        // Should be safe because `#rust_name` is a transparent
                        // wrapper around `val`
                        unsafe { &*(val as *const JsValue as *const #rust_name) }
                    }
                }

                impl JsObject for #rust_name {}
            };
        })
        .to_tokens(tokens);

        if !no_deref {
            (quote! {
                #[automatically_derived]
                impl core::ops::Deref for #rust_name {
                    type Target = #internal_obj;

                    #[inline]
                    fn deref(&self) -> &#internal_obj {
                        &self.obj
                    }
                }
            })
            .to_tokens(tokens);
        }

        for superclass in self.extends.iter() {
            (quote! {
                #[automatically_derived]
                impl From<#rust_name> for #superclass {
                    #[inline]
                    fn from(obj: #rust_name) -> #superclass {
                        use wasm_bindgen::JsCast;
                        #superclass::unchecked_from_js(obj.into())
                    }
                }

                #[automatically_derived]
                impl AsRef<#superclass> for #rust_name {
                    #[inline]
                    fn as_ref(&self) -> &#superclass {
                        use wasm_bindgen::JsCast;
                        #superclass::unchecked_from_js_ref(self.as_ref())
                    }
                }
            })
            .to_tokens(tokens);
        }
    }
}

impl ToTokens for ast::ImportEnum {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.vis;
        let name = &self.name;
        let expect_string = format!("attempted to convert invalid {} into JSValue", name);
        let variants = &self.variants;
        let variant_strings = &self.variant_values;
        let attrs = &self.rust_attrs;

        let mut current_idx: usize = 0;
        let variant_indexes: Vec<Literal> = variants
            .iter()
            .map(|_| {
                let this_index = current_idx;
                current_idx += 1;
                Literal::usize_unsuffixed(this_index)
            })
            .collect();

        // Borrow variant_indexes because we need to use it multiple times inside the quote! macro
        let variant_indexes_ref = &variant_indexes;

        // A vector of EnumName::VariantName tokens for this enum
        let variant_paths: Vec<TokenStream> = self
            .variants
            .iter()
            .map(|v| quote!(#name::#v).into_token_stream())
            .collect();

        // Borrow variant_paths because we need to use it multiple times inside the quote! macro
        let variant_paths_ref = &variant_paths;

        (quote! {
            #(#attrs)*
            #vis enum #name {
                #(#variants = #variant_indexes_ref,)*
                #[automatically_derived]
                #[doc(hidden)]
                __Nonexhaustive,
            }

            #[automatically_derived]
            impl #name {
                fn from_str(s: &str) -> Option<#name> {
                    match s {
                        #(#variant_strings => Some(#variant_paths_ref),)*
                        _ => None,
                    }
                }

                fn to_str(&self) -> &'static str {
                    match self {
                        #(#variant_paths_ref => #variant_strings,)*
                        #name::__Nonexhaustive => panic!(#expect_string),
                    }
                }

                #vis fn from_js_value(obj: &wasm_bindgen::JsValue) -> Option<#name> {
                    obj.as_string().and_then(|obj_str| Self::from_str(obj_str.as_str()))
                }
            }

            // It should really be using &str for all of these, but that requires some major changes to cli-support
            #[automatically_derived]
            impl wasm_bindgen::describe::WasmDescribe for #name {
                fn describe() {
                    <wasm_bindgen::JsValue as wasm_bindgen::describe::WasmDescribe>::describe()
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::IntoWasmAbi for #name {
                type Abi = <wasm_bindgen::JsValue as wasm_bindgen::convert::IntoWasmAbi>::Abi;

                #[inline]
                fn into_abi(self) -> Self::Abi {
                    <wasm_bindgen::JsValue as wasm_bindgen::convert::IntoWasmAbi>::into_abi(self.into())
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::FromWasmAbi for #name {
                type Abi = <wasm_bindgen::JsValue as wasm_bindgen::convert::FromWasmAbi>::Abi;

                unsafe fn from_abi(js: Self::Abi) -> Self {
                    let s = <wasm_bindgen::JsValue as wasm_bindgen::convert::FromWasmAbi>::from_abi(js);
                    #name::from_js_value(&s).unwrap_or(#name::__Nonexhaustive)
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::OptionIntoWasmAbi for #name {
                #[inline]
                fn none() -> Self::Abi { <::js_sys::Object as wasm_bindgen::convert::OptionIntoWasmAbi>::none() }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::OptionFromWasmAbi for #name {
                #[inline]
                fn is_none(abi: &Self::Abi) -> bool { <::js_sys::Object as wasm_bindgen::convert::OptionFromWasmAbi>::is_none(abi) }
            }

            #[automatically_derived]
            impl From<#name> for wasm_bindgen::JsValue {
                fn from(obj: #name) -> wasm_bindgen::JsValue {
                    wasm_bindgen::JsValue::from(obj.to_str())
                }
            }
        }).to_tokens(tokens);
    }
}

impl TryToTokens for ast::ImportFunction {
    fn try_to_tokens(&self, tokens: &mut TokenStream) -> Result<(), Diagnostic> {
        let mut class_ty = None;
        let mut is_method = false;
        match self.kind {
            ast::ImportFunctionKind::Method {
                ref ty, ref kind, ..
            } => {
                if let ast::MethodKind::Operation(ast::Operation {
                    is_static: false, ..
                }) = kind
                {
                    is_method = true;
                }
                class_ty = Some(ty);
            }
            ast::ImportFunctionKind::Normal => {}
        }
        let vis = &self.function.rust_vis;
        let ret = match &self.function.ret {
            Some(ty) => quote! { -> #ty },
            None => quote!(),
        };

        let mut abi_argument_names = Vec::new();
        let mut abi_arguments = Vec::new();
        let mut arg_conversions = Vec::new();
        let mut arguments = Vec::new();
        let ret_ident = Ident::new("_ret", Span::call_site());

        for (i, arg) in self.function.arguments.iter().enumerate() {
            let ty = &arg.ty;
            let name = match &*arg.pat {
                syn::Pat::Ident(syn::PatIdent {
                    by_ref: None,
                    ident,
                    subpat: None,
                    ..
                }) => ident.clone(),
                syn::Pat::Wild(_) => syn::Ident::new(&format!("__genarg_{}", i), Span::call_site()),
                _ => bail_span!(
                    arg.pat,
                    "unsupported pattern in #[wasm_bindgen] imported function",
                ),
            };

            abi_argument_names.push(name.clone());
            abi_arguments.push(quote! {
                #name: <#ty as wasm_bindgen::convert::IntoWasmAbi>::Abi
            });
            let var = if i == 0 && is_method {
                quote! { self }
            } else {
                arguments.push(quote! { #name: #ty });
                quote! { #name }
            };
            arg_conversions.push(quote! {
                let #name = <#ty as wasm_bindgen::convert::IntoWasmAbi>
                    ::into_abi(#var);
            });
        }
        let abi_ret;
        let mut convert_ret;
        match &self.js_ret {
            Some(syn::Type::Reference(_)) => {
                bail_span!(
                    self.js_ret,
                    "cannot return references in #[wasm_bindgen] imports yet"
                );
            }
            Some(ref ty) => {
                if self.function.r#async {
                    abi_ret =
                        quote! { <js_sys::Promise as wasm_bindgen::convert::FromWasmAbi>::Abi };
                    let future = quote! {
                        wasm_bindgen_futures::JsFuture::from(
                            <js_sys::Promise as wasm_bindgen::convert::FromWasmAbi>
                                ::from_abi(#ret_ident)
                        ).await
                    };
                    convert_ret = if self.catch {
                        quote! { Ok(#future?) }
                    } else {
                        quote! { #future.expect("unexpected exception") }
                    };
                } else {
                    abi_ret = quote! {
                        <#ty as wasm_bindgen::convert::FromWasmAbi>::Abi
                    };
                    convert_ret = quote! {
                        <#ty as wasm_bindgen::convert::FromWasmAbi>
                            ::from_abi(#ret_ident)
                    };
                }
            }
            None => {
                if self.function.r#async {
                    abi_ret =
                        quote! { <js_sys::Promise as wasm_bindgen::convert::FromWasmAbi>::Abi };
                    let future = quote! {
                        wasm_bindgen_futures::JsFuture::from(
                            <js_sys::Promise as wasm_bindgen::convert::FromWasmAbi>
                                ::from_abi(#ret_ident)
                        ).await
                    };
                    convert_ret = if self.catch {
                        quote! { #future?; Ok(()) }
                    } else {
                        quote! { #future.expect("uncaught exception"); }
                    };
                } else {
                    abi_ret = quote! { () };
                    convert_ret = quote! { () };
                }
            }
        }

        let mut exceptional_ret = quote!();
        if self.catch && !self.function.r#async {
            convert_ret = quote! { Ok(#convert_ret) };
            exceptional_ret = quote! {
                wasm_bindgen::__rt::take_last_exception()?;
            };
        }

        let rust_name = &self.rust_name;
        let import_name = &self.shim;
        let attrs = &self.function.rust_attrs;
        let arguments = &arguments;
        let abi_arguments = &abi_arguments;
        let abi_argument_names = &abi_argument_names;

        let doc_comment = match &self.doc_comment {
            None => "",
            Some(doc_string) => doc_string,
        };
        let me = if is_method {
            quote! { &self, }
        } else {
            quote!()
        };

        // Route any errors pointing to this imported function to the identifier
        // of the function we're imported from so we at least know what function
        // is causing issues.
        //
        // Note that this is where type errors like "doesn't implement
        // FromWasmAbi" or "doesn't implement IntoWasmAbi" currently get routed.
        // I suspect that's because they show up in the signature via trait
        // projections as types of arguments, and all that needs to typecheck
        // before the body can be typechecked. Due to rust-lang/rust#60980 (and
        // probably related issues) we can't really get a precise span.
        //
        // Ideally what we want is to point errors for particular types back to
        // the specific argument/type that generated the error, but it looks
        // like rustc itself doesn't do great in that regard so let's just do
        // the best we can in the meantime.
        let extern_fn = respan(
            quote! {
                #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
                #(#attrs)*
                #[link(wasm_import_module = "__wbindgen_placeholder__")]
                extern "C" {
                    fn #import_name(#(#abi_arguments),*) -> #abi_ret;
                }

                #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
                unsafe fn #import_name(#(#abi_arguments),*) -> #abi_ret {
                    #(
                        drop(#abi_argument_names);
                    )*
                    panic!("cannot call wasm-bindgen imported functions on \
                            non-wasm targets");
                }
            },
            &self.rust_name,
        );

        let maybe_async = if self.function.r#async {
            Some(quote! {async})
        } else {
            None
        };
        let invocation = quote! {
            // This is due to `#[automatically_derived]` attribute cannot be
            // placed onto bare functions.
            #[allow(nonstandard_style)]
            #[allow(clippy::all, clippy::nursery, clippy::pedantic, clippy::restriction)]
            #(#attrs)*
            #[doc = #doc_comment]
            #vis #maybe_async fn #rust_name(#me #(#arguments),*) #ret {
                #extern_fn

                unsafe {
                    let #ret_ident = {
                        #(#arg_conversions)*
                        #import_name(#(#abi_argument_names),*)
                    };
                    #exceptional_ret
                    #convert_ret
                }
            }
        };

        if let Some(class) = class_ty {
            (quote! {
                #[automatically_derived]
                impl #class {
                    #invocation
                }
            })
            .to_tokens(tokens);
        } else {
            invocation.to_tokens(tokens);
        }

        Ok(())
    }
}

// See comment above in ast::Export for what's going on here.
struct DescribeImport<'a> {
    kind: &'a ast::ImportKind,
}

impl<'a> ToTokens for DescribeImport<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let f = match *self.kind {
            ast::ImportKind::Function(ref f) => f,
            ast::ImportKind::Static(_) => return,
            ast::ImportKind::Type(_) => return,
            ast::ImportKind::Enum(_) => return,
        };
        let argtys = f.function.arguments.iter().map(|arg| &arg.ty);
        let nargs = f.function.arguments.len() as u32;
        let inform_ret = match &f.js_ret {
            Some(ref t) => quote! { <#t as WasmDescribe>::describe(); },
            // async functions always return a JsValue, even if they say to return ()
            None if f.function.r#async => quote! { <JsValue as WasmDescribe>::describe(); },
            None => quote! { <() as WasmDescribe>::describe(); },
        };

        Descriptor {
            ident: &f.shim,
            inner: quote! {
                inform(FUNCTION);
                inform(0);
                inform(#nargs);
                #(<#argtys as WasmDescribe>::describe();)*
                #inform_ret
                #inform_ret
            },
            attrs: f.function.rust_attrs.clone(),
        }
        .to_tokens(tokens);
    }
}

impl ToTokens for ast::Enum {
    fn to_tokens(&self, into: &mut TokenStream) {
        let enum_name = &self.rust_name;
        let hole = &self.hole;
        let cast_clauses = self.variants.iter().map(|variant| {
            let variant_name = &variant.name;
            quote! {
                if js == #enum_name::#variant_name as u32 {
                    #enum_name::#variant_name
                }
            }
        });
        (quote! {
            #[automatically_derived]
            impl wasm_bindgen::convert::IntoWasmAbi for #enum_name {
                type Abi = u32;

                #[inline]
                fn into_abi(self) -> u32 {
                    self as u32
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::FromWasmAbi for #enum_name {
                type Abi = u32;

                #[inline]
                unsafe fn from_abi(js: u32) -> Self {
                    #(#cast_clauses else)* {
                        wasm_bindgen::throw_str("invalid enum value passed")
                    }
                }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::OptionFromWasmAbi for #enum_name {
                #[inline]
                fn is_none(val: &u32) -> bool { *val == #hole }
            }

            #[automatically_derived]
            impl wasm_bindgen::convert::OptionIntoWasmAbi for #enum_name {
                #[inline]
                fn none() -> Self::Abi { #hole }
            }

            #[automatically_derived]
            impl wasm_bindgen::describe::WasmDescribe for #enum_name {
                fn describe() {
                    use wasm_bindgen::describe::*;
                    inform(ENUM);
                    inform(#hole);
                }
            }
        })
        .to_tokens(into);
    }
}

impl ToTokens for ast::ImportStatic {
    fn to_tokens(&self, into: &mut TokenStream) {
        let name = &self.rust_name;
        let ty = &self.ty;
        let shim_name = &self.shim;
        let vis = &self.vis;
        (quote! {
            #[automatically_derived]
            #vis static #name: wasm_bindgen::JsStatic<#ty> = {
                fn init() -> #ty {
                    #[link(wasm_import_module = "__wbindgen_placeholder__")]
                    #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
                    extern "C" {
                        fn #shim_name() -> <#ty as wasm_bindgen::convert::FromWasmAbi>::Abi;
                    }
                    #[cfg(not(all(target_arch = "wasm32", not(target_os = "emscripten"))))]
                    unsafe fn #shim_name() -> <#ty as wasm_bindgen::convert::FromWasmAbi>::Abi {
                        panic!("cannot access imported statics on non-wasm targets")
                    }

                    unsafe {
                        <#ty as wasm_bindgen::convert::FromWasmAbi>::from_abi(#shim_name())
                    }
                }
                thread_local!(static _VAL: #ty = init(););
                wasm_bindgen::JsStatic {
                    __inner: &_VAL,
                }
            };
        })
        .to_tokens(into);

        Descriptor {
            ident: &shim_name,
            inner: quote! {
                <#ty as WasmDescribe>::describe();
            },
            attrs: vec![],
        }
        .to_tokens(into);
    }
}

/// Emits the necessary glue tokens for "descriptor", generating an appropriate
/// symbol name as well as attributes around the descriptor function itself.
struct Descriptor<'a, T> {
    ident: &'a Ident,
    inner: T,
    attrs: Vec<syn::Attribute>,
}

impl<'a, T: ToTokens> ToTokens for Descriptor<'a, T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // It's possible for the same descriptor to be emitted in two different
        // modules (aka a value imported twice in a crate, each in a separate
        // module). In this case no need to emit duplicate descriptors (which
        // leads to duplicate symbol errors), instead just emit one.
        //
        // It's up to the descriptors themselves to ensure they have unique
        // names for unique items imported, currently done via `ShortHash` and
        // hashing appropriate data into the symbol name.
        lazy_static::lazy_static! {
            static ref DESCRIPTORS_EMITTED: Mutex<HashSet<String>> = Default::default();
        }

        let ident = self.ident;

        if !DESCRIPTORS_EMITTED
            .lock()
            .unwrap()
            .insert(ident.to_string())
        {
            return;
        }

        let name = Ident::new(&format!("__wbindgen_describe_{}", ident), ident.span());
        let const_name = Ident::new(
            &format!("__wbindgen_const_describe_{}", ident),
            ident.span(),
        );
        let inner = &self.inner;
        let attrs = &self.attrs;
        (quote! {
            #[cfg(all(target_arch = "wasm32", not(target_os = "emscripten")))]
            #[automatically_derived]
            const #const_name: () = {
                #(#attrs)*
                #[no_mangle]
                #[doc(hidden)]
                pub extern "C" fn #name() {
                    use wasm_bindgen::describe::*;
                    // See definition of `link_mem_intrinsics` for what this is doing
                    wasm_bindgen::__rt::link_mem_intrinsics();
                    #inner
                }
            };
        })
        .to_tokens(tokens);
    }
}

/// Converts `span` into a stream of tokens, and attempts to ensure that `input`
/// has all the appropriate span information so errors in it point to `span`.
fn respan(input: TokenStream, span: &dyn ToTokens) -> TokenStream {
    let mut first_span = Span::call_site();
    let mut last_span = Span::call_site();
    let mut spans = TokenStream::new();
    span.to_tokens(&mut spans);

    for (i, token) in spans.into_iter().enumerate() {
        if i == 0 {
            first_span = token.span();
        }
        last_span = token.span();
    }

    let mut new_tokens = Vec::new();
    for (i, mut token) in input.into_iter().enumerate() {
        if i == 0 {
            token.set_span(first_span);
        } else {
            token.set_span(last_span);
        }
        new_tokens.push(token);
    }
    new_tokens.into_iter().collect()
}
