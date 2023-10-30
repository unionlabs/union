use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::Parser;

// syn::AttributeArgs does not implement syn::Parse
type AttributeArgs = syn::punctuated::Punctuated<syn::NestedMeta, syn::Token![,]>;

#[derive(Clone, Copy, PartialEq)]
enum RuntimeFlavor {
    CurrentThread,
    Threaded,
}

impl RuntimeFlavor {
    fn from_str(s: &str) -> Result<RuntimeFlavor, String> {
        match s {
            "current_thread" => Ok(RuntimeFlavor::CurrentThread),
            "multi_thread" => Ok(RuntimeFlavor::Threaded),
            "single_thread" => Err("The single threaded runtime flavor is called `current_thread`.".to_string()),
            "basic_scheduler" => Err("The `basic_scheduler` runtime flavor has been renamed to `current_thread`.".to_string()),
            "threaded_scheduler" => Err("The `threaded_scheduler` runtime flavor has been renamed to `multi_thread`.".to_string()),
            _ => Err(format!("No such runtime flavor `{}`. The runtime flavors are `current_thread` and `multi_thread`.", s)),
        }
    }
}

struct FinalConfig {
    flavor: RuntimeFlavor,
    worker_threads: Option<usize>,
    start_paused: Option<bool>,
    crate_name: Option<String>,
}

/// Config used in case of the attribute not being able to build a valid config
const DEFAULT_ERROR_CONFIG: FinalConfig = FinalConfig {
    flavor: RuntimeFlavor::CurrentThread,
    worker_threads: None,
    start_paused: None,
    crate_name: None,
};

struct Configuration {
    rt_multi_thread_available: bool,
    default_flavor: RuntimeFlavor,
    flavor: Option<RuntimeFlavor>,
    worker_threads: Option<(usize, Span)>,
    start_paused: Option<(bool, Span)>,
    is_test: bool,
    crate_name: Option<String>,
}

impl Configuration {
    fn new(is_test: bool, rt_multi_thread: bool) -> Self {
        Configuration {
            rt_multi_thread_available: rt_multi_thread,
            default_flavor: match is_test {
                true => RuntimeFlavor::CurrentThread,
                false => RuntimeFlavor::Threaded,
            },
            flavor: None,
            worker_threads: None,
            start_paused: None,
            is_test,
            crate_name: None,
        }
    }

    fn set_flavor(&mut self, runtime: syn::Lit, span: Span) -> Result<(), syn::Error> {
        if self.flavor.is_some() {
            return Err(syn::Error::new(span, "`flavor` set multiple times."));
        }

        let runtime_str = parse_string(runtime, span, "flavor")?;
        let runtime =
            RuntimeFlavor::from_str(&runtime_str).map_err(|err| syn::Error::new(span, err))?;
        self.flavor = Some(runtime);
        Ok(())
    }

    fn set_worker_threads(
        &mut self,
        worker_threads: syn::Lit,
        span: Span,
    ) -> Result<(), syn::Error> {
        if self.worker_threads.is_some() {
            return Err(syn::Error::new(
                span,
                "`worker_threads` set multiple times.",
            ));
        }

        let worker_threads = parse_int(worker_threads, span, "worker_threads")?;
        if worker_threads == 0 {
            return Err(syn::Error::new(span, "`worker_threads` may not be 0."));
        }
        self.worker_threads = Some((worker_threads, span));
        Ok(())
    }

    fn set_start_paused(&mut self, start_paused: syn::Lit, span: Span) -> Result<(), syn::Error> {
        if self.start_paused.is_some() {
            return Err(syn::Error::new(span, "`start_paused` set multiple times."));
        }

        let start_paused = parse_bool(start_paused, span, "start_paused")?;
        self.start_paused = Some((start_paused, span));
        Ok(())
    }

    fn set_crate_name(&mut self, name: syn::Lit, span: Span) -> Result<(), syn::Error> {
        if self.crate_name.is_some() {
            return Err(syn::Error::new(span, "`crate` set multiple times."));
        }
        let name_ident = parse_ident(name, span, "crate")?;
        self.crate_name = Some(name_ident.to_string());
        Ok(())
    }

