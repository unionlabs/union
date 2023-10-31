use crate::directory::Directory;
use crate::run::PathDependency;
use std::path::Path;

macro_rules! test_normalize {
    (
        $name:ident
        $(DIR=$dir:literal)?
        $(WORKSPACE=$workspace:literal)?
        $(INPUT=$input:literal)?
        $(TARGET=$target:literal)?
        $original:literal
        $expected:literal
    ) => {
        #[test]
        fn $name() {
            let context = super::Context {
                krate: "trybuild000",
                input_file: Path::new({ "tests/ui/error.rs" $(; $input)? }),
                source_dir: &Directory::new({ "/git/trybuild/test_suite" $(; $dir)? }),
                workspace: &Directory::new({ "/git/trybuild" $(; $workspace)? }),
                target_dir: &Directory::new({ "/git/trybuild/target" $(; $target)? }),
                path_dependencies: &[PathDependency {
                    name: String::from("diesel"),
                    normalized_path: Directory::new("/home/user/documents/rust/diesel/diesel"),
                }],
            };
            let original = $original;
            let variations = super::diagnostics(original, context);
            let preferred = variations.preferred();
            let expected = $expected;
            if preferred != expected {
                panic!("\nACTUAL: \"{}\"\nEXPECTED: \"{}\"", preferred, expected);
            }
        }
    };
}

