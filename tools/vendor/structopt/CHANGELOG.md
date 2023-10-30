# v0.3.25 (2021-10-18)

* Fix duplication of aliases in subcommands [#504](https://github.com/TeXitoi/structopt/pull/504)

# v0.3.25 (2021-10-18)

* No changes

# v0.3.23 (2021-08-30)

* Update minimal rust version to 1.46 because of bitflags 1.3
* Fixed [a bug that occurs when the type of `map` becomes ambiguous](https://github.com/TeXitoi/structopt/issues/490).
* Add support for [skip for enum variant subcommands](https://github.com/TeXitoi/structopt/issues/493)

# v0.3.22 (2021-07-04)

* Add support for [generics in derive](https://github.com/TeXitoi/structopt/issues/128)

# v0.3.21 (2020-11-30)

* Fixed [another breakage](https://github.com/TeXitoi/structopt/issues/447)
  when the struct is placed inside a `macro_rules!` macro.

# v0.3.20 (2020-10-12)

* Fixed [a breakage](https://github.com/TeXitoi/structopt/issues/439)
  when the struct is placed inside a `macro_rules!` macro.

# v0.3.19 (2020-10-08)

* Added [StructOpt::from_args_safe](https://docs.rs/structopt/0.3/structopt/trait.StructOpt.html#tymethod.from_args_safe) as a shortcut for `StructOpt::from_iter_safe(std::env::args_os())`.
* Some links in documentation have been corrected.

# v0.3.18 (2020-09-23)

* Unsafe code [has been forbidden](https://github.com/TeXitoi/structopt/issues/432). This makes
  [`cargo geiger`](https://github.com/rust-secure-code/cargo-geiger) list structopt as "safe".
  Maybe it will help somebody trying to locate a bug in their dependency tree.

# v0.3.17 (2020-08-25)

* Fixed [a breakage](https://github.com/TeXitoi/structopt/issues/424) with resent rustc versions
  due to `quote_spanned` misuse.

# v0.3.16 (2020-08-05)

* Added [the new example](https://github.com/TeXitoi/structopt/blob/master/examples/required_if.rs).
* Allow `#[structopt(flatten)]` fields to have doc comments. The comments are ignored.
* The `paw` crate is now being reexported when `paw` feature is enabled,
  see [`#407`](https://github.com/TeXitoi/structopt/issues/407).

# v0.3.15 (2020-06-16)

* Minor documentation improvements.
* Fixed [a latent bug](https://github.com/TeXitoi/structopt/pull/398),
  courtesy of [@Aaron1011](https://github.com/Aaron1011).

# v0.3.14 (2020-04-22)

* Minor documentation improvements.

# v0.3.13 (2020-04-9)

* Bump `proc-macro-error` to `1.0`.

# v0.3.12 (2020-03-18)

* Fixed [bug in `external_subcommand`](https://github.com/TeXitoi/structopt/issues/359).

# v0.3.11 (2020-03-01)

* `syn`'s "full" feature is now explicitly enabled. It must have been, but hasn't.

# v0.3.10 (2020-03-01) - YANKED

* Fixed the breakage due to a required `syn` feature was not enabled.

# v0.3.9 (2020-02-01) - YANKED

* `clippy` warnings triggered by generated code shall not annoy you anymore!
  Except for those from `clippy::correctness`, these lints are useful even
  for auto generated code.
* Improved error messages.

# v0.3.8 (2020-1-19) - YANKED

* You don't have to apply `#[no_version]` to every `enum` variant anymore.
  Just annotate the `enum` and the setting will be propagated down
  ([#242](https://github.com/TeXitoi/structopt/issues/242)).
* [Auto-default](https://docs.rs/structopt/0.3/structopt/#default-values).
* [External subcommands](https://docs.rs/structopt/0.3/structopt/#external-subcommands).
* [Flattening subcommands](https://docs.rs/structopt/0.3.8/structopt/#flattening-subcommands).

# v0.3.7 (2019-12-28)

Nothing's new. Just re-release of `v0.3.6` due to
[the mess with versioning](https://github.com/TeXitoi/structopt/issues/315#issuecomment-568502792).

You may notice that `structopt-derive` was bumped to `v0.4.0`, that's OK, it's not a breaking change.
`structopt` will pull the right version in on its on.

# v0.3.6 (2019-12-22) - YANKED

This is unusually big patch release. It contains a number of bugfixes and
new features, some of them may theoretically be considered breaking. We did our best
to avoid any problems on user's side but, if it wasn't good enough, please
[file an issue ASAP](https://github.com/TeXitoi/structopt/issues).

## Bugfixes

* `structopt` used to treat `::path::to::type::Vec<T>` as `Vec<T>`
  special type. [This was considered erroneous](https://github.com/TeXitoi/structopt/pull/287).
  (same for `Option<T>` and `bool`). Now only exact `Vec<T>` match is a special type.

* `#[structopt(version = expr)]` where `expr` is not a string literal used to get
  overridden by auto generated `.version()` call,
  [incorrectly](https://github.com/TeXitoi/structopt/issues/283). Now it doesn't.

* Fixed bug with top-level `App::*` calls on multiple `struct`s, see
  [#289](https://github.com/TeXitoi/structopt/issues/265).

* Positional `bool` args with no explicit `#[structopt(parse(...))]` annotation are
  now prohibited. This couldn't work well anyway, see
  [this example](https://github.com/TeXitoi/structopt/blob/master/examples/true_or_false.rs)
  for details.

* Now we've instituted strict priority between doc comments, about, help, and the like.
  See [the documentation](https://docs.rs/structopt/0.3/structopt/#help-messages).

  **HUGE THANKS to [`@ssokolow`](https://github.com/ssokolow)** for tidying up our documentation,
  teaching me English and explaining why our doc used to suck. I promise I'll make the rest
  of the doc up to your standards... sometime later!

## New features

* Implement `StructOpt` for `Box<impl StructOpt>` so from now on you can use `Box<T>`
  with `flatten` and `subcommand` ([#304](https://github.com/TeXitoi/structopt/issues/304)).

  ```rust
  enum Command {
      #[structopt(name = "version")]
      PrintVersion,

      #[structopt(name = "second")]
      DoSomething {
          #[structopt(flatten)]
          config: Box<DoSomethingConfig>,
      },

      #[structopt(name = "first")]
      DoSomethingElse {
          #[structopt(flatten)]
          config: Box<DoSomethingElseConfig>,
      }
  }
  ```

* Introduced `#[structopt(verbatim_doc_comment)]` attribute that keeps line breaks in
  doc comments, see
  [the documentation](https://docs.rs/structopt/0.3/structopt/#doc-comment-preprocessing-and-structoptverbatim_doc_comment).

* Introduced `#[structopt(rename_all_env)]` and `#[structopt(env)]` magical methods
  so you can derive env var's name from field's name. See
  [the documentation](https://docs.rs/structopt/0.3/structopt/#auto-deriving-environment-variables).

## Improvements

* Now we have nice README for our examples,
  [check it out](https://github.com/TeXitoi/structopt/tree/master/examples)!

* Some error messages were improved and clarified, thanks for all people involved!


# v0.3.5 (2019-11-22)

* `try_from_str` functions are now called with a `&str` instead of a `&String` ([#282](https://github.com/TeXitoi/structopt/pull/282))

# v0.3.4 (2019-11-08)

* `rename_all` does not apply to fields that were annotated with explicit
  `short/long/name = "..."` anymore ([#265](https://github.com/TeXitoi/structopt/issues/265))
* Now raw idents are handled correctly ([#269](https://github.com/TeXitoi/structopt/issues/269))
* Some documentation improvements and clarification.

# v0.3.3 (2019-10-10)

* Add `from_flag` custom parser to create flags from non-bool types.
  Fixes [#185](https://github.com/TeXitoi/structopt/issues/185)

# v0.3.2 (2019-09-18)

* `structopt` does not replace `:` with `, ` inside "author" strings while inside `<...>`.
  Fixes [#156](https://github.com/TeXitoi/structopt/issues/156)
* Introduced [`#[structopt(skip = expr)]` syntax](https://docs.rs/structopt/0.3.2/structopt/#skipping-fields).

# v0.3.1 (2019-09-06)

* Fix error messages ([#241](https://github.com/TeXitoi/structopt/issues/241))
* Fix "`skip` plus long doc comment" bug ([#245](https://github.com/TeXitoi/structopt/issues/245))
* Now `structopt` emits dummy `StructOpt` implementation along with an error. It suppresses
  meaningless errors like `from_args method is not found for Opt`
* `.version()` not get generated if `CARGO_PKG_VERSION` is not set anymore.

# v0.3.0 (2019-08-30)

## Breaking changes

### Bump minimum rustc version to 1.36 by [@TeXitoi](https://github.com/TeXitoi)
Now `rustc` 1.36 is the minimum compiler version supported by `structopt`,
it likely won't work with older compilers.

### Remove "nightly" feature
Once upon a time this feature had been used to enable some of improvements
in `proc-macro2` crate that were available only on nightly. Nowadays this feature doesn't
mean anything so it's now removed.

### Support optional vectors of arguments for distinguishing between `-o 1 2`, `-o` and no option provided at all by [@sphynx](https://github.com/sphynx) ([#180](https://github.com/TeXitoi/structopt/issues/188)).

```rust
#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  fruit: Option<Vec<String>>,
}

fn main() {
  assert_eq!(Opt::from_args(&["test"]), None);
  assert_eq!(Opt::from_args(&["test", "--fruit"]), Some(vec![]));
  assert_eq!(Opt::from_args(&["test", "--fruit=apple orange"]), Some(vec!["apple", "orange"]));
}
```

If you need to fall back to the old behavior you can use a type alias:
```rust
type Something = Vec<String>;

#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  fruit: Option<Something>,
}
```

### Change default case from 'Verbatim' into 'Kebab' by [@0ndorio](https://github.com/0ndorio) ([#202](https://github.com/TeXitoi/structopt/issues/202)).
`structopt` 0.3 uses field renaming to deduce a name for long options and subcommands.

```rust
#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  http_addr: String, // will be renamed to `--http-addr`

  #[structopt(subcommand)]
  addr_type: AddrType // this adds `addr-type` subcommand
}
```

`structopt` 0.2 used to leave things "as is", not renaming anything. If you want to keep old
behavior add `#[structopt(rename_all = "verbatim")]` on top of a `struct`/`enum`.

### Change `version`, `author` and `about` attributes behavior.
Proposed by [@TeXitoi](https://github.com/TeXitoi) [(#217)](https://github.com/TeXitoi/structopt/issues/217), implemented by [@CreepySkeleton](https://github.com/CreepySkeleton) [(#229)](https://github.com/TeXitoi/structopt/pull/229).

`structopt` have been deducing `version`, `author`, and `about` properties from `Cargo.toml`
for a long time (more accurately, from `CARGO_PKG_...` environment variables).
But many users found this behavior somewhat confusing, and a hack was added to cancel out
this behavior: `#[structopt(author = "")]`.

In `structopt` 0.3 this has changed.
* `author` and `about` are no longer deduced by default. You should use `#[structopt(author, about)]`
  to explicitly request `structopt` to deduce them.
* Contrary, `version` **is still deduced by default**. You can use `#[structopt(no_version)]` to
  cancel it out.
* `#[structopt(author = "", about = "", version = "")]` is no longer a valid syntax
  and will trigger an error.
* `#[structopt(version = "version", author = "author", about = "about")]` syntax
  stays unaffected by this changes.

### Raw attributes are removed ([#198](https://github.com/TeXitoi/structopt/pull/198)) by [@sphynx](https://github.com/sphynx)
In `structopt` 0.2 you were able to use any method from `clap::App` and `clap::Arg` via
raw attribute: `#[structopt(raw(method_name = "arg"))]`. This syntax was kind of awkward.

```rust
#[derive(StructOpt, Debug)]
#[structopt(raw(
    global_settings = "&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]"
))]
struct Opt {
    #[structopt(short = "l", long = "level", raw(aliases = r#"&["set-level", "lvl"]"#))]
    level: Vec<String>,
}
```

Raw attributes were removed in 0.3. Now you can use any method from `App` and `Arg` *directly*:
```rust
#[derive(StructOpt)]
#[structopt(global_settings(&[AppSettings::ColoredHelp, AppSettings::VersionlessSubcommands]))]
struct Opt {
    #[structopt(short = "l", long = "level", aliases(&["set-level", "lvl"]))]
    level: Vec<String>,
}
```

## Improvements

### Support skipping struct fields
Proposed by [@Morganamilo](https://github.com/Morganamilo) in ([#174](https://github.com/TeXitoi/structopt/issues/174))
implemented by [@sphynx](https://github.com/sphynx) in ([#213](https://github.com/TeXitoi/structopt/issues/213)).

Sometimes you want to include some fields in your `StructOpt` `struct` that are not options
and `clap` should know nothing about them. In `structopt` 0.3 it's possible via the
`#[structopt(skip)]` attribute. The field in question will be assigned with `Default::default()`
value.

```rust
#[derive(StructOpt)]
struct Opt {
    #[structopt(short, long)]
    speed: f32,

    car: String,

    // this field should not generate any arguments
    #[structopt(skip)]
    meta: Vec<u64>
}
```

### Add optional feature to support `paw` by [@gameldar](https://github.com/gameldar) ([#187](https://github.com/TeXitoi/structopt/issues/187))

### Significantly improve error reporting by [@CreepySkeleton](https://github.com/CreepySkeleton) ([#225](https://github.com/TeXitoi/structopt/pull/225/))
Now (almost) every error message points to the location it originates from:

```text
error: default_value is meaningless for bool
  --> $DIR/bool_default_value.rs:14:24
   |
14 |     #[structopt(short, default_value = true)]
   |                        ^^^^^^^^^^^^^
```

# v0.2.16 (2019-05-29)

### Support optional options with optional argument, allowing `cmd [--opt[=value]]` by [@sphynx](https://github.com/sphynx) ([#188](https://github.com/TeXitoi/structopt/issues/188))
Sometimes you want to represent an optional option that optionally takes an argument,
i.e `[--opt[=value]]`. This is represented by `Option<Option<T>>`

```rust
#[derive(StructOpt)]
struct Opt {
  #[structopt(long)]
  fruit: Option<Option<String>>,
}

fn main() {
  assert_eq!(Opt::from_args(&["test"]), None);
  assert_eq!(Opt::from_args(&["test", "--fruit"]), Some(None));
  assert_eq!(Opt::from_args(&["test", "--fruit=apple"]), Some("apple"));
}
```

# v0.2.15 (2019-03-08)

* Fix [#168](https://github.com/TeXitoi/structopt/issues/168) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.14 (2018-12-10)

* Introduce smarter parsing of doc comments by [@0ndorio](https://github.com/0ndorio)

# v0.2.13 (2018-11-01)

* Automatic naming of fields and subcommands by [@0ndorio](https://github.com/0ndorio)

# v0.2.12 (2018-10-11)

* Fix minimal clap version by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.11 (2018-10-05)

* Upgrade syn to 0.15 by [@konstin](https://github.com/konstin)

# v0.2.10 (2018-06-07)

* 1.21.0 is the minimum required rustc version by
  [@TeXitoi](https://github.com/TeXitoi)

# v0.2.9 (2018-06-05)

* Fix a bug when using `flatten` by
  [@fbenkstein](https://github.com/fbenkstein)
* Update syn, quote and proc_macro2 by
  [@TeXitoi](https://github.com/TeXitoi)
* Fix a regression when there is multiple authors by
  [@windwardly](https://github.com/windwardly)

# v0.2.8 (2018-04-28)

* Add `StructOpt::from_iter_safe()`, which returns an `Error` instead of
  killing the program when it fails to parse, or parses one of the
  short-circuiting flags. ([#98](https://github.com/TeXitoi/structopt/pull/98)
  by [@quodlibetor](https://github.com/quodlibetor))
* Allow users to enable `clap` features independently by
  [@Kerollmops](https://github.com/Kerollmops)
* Fix a bug when flattening an enum
  ([#103](https://github.com/TeXitoi/structopt/pull/103) by
  [@TeXitoi](https://github.com/TeXitoi)

# v0.2.7 (2018-04-12)

* Add flattening, the insertion of options of another StructOpt struct
  into another ([#92](https://github.com/TeXitoi/structopt/pull/92))
  by [@birkenfeld](https://github.com/birkenfeld)
* Fail compilation when using `default_value` or `required` with
  `Option` ([#88](https://github.com/TeXitoi/structopt/pull/88)) by
  [@Kerollmops](https://github.com/Kerollmops)

# v0.2.6 (2018-03-31)

* Fail compilation when using `default_value` or `required` with `bool` ([#80](https://github.com/TeXitoi/structopt/issues/80)) by [@TeXitoi](https://github.com/TeXitoi)
* Fix compilation with `#[deny(warnings)]` with the `!` type (https://github.com/rust-lang/rust/pull/49039#issuecomment-376398999) by [@TeXitoi](https://github.com/TeXitoi)
* Improve first example in the documentation ([#82](https://github.com/TeXitoi/structopt/issues/82)) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.5 (2018-03-07)

* Work around breakage when `proc-macro2`'s nightly feature is enabled. ([#77](https://github.com/Texitoi/structopt/pull/77) and [proc-macro2#67](https://github.com/alexcrichton/proc-macro2/issues/67)) by [@fitzgen](https://github.com/fitzgen)

# v0.2.4 (2018-02-25)

* Fix compilation with `#![deny(missig_docs]` ([#74](https://github.com/TeXitoi/structopt/issues/74)) by [@TeXitoi](https://github.com/TeXitoi)
* Fix [#76](https://github.com/TeXitoi/structopt/issues/76) by [@TeXitoi](https://github.com/TeXitoi)
* Re-licensed to Apache-2.0/MIT by [@CAD97](https://github.com/cad97)

# v0.2.3 (2018-02-16)

* An empty line in a doc comment will result in a double linefeed in the generated about/help call by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.2 (2018-02-12)

* Fix [#66](https://github.com/TeXitoi/structopt/issues/66) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.1 (2018-02-11)

* Fix a bug around enum tuple and the about message in the global help by [@TeXitoi](https://github.com/TeXitoi)
* Fix [#65](https://github.com/TeXitoi/structopt/issues/65) by [@TeXitoi](https://github.com/TeXitoi)

# v0.2.0 (2018-02-10)

## Breaking changes

### Don't special case `u64` by [@SergioBenitez](https://github.com/SergioBenitez)

If you are using a `u64` in your struct to get the number of occurence of a flag, you should now add `parse(from_occurrences)` on the flag.

For example
```rust
#[structopt(short = "v", long = "verbose")]
verbose: u64,
```
must be changed by
```rust
#[structopt(short = "v", long = "verbose", parse(from_occurrences))]
verbose: u64,
```

This feature was surprising as shown in [#30](https://github.com/TeXitoi/structopt/issues/30). Using the `parse` feature seems much more natural.

### Change the signature of `Structopt::from_clap` to take its argument by reference by [@TeXitoi](https://github.com/TeXitoi)

There was no reason to take the argument by value. Most of the StructOpt users will not be impacted by this change. If you are using `StructOpt::from_clap`, just add a `&` before the argument.

### Fail if attributes are not used by [@TeXitoi](https://github.com/TeXitoi)

StructOpt was quite fuzzy in its attribute parsing: it was only searching for interresting things, e. g. something like `#[structopt(foo(bar))]` was accepted but not used. It now fails the compilation.

You should have nothing to do here. This breaking change may highlight some missuse that can be bugs.

In future versions, if there is cases that are not highlighed, they will be considerated as bugs, not breaking changes.

### Use `raw()` wrapping instead of `_raw` suffixing by [@TeXitoi](https://github.com/TeXitoi)

The syntax of raw attributes is changed to improve the syntax.

You have to change `foo_raw = "bar", baz_raw = "foo"` by `raw(foo = "bar", baz = "foo")` or `raw(foo = "bar"), raw(baz = "foo")`.

## New features

* Add `parse(from_occurrences)` parser by [@SergioBenitez](https://github.com/SergioBenitez)
* Support 1-uple enum variant as subcommand by [@TeXitoi](https://github.com/TeXitoi)
* structopt-derive crate is now an implementation detail, structopt reexport the custom derive macro by [@TeXitoi](https://github.com/TeXitoi)
* Add the `StructOpt::from_iter` method by [@Kerollmops](https://github.com/Kerollmops)

## Documentation

* Improve doc by [@bestouff](https://github.com/bestouff)
* All the documentation is now on the structopt crate by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.7 (2018-01-23)

* Allow opting out of clap default features by [@ski-csis](https://github.com/ski-csis)

# v0.1.6 (2017-11-25)

* Improve documentation by [@TeXitoi](https://github.com/TeXitoi)
* Fix bug [#31](https://github.com/TeXitoi/structopt/issues/31) by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.5 (2017-11-14)

* Fix a bug with optional subsubcommand and Enum by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.4 (2017-11-09)

* Implement custom string parser from either `&str` or `&OsStr` by [@kennytm](https://github.com/kennytm)

# v0.1.3 (2017-11-01)

* Improve doc by [@TeXitoi](https://github.com/TeXitoi)

# v0.1.2 (2017-11-01)

* Fix bugs [#24](https://github.com/TeXitoi/structopt/issues/24) and [#25](https://github.com/TeXitoi/structopt/issues/25) by [@TeXitoi](https://github.com/TeXitoi)
* Support of methods with something else that a string as argument thanks to `_raw` suffix by [@Flakebi](https://github.com/Flakebi)

# v0.1.1 (2017-09-22)

* Better formating of multiple authors by [@killercup](https://github.com/killercup)

# v0.1.0 (2017-07-17)

* Subcommand support by [@williamyaoh](https://github.com/williamyaoh)

# v0.0.5 (2017-06-16)

* Using doc comment to populate help by [@killercup](https://github.com/killercup)

# v0.0.3 (2017-02-11)

* First version with flags, arguments and options support by [@TeXitoi](https://github.com/TeXitoi)