    fn macro_name(&self) -> &'static str {
        if self.is_test {
            "tokio::test"
        } else {
            "tokio::main"
        }
    }

    fn build(&self) -> Result<FinalConfig, syn::Error> {
        let flavor = self.flavor.unwrap_or(self.default_flavor);
        use RuntimeFlavor::*;

        let worker_threads = match (flavor, self.worker_threads) {
            (CurrentThread, Some((_, worker_threads_span))) => {
                let msg = format!(
                    "The `worker_threads` option requires the `multi_thread` runtime flavor. Use `#[{}(flavor = \"multi_thread\")]`",
                    self.macro_name(),
                );
                return Err(syn::Error::new(worker_threads_span, msg));
            }
            (CurrentThread, None) => None,
            (Threaded, worker_threads) if self.rt_multi_thread_available => {
                worker_threads.map(|(val, _span)| val)
            }
            (Threaded, _) => {
                let msg = if self.flavor.is_none() {
                    "The default runtime flavor is `multi_thread`, but the `rt-multi-thread` feature is disabled."
                } else {
                    "The runtime flavor `multi_thread` requires the `rt-multi-thread` feature."
                };
                return Err(syn::Error::new(Span::call_site(), msg));
            }
        };

        let start_paused = match (flavor, self.start_paused) {
            (Threaded, Some((_, start_paused_span))) => {
                let msg = format!(
                    "The `start_paused` option requires the `current_thread` runtime flavor. Use `#[{}(flavor = \"current_thread\")]`",
                    self.macro_name(),
                );
                return Err(syn::Error::new(start_paused_span, msg));
            }
            (CurrentThread, Some((start_paused, _))) => Some(start_paused),
            (_, None) => None,
        };

        Ok(FinalConfig {
            crate_name: self.crate_name.clone(),
            flavor,
            worker_threads,
            start_paused,
        })
    }
}

fn parse_int(int: syn::Lit, span: Span, field: &str) -> Result<usize, syn::Error> {
    match int {
        syn::Lit::Int(lit) => match lit.base10_parse::<usize>() {
            Ok(value) => Ok(value),
            Err(e) => Err(syn::Error::new(
                span,
                format!("Failed to parse value of `{}` as integer: {}", field, e),
            )),
        },
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{}` as integer.", field),
        )),
    }
}

fn parse_string(int: syn::Lit, span: Span, field: &str) -> Result<String, syn::Error> {
    match int {
        syn::Lit::Str(s) => Ok(s.value()),
        syn::Lit::Verbatim(s) => Ok(s.to_string()),
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{}` as string.", field),
        )),
    }
}