test_normalize! {test_basic "
error: `self` parameter is only allowed in associated functions
  --> /git/trybuild/test_suite/tests/ui/error.rs:11:23
   |
11 | async fn bad_endpoint(self) -> Result<HttpResponseOkObject<()>, HttpError> {
   |                       ^^^^ not semantically valid as function parameter

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0401`.
error: could not compile `trybuild-tests`.

To learn more, run the command again with --verbose.
" "
error: `self` parameter is only allowed in associated functions
  --> tests/ui/error.rs:11:23
   |
11 | async fn bad_endpoint(self) -> Result<HttpResponseOkObject<()>, HttpError> {
   |                       ^^^^ not semantically valid as function parameter
"}

test_normalize! {test_dir_backslash "
error[E0277]: the trait bound `QueryParams: serde::de::Deserialize<'de>` is not satisfied
   --> \\git\\trybuild\\test_suite\\tests\\ui\\error.rs:22:61
" "
error[E0277]: the trait bound `QueryParams: serde::de::Deserialize<'de>` is not satisfied
   --> tests/ui/error.rs:22:61
"}

test_normalize! {test_rust_lib
    INPUT="tests/ui/not-repeatable.rs"
"
error[E0599]: no method named `quote_into_iter` found for struct `std::net::Ipv4Addr` in the current scope
  --> /git/trybuild/test_suite/tests/ui/not-repeatable.rs:6:13
   |
6  |     let _ = quote! { #(#ip)* };
   |             ^^^^^^^^^^^^^^^^^^ method not found in `std::net::Ipv4Addr`
   |
  ::: /rustlib/src/rust/src/libstd/net/ip.rs:83:1
  ::: /rustlib/src/rust/library/std/src/net/ip.rs:83:1
   |
83 | pub struct Ipv4Addr {
   | -------------------
   | |
   | doesn't satisfy `std::net::Ipv4Addr: quote::to_tokens::ToTokens`
" "
error[E0599]: no method named `quote_into_iter` found for struct `std::net::Ipv4Addr` in the current scope
  --> tests/ui/not-repeatable.rs:6:13
   |
6  |     let _ = quote! { #(#ip)* };
   |             ^^^^^^^^^^^^^^^^^^ method not found in `std::net::Ipv4Addr`
   |
  ::: $RUST/src/libstd/net/ip.rs
  ::: $RUST/std/src/net/ip.rs
   |
   | pub struct Ipv4Addr {
   | -------------------
   | |
   | doesn't satisfy `std::net::Ipv4Addr: quote::to_tokens::ToTokens`
"}

test_normalize! {test_type_dir_backslash
    INPUT="tests/ui/compile-fail-3.rs"
"
error[E0277]: `*mut _` cannot be shared between threads safely
   --> /git/trybuild/test_suite/tests/ui/compile-fail-3.rs:7:5
    |
7   |     thread::spawn(|| {
    |     ^^^^^^^^^^^^^ `*mut _` cannot be shared between threads safely
    |
    = help: the trait `std::marker::Sync` is not implemented for `*mut _`
    = note: required because of the requirements on the impl of `std::marker::Send` for `&*mut _`
    = note: required because it appears within the type `[closure@/git/trybuild/test_suite/ui/compile-fail-3.rs:7:19: 9:6 x:&*mut _]`
" "
error[E0277]: `*mut _` cannot be shared between threads safely
   --> tests/ui/compile-fail-3.rs:7:5
    |
7   |     thread::spawn(|| {
    |     ^^^^^^^^^^^^^ `*mut _` cannot be shared between threads safely
    |
    = help: the trait `std::marker::Sync` is not implemented for `*mut _`
    = note: required because of the requirements on the impl of `std::marker::Send` for `&*mut _`
    = note: required because it appears within the type `[closure@$DIR/ui/compile-fail-3.rs:7:19: 9:6 x:&*mut _]`
"}

test_normalize! {test_strip_path_dependencies "
error[E0277]: the trait bound `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>: diesel::query_builder::IntoUpdateTarget` is not satisfied
  --> $DIR/update_requires_valid_where_clause.rs:21:12
   |
21 |     update(users::table.filter(posts::id.eq(1)));
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `diesel::query_builder::IntoUpdateTarget` is not implemented for `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>`
   |
  ::: /home/user/documents/rust/diesel/diesel/src/query_builder/functions.rs:78:18
   |
78 | pub fn update<T: IntoUpdateTarget>(source: T) -> UpdateStatement<T::Table, T::WhereClause> {
   |                  ---------------- required by this bound in `diesel::update`
   |
   = help: the following implementations were found:
             <diesel::query_builder::SelectStatement<F, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, W> as diesel::query_builder::IntoUpdateTarget>
" "
error[E0277]: the trait bound `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>: diesel::query_builder::IntoUpdateTarget` is not satisfied
  --> $DIR/update_requires_valid_where_clause.rs:21:12
   |
21 |     update(users::table.filter(posts::id.eq(1)));
   |            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `diesel::query_builder::IntoUpdateTarget` is not implemented for `diesel::query_builder::SelectStatement<users::table, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, diesel::query_builder::where_clause::WhereClause<diesel::expression::grouped::Grouped<diesel::expression::operators::Eq<posts::columns::id, diesel::expression::bound::Bound<diesel::sql_types::Integer, i32>>>>>`
   |
  ::: $DIESEL/src/query_builder/functions.rs
   |
   | pub fn update<T: IntoUpdateTarget>(source: T) -> UpdateStatement<T::Table, T::WhereClause> {
   |                  ---------------- required by this bound in `diesel::update`
   |
   = help: the following implementations were found:
             <diesel::query_builder::SelectStatement<F, diesel::query_builder::select_clause::DefaultSelectClause, diesel::query_builder::distinct_clause::NoDistinctClause, W> as diesel::query_builder::IntoUpdateTarget>
"}

test_normalize! {test_cargo_registry "
error[E0277]: the trait bound `Thread: serde::de::Deserialize<'_>` is not satisfied
    --> src/main.rs:2:36
     |
2    |     let _ = serde_json::from_str::<std::thread::Thread>(\"???\");
     |                                    ^^^^^^^^^^^^^^^^^^^ the trait `serde::de::Deserialize<'_>` is not implemented for `Thread`
     |
    ::: /home/ferris/.cargo/registry/src/github.com-1ecc6299db9ec823/serde_json-1.0.64/src/de.rs:2584:8
     |
2584 |     T: de::Deserialize<'a>,
     |        ------------------- required by this bound in `serde_json::from_str`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `testing` due to previous error
" "
error[E0277]: the trait bound `Thread: serde::de::Deserialize<'_>` is not satisfied
    --> src/main.rs:2:36
     |
2    |     let _ = serde_json::from_str::<std::thread::Thread>(\"???\");
     |                                    ^^^^^^^^^^^^^^^^^^^ the trait `serde::de::Deserialize<'_>` is not implemented for `Thread`
     |
    ::: $CARGO/serde_json-1.0.64/src/de.rs
     |
     |     T: de::Deserialize<'a>,
     |        ------------------- required by this bound in `serde_json::from_str`
"}

test_normalize! {test_traits_must_be_implemented "
error[E0599]: the method `anyhow_kind` exists for reference `&Error`, but its trait bounds were not satisfied
   --> src/main.rs:7:13
    |
4   | struct Error;
    | -------------
    | |
    | doesn't satisfy `Error: Into<anyhow::Error>`
    | doesn't satisfy `Error: anyhow::private::kind::TraitKind`
    | doesn't satisfy `Error: std::fmt::Display`
...
7   |     let _ = anyhow!(Error);
    |             ^^^^^^^^^^^^^^ method cannot be called on `&Error` due to unsatisfied trait bounds
    |
    = note: the following trait bounds were not satisfied:
            `Error: Into<anyhow::Error>`
            which is required by `Error: anyhow::private::kind::TraitKind`
            `Error: std::fmt::Display`
            which is required by `&Error: anyhow::private::kind::AdhocKind`
            `&Error: Into<anyhow::Error>`
            which is required by `&Error: anyhow::private::kind::TraitKind`
note: the following traits must be implemented
   --> /rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/convert/mod.rs:274:1
    |
274 | / pub trait Into<T>: Sized {
275 | |     /// Performs the conversion.
276 | |     #[stable(feature = \"rust1\", since = \"1.0.0\")]
277 | |     fn into(self) -> T;
278 | | }
    | |_^
    |
   ::: /rustup/toolchains/nightly-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/fmt/mod.rs:715:1
    |
715 | / pub trait Display {
716 | |     /// Formats the value using the given formatter.
717 | |     ///
718 | |     /// # Examples
...   |
738 | |     fn fmt(&self, f: &mut Formatter<'_>) -> Result;
739 | | }
    | |_^
    = note: this error originates in the macro `anyhow` (in Nightly builds, run with -Z macro-backtrace for more info)
" "
error[E0599]: the method `anyhow_kind` exists for reference `&Error`, but its trait bounds were not satisfied
   --> src/main.rs:7:13
    |
4   | struct Error;
    | -------------
    | |
    | doesn't satisfy `Error: Into<anyhow::Error>`
    | doesn't satisfy `Error: anyhow::private::kind::TraitKind`
    | doesn't satisfy `Error: std::fmt::Display`
...
7   |     let _ = anyhow!(Error);
    |             ^^^^^^^^^^^^^^ method cannot be called on `&Error` due to unsatisfied trait bounds
    |
    = note: the following trait bounds were not satisfied:
            `Error: Into<anyhow::Error>`
            which is required by `Error: anyhow::private::kind::TraitKind`
            `Error: std::fmt::Display`
            which is required by `&Error: anyhow::private::kind::AdhocKind`
            `&Error: Into<anyhow::Error>`
            which is required by `&Error: anyhow::private::kind::TraitKind`
note: the following traits must be implemented
   --> $RUST/core/src/convert/mod.rs
    |
    | / pub trait Into<T>: Sized {
    | |     /// Performs the conversion.
    | |     #[stable(feature = \"rust1\", since = \"1.0.0\")]
    | |     fn into(self) -> T;
    | | }
    | |_^
    |
   ::: $RUST/core/src/fmt/mod.rs
    |
    | / pub trait Display {
    | |     /// Formats the value using the given formatter.
    | |     ///
    | |     /// # Examples
...   |
    | |     fn fmt(&self, f: &mut Formatter<'_>) -> Result;
    | | }
    | |_^
    = note: this error originates in the macro `anyhow` (in Nightly builds, run with -Z macro-backtrace for more info)
"}

test_normalize! {test_pyo3_url
    DIR="/pyo3"
    WORKSPACE="/pyo3"
"
error: `async fn` is not yet supported for Python functions.

Additional crates such as `pyo3-asyncio` can be used to integrate async Rust and Python. For more information, see https://github.com/PyO3/pyo3/issues/1632
  --> tests/ui/invalid_pyfunctions.rs:10:1
   |
10 | async fn async_function() {}
   | ^^^^^
" "
error: `async fn` is not yet supported for Python functions.

Additional crates such as `pyo3-asyncio` can be used to integrate async Rust and Python. For more information, see https://github.com/PyO3/pyo3/issues/1632
  --> tests/ui/invalid_pyfunctions.rs:10:1
   |
10 | async fn async_function() {}
   | ^^^^^
"}

test_normalize! {test_dropshot_required_by
    DIR="/git/dropshot/dropshot"
    WORKSPACE="/git/dropshot"
    INPUT="tests/fail/bad_endpoint4.rs"
"
error[E0277]: the trait bound `QueryParams: schemars::JsonSchema` is not satisfied
   --> /git/dropshot/dropshot/tests/fail/bad_endpoint4.rs:24:14
    |
24  |     _params: Query<QueryParams>,
    |              ^^^^^^^^^^^^^^^^^^ the trait `schemars::JsonSchema` is not implemented for `QueryParams`
    |
note: required by a bound in `dropshot::Query`
   --> /git/dropshot/dropshot/src/handler.rs:547:48
    |
547 | pub struct Query<QueryType: DeserializeOwned + JsonSchema + Send + Sync> {
    |                                                ^^^^^^^^^^ required by this bound in `dropshot::Query`
"
// TODO: it would be nice to also unindent the column of `|` by one column.
// https://github.com/dtolnay/trybuild/issues/86
"
error[E0277]: the trait bound `QueryParams: schemars::JsonSchema` is not satisfied
   --> tests/fail/bad_endpoint4.rs:24:14
    |
24  |     _params: Query<QueryParams>,
    |              ^^^^^^^^^^^^^^^^^^ the trait `schemars::JsonSchema` is not implemented for `QueryParams`
    |
note: required by a bound in `dropshot::Query`
   --> src/handler.rs
    |
    | pub struct Query<QueryType: DeserializeOwned + JsonSchema + Send + Sync> {
    |                                                ^^^^^^^^^^ required by this bound in `dropshot::Query`
"}

test_normalize! {test_uniffi_out_dir
    DIR="/git/uniffi-rs/fixtures/uitests"
    WORKSPACE="/git/uniffi-rs"
    TARGET="/git/uniffi-rs/target"
"
error[E0277]: the trait bound `Arc<Counter>: FfiConverter` is not satisfied
   --> /git/uniffi-rs/target/debug/build/uniffi_uitests-1a51d46aecb559a7/out/counter.uniffi.rs:160:19
    |
160 |             match <std::sync::Arc<Counter> as uniffi::FfiConverter>::try_lift(ptr) {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `FfiConverter` is not implemented for `Arc<Counter>`
    |
    = help: the following implementations were found:
              <Arc<T> as FfiConverter>
    = note: required by `try_lift`
" "
error[E0277]: the trait bound `Arc<Counter>: FfiConverter` is not satisfied
   --> $OUT_DIR[uniffi_uitests]/counter.uniffi.rs
    |
    |             match <std::sync::Arc<Counter> as uniffi::FfiConverter>::try_lift(ptr) {
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `FfiConverter` is not implemented for `Arc<Counter>`
    |
    = help: the following implementations were found:
              <Arc<T> as FfiConverter>
    = note: required by `try_lift`
"}

test_normalize! {test_proc_macro_panic
    DIR="D:\\repro"
    INPUT="tests\\ui\\nonzero_fail.rs"
"
error[E0080]: evaluation of constant value failed
 --> D:\\repro\\tests\\ui\\nonzero_fail.rs:7:10
  |
7 | #[derive(NonZeroRepr)]
  |          ^^^^^^^^^^^ the evaluated program panicked at 'expected non-zero discriminant expression', D:\\repro\\tests\\ui\\nonzero_fail.rs:7:10
" "
error[E0080]: evaluation of constant value failed
 --> tests/ui/nonzero_fail.rs:7:10
  |
7 | #[derive(NonZeroRepr)]
  |          ^^^^^^^^^^^ the evaluated program panicked at 'expected non-zero discriminant expression', $DIR/tests/ui/nonzero_fail.rs:7:10
"}