fn parse_ident(lit: syn::Lit, span: Span, field: &str) -> Result<Ident, syn::Error> {
    match lit {
        syn::Lit::Str(s) => {
            let err = syn::Error::new(
                span,
                format!(
                    "Failed to parse value of `{}` as ident: \"{}\"",
                    field,
                    s.value()
                ),
            );
            let path = s.parse::<syn::Path>().map_err(|_| err.clone())?;
            path.get_ident().cloned().ok_or(err)
        }
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{}` as ident.", field),
        )),
    }
}

fn parse_bool(bool: syn::Lit, span: Span, field: &str) -> Result<bool, syn::Error> {
    match bool {
        syn::Lit::Bool(b) => Ok(b.value),
        _ => Err(syn::Error::new(
            span,
            format!("Failed to parse value of `{}` as bool.", field),
        )),
    }
}

fn build_config(
    input: syn::ItemFn,
    args: AttributeArgs,
    is_test: bool,
    rt_multi_thread: bool,
) -> Result<FinalConfig, syn::Error> {
    if input.sig.asyncness.is_none() {
        let msg = "the `async` keyword is missing from the function declaration";
        return Err(syn::Error::new_spanned(input.sig.fn_token, msg));
    }

    let mut config = Configuration::new(is_test, rt_multi_thread);
    let macro_name = config.macro_name();

    for arg in args {
        match arg {
            syn::NestedMeta::Meta(syn::Meta::NameValue(namevalue)) => {
                let ident = namevalue
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&namevalue, "Must have specified ident")
                    })?
                    .to_string()
                    .to_lowercase();
                match ident.as_str() {
                    "worker_threads" => {
                        config.set_worker_threads(
                            namevalue.lit.clone(),
                            syn::spanned::Spanned::span(&namevalue.lit),
                        )?;
                    }
                    "flavor" => {
                        config.set_flavor(
                            namevalue.lit.clone(),
                            syn::spanned::Spanned::span(&namevalue.lit),
                        )?;
                    }
                    "start_paused" => {
                        config.set_start_paused(
                            namevalue.lit.clone(),
                            syn::spanned::Spanned::span(&namevalue.lit),
                        )?;
                    }
                    "core_threads" => {
                        let msg = "Attribute `core_threads` is renamed to `worker_threads`";
                        return Err(syn::Error::new_spanned(namevalue, msg));
                    }
                    "crate" => {
                        config.set_crate_name(
                            namevalue.lit.clone(),
                            syn::spanned::Spanned::span(&namevalue.lit),
                        )?;
                    }
                    name => {
                        let msg = format!(
                            "Unknown attribute {} is specified; expected one of: `flavor`, `worker_threads`, `start_paused`, `crate`",
                            name,
                        );
                        return Err(syn::Error::new_spanned(namevalue, msg));
                    }
                }
            }
            syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                let name = path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&path, "Must have specified ident"))?
                    .to_string()
                    .to_lowercase();
                let msg = match name.as_str() {
                    "threaded_scheduler" | "multi_thread" => {
                        format!(
                            "Set the runtime flavor with #[{}(flavor = \"multi_thread\")].",
                            macro_name
                        )
                    }
                    "basic_scheduler" | "current_thread" | "single_threaded" => {
                        format!(
                            "Set the runtime flavor with #[{}(flavor = \"current_thread\")].",
                            macro_name
                        )
                    }
                    "flavor" | "worker_threads" | "start_paused" => {
                        format!("The `{}` attribute requires an argument.", name)
                    }
                    name => {
                        format!("Unknown attribute {} is specified; expected one of: `flavor`, `worker_threads`, `start_paused`, `crate`", name)
                    }
                };
                return Err(syn::Error::new_spanned(path, msg));
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "Unknown attribute inside the macro",
                ));
            }
        }
    }

    config.build()
}

fn parse_knobs(mut input: syn::ItemFn, is_test: bool, config: FinalConfig) -> TokenStream {
    input.sig.asyncness = None;

    // If type mismatch occurs, the current rustc points to the last statement.
    let (last_stmt_start_span, last_stmt_end_span) = {
        let mut last_stmt = input
            .block
            .stmts
            .last()
            .map(ToTokens::into_token_stream)
            .unwrap_or_default()
            .into_iter();
        // `Span` on stable Rust has a limitation that only points to the first
        // token, not the whole tokens. We can work around this limitation by
        // using the first/last span of the tokens like
        // `syn::Error::new_spanned` does.
        let start = last_stmt.next().map_or_else(Span::call_site, |t| t.span());
        let end = last_stmt.last().map_or(start, |t| t.span());
        (start, end)
    };

    let crate_name = config.crate_name.as_deref().unwrap_or("tokio");

    let crate_ident = Ident::new(crate_name, last_stmt_start_span);

    let mut rt = match config.flavor {
        RuntimeFlavor::CurrentThread => quote_spanned! {last_stmt_start_span=>
            #crate_ident::runtime::Builder::new_current_thread()
        },
        RuntimeFlavor::Threaded => quote_spanned! {last_stmt_start_span=>
            #crate_ident::runtime::Builder::new_multi_thread()
        },
    };
    if let Some(v) = config.worker_threads {
        rt = quote! { #rt.worker_threads(#v) };
    }
    if let Some(v) = config.start_paused {
        rt = quote! { #rt.start_paused(#v) };
    }

    let header = if is_test {
        quote! {
            #[::core::prelude::v1::test]
        }
    } else {
        quote! {}
    };

    let body = &input.block;
    let brace_token = input.block.brace_token;
    input.block = syn::parse2(quote_spanned! {last_stmt_end_span=>
        {
            let body = async #body;
            #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
            {
                return #rt
                    .enable_all()
                    .build()
                    .expect("Failed building the Runtime")
                    .block_on(body);
            }
        }
    })
    .expect("Parsing failure");
    input.block.brace_token = brace_token;

    let result = quote! {
        #header
        #input
    };

    result.into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}

#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub(crate) fn main(args: TokenStream, item: TokenStream, rt_multi_thread: bool) -> TokenStream {
    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };

    let config = if input.sig.ident == "main" && !input.sig.inputs.is_empty() {
        let msg = "the main function cannot accept arguments";
        Err(syn::Error::new_spanned(&input.sig.ident, msg))
    } else {
        AttributeArgs::parse_terminated
            .parse(args)
            .and_then(|args| build_config(input.clone(), args, false, rt_multi_thread))
    };

    match config {
        Ok(config) => parse_knobs(input, false, config),
        Err(e) => token_stream_with_error(parse_knobs(input, false, DEFAULT_ERROR_CONFIG), e),
    }
}

pub(crate) fn test(args: TokenStream, item: TokenStream, rt_multi_thread: bool) -> TokenStream {
    // If any of the steps for this macro fail, we still want to expand to an item that is as close
    // to the expected output as possible. This helps out IDEs such that completions and other
    // related features keep working.
    let input: syn::ItemFn = match syn::parse(item.clone()) {
        Ok(it) => it,
        Err(e) => return token_stream_with_error(item, e),
    };
    let config = if let Some(attr) = input.attrs.iter().find(|attr| attr.path.is_ident("test")) {
        let msg = "second test attribute is supplied";
        Err(syn::Error::new_spanned(&attr, msg))
    } else {
        AttributeArgs::parse_terminated
            .parse(args)
            .and_then(|args| build_config(input.clone(), args, true, rt_multi_thread))
    };

    match config {
        Ok(config) => parse_knobs(input, true, config),
        Err(e) => token_stream_with_error(parse_knobs(input, true, DEFAULT_ERROR_CONFIG), e),
    }
}
