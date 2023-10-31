<a name="v2.34.0"></a>
## v2.34.0 (2021-11-30)

- Updates to Rust 2018 edition and bumps the MSRV to Rust 1.46

<a name="v2.33.4"></a>
### v2.33.4 (2021-11-29)

#### Bug Fixes

* **prevents `panic`:**  swallows broken pipe errors on error output ([7a729bc4](https://github.com/kbknapp/clap-rs/commit/7a729bc4df2646b05f6bf15f001124cd39d076ce))

<a name="v2.33.3"></a>
### v2.33.3 (2020-08-13)

#### Improvements

* Suppress deprecation warnings when using `crate_*` macros.

<a name="v2.33.2"></a>
### v2.33.2 (2020-08-5)

#### Documentation

* Fixed links to `2.x` examples. Now they point to the right place.

<a name="v2.33.1"></a>
### v2.33.1 (2020-05-11)

#### Bug Fixes

* Windows: Prevent some panics when parsing invalid Unicode on Windows ([922c645](https://github.com/clap-rs/clap/commit/922c64508389170c9c77f1c8a4e597d14d3ed2f0), closes [#1905](https://github.com/clap-rs/clap/issues/1905))

#### Documentation

*   fixes versions referenced in the README ([d307466a](https://github.com/kbknapp/clap-rs/commit/d307466af1013f172b8ec0252f01a473e2192d6b))
* **README.md:**
  *  cuts down the number of examples to reduce confusion ([6e508ee0](https://github.com/kbknapp/clap-rs/commit/6e508ee09e7153de4adf4e88b0aa6418a537dadd))

#### Improvements

* **Deps:**  doesnt compile ansi_term on Windows since its not used ([b57ee946](https://github.com/kbknapp/clap-rs/commit/b57ee94609da3ddc897286cfba968f26ff961491), closes [#1155](https://github.com/kbknapp/clap-rs/issues/1155))

#### Minimum Required Rust

* As of this release, `clap` requires `rustc 1.36.0` or greater.

<a name="v2.33.0"></a>
## v2.33.0 (2019-04-06)

#### New Sponsor

*   Stephen Oats is now a sponsor \o/ ([823457c0](https://github.com/kbknapp/clap-rs/commit/823457c0ef5e994ed7080cf62addbfe1aa3b1833))
* **SPONSORS.md:**  fixes Josh Triplett's info in the sponsor document ([24cb5740](https://github.com/kbknapp/clap-rs/commit/24cb574090a11159b48bba105d5ec2dfb0a20e4e))

#### Features

* **Completions:**  adds completion support for Elvish. ([e9d0562a](https://github.com/kbknapp/clap-rs/commit/e9d0562a1dc5dfe731ed7c767e6cee0af08f0cf9))
* There is a new setting to disable automatic building of `--help` and `-h` flags (`AppSettings::DisableAutoHelp`)

#### Improvements

* **arg_matches.rs:**  add Debug implementations ([47192b7a](https://github.com/kbknapp/clap-rs/commit/47192b7a2d84ec716b81ae4af621e008a8762dc9))
* **macros:**  Support shorthand syntax for ArgGroups ([df9095e7](https://github.com/kbknapp/clap-rs/commit/df9095e75bb1e7896415251d0d4ffd8a0ebcd559))

#### Documentation

*   Refer to macOS rather than OSX. ([ab0d767f](https://github.com/kbknapp/clap-rs/commit/ab0d767f3a5a57e2bbb97d0183c2ef63c8c77a6c))
* **README.md:**  use https for all links ([96a7639a](https://github.com/kbknapp/clap-rs/commit/96a7639a36bcb184c3f45348986883115ef1ab3a))

#### Bug Fixes

*   add debug assertion for missing args in subcommand ArgGroup ([2699d9e5](https://github.com/kbknapp/clap-rs/commit/2699d9e51e7eadc258ba64c4e347c5d1fef61343))
*   Restore compat with Rust 1.21 ([6b263de1](https://github.com/kbknapp/clap-rs/commit/6b263de1d42ede692ec5ee55019ad2fc6386f92e))
*   Dont mention unused subcommands ([ef92e2b6](https://github.com/kbknapp/clap-rs/commit/ef92e2b639ed305bdade4741f60fa85cb0101c5a))
* **OsValues:**  Add `ExactSizeIterator` implementation ([356c69e5](https://github.com/kbknapp/clap-rs/commit/356c69e508fd25a9f0ea2d27bf80ae1d9a8d88f4))
* **arg_enum!:**
  *  Fix comma position for valid values. ([1f1f9ff3](https://github.com/kbknapp/clap-rs/commit/1f1f9ff3fa38a43231ef8be9cfea89a32e53f518))
  *  Invalid expansions of some trailing-comma patterns ([7023184f](https://github.com/kbknapp/clap-rs/commit/7023184fca04e852c270341548d6a16207d13862))
* **completions:**  improve correctness of completions when whitespace is involved ([5a08ff29](https://github.com/kbknapp/clap-rs/commit/5a08ff295b2aa6ce29420df6252a0e3ff4441bdc))
* **help message:**  Unconditionally uses long description for subcommands ([6acc8b6a](https://github.com/kbknapp/clap-rs/commit/6acc8b6a621a765cbf513450188000d943676a30), closes [#897](https://github.com/kbknapp/clap-rs/issues/897))
* **macros:**  fixes broken pattern which prevented calling multi-argument Arg methods ([9e7a352e](https://github.com/kbknapp/clap-rs/commit/9e7a352e13aaf8025d80f2bac5c47fb32528672b))
* **parser:**  Better interaction between AllowExternalSubcommands and SubcommandRequired ([9601c95a](https://github.com/kbknapp/clap-rs/commit/9601c95a03d2b82bf265c328b4769238f1b79002))

#### Minimum Required Rust

* As of this release, `clap` requires `rustc 1.31.0` or greater.

<a name="v2.32.0"></a>
## v2.32.0 (2018-06-26)

#### Minimum Required Rust

* As of this release, `clap` requires `rustc 1.21.0` or greater.


#### Features

* **Completions:**  adds completion support for Elvish. ([e9d0562a](https://github.com/kbknapp/clap-rs/commit/e9d0562a1dc5dfe731ed7c767e6cee0af08f0cf9))

#### Improvements

* **macros:**  Support shorthand syntax for ArgGroups ([df9095e7](https://github.com/kbknapp/clap-rs/commit/df9095e75bb1e7896415251d0d4ffd8a0ebcd559))

#### Bug Fixes

* **OsValues:**  Add `ExactSizeIterator` implementation ([356c69e5](https://github.com/kbknapp/clap-rs/commit/356c69e508fd25a9f0ea2d27bf80ae1d9a8d88f4))
* **arg_enum!:**  Invalid expansions of some trailing-comma patterns ([7023184f](https://github.com/kbknapp/clap-rs/commit/7023184fca04e852c270341548d6a16207d13862))
* **help message:**  Unconditionally uses long description for subcommands ([6acc8b6a](https://github.com/kbknapp/clap-rs/commit/6acc8b6a621a765cbf513450188000d943676a30), closes [#897](https://github.com/kbknapp/clap-rs/issues/897))

#### Documentation

*   Refer to macOS rather than OSX. ([ab0d767f](https://github.com/kbknapp/clap-rs/commit/ab0d767f3a5a57e2bbb97d0183c2ef63c8c77a6c))



<a name="v2.31.2"></a>
### v2.31.2 (2018-03-19)

#### Bug Fixes

* **Fish Completions:**  fixes a bug that only allowed a single completion in in Fish Shell ([e8774a8](https://github.com/kbknapp/clap-rs/pull/1214/commits/e8774a84ee4a319c888036e7c595ab46451d8e48), closes [#1212](https://github.com/kbknapp/clap-rs/issues/1212))
* **AllowExternalSubcommands**: fixes a bug where external subcommands would be blocked by a similarly named subcommand (suggestions were getting in the way). ([a410e85](https://github.com/kbknapp/clap-rs/pull/1215/commits/a410e855bcd82b05f9efa73fa8b9774dc8842c6b))

#### Documentation

* Fixes some typos in the `README.md` ([c8e685d7](https://github.com/kbknapp/clap-rs/commit/c8e685d76adee2a3cc06cac6952ffcf6f9548089))

<a name="v2.31.1"></a>
### v2.31.1 (2018-03-06)


#### Improvements

* **AllowMissingPositional:**  improves the ability of AllowMissingPositional to allow 'skipping' to the last positional arg with '--' ([df20e6e2](https://github.com/kbknapp/clap-rs/commit/df20e6e24b4e782be0b423b484b9798e3e2efe2f))


<a name="v2.31.0"></a>
## v2.31.0 (2018-03-04)


#### Features

* **Arg Indices:**  adds the ability to query argument value indices ([f58d0576](https://github.com/kbknapp/clap-rs/commit/f58d05767ec8133c8eb2de117cb642b9ae29ccbc))
* **Indices:**  implements an Indices<Item=&usize> iterator ([1e67be44](https://github.com/kbknapp/clap-rs/commit/1e67be44f0ccf161cc84c4e6082382072e89c302))
* **Raw Args** adds a convenience function to `Arg` that allows implying all of `Arg::last` `Arg::allow_hyphen_values` and `Arg::multiple(true)` ([66a78f29](https://github.com/kbknapp/clap-rs/commit/66a78f2972786f5fe7c07937a1ac23da2542afd2))

#### Documentation

*   Fix some typos and markdown issues. ([935ba0dd](https://github.com/kbknapp/clap-rs/commit/935ba0dd547a69c3f636c5486795012019408794))
* **Arg Indices:**  adds the documentation for the arg index querying methods ([50bc0047](https://github.com/kbknapp/clap-rs/commit/50bc00477afa64dc6cdc5de161d3de3ba1d105a7))
* **CONTRIBUTING.md:**  fix url to clippy upstream repo to point to https://github.com/rust-lang-nursery/rust-clippy instead of https://github.com/Manishearth/rust-clippy ([42407d7f](https://github.com/kbknapp/clap-rs/commit/42407d7f21d794103cda61f49d2615aae0a4bcd9))
* **Values:**  improves the docs example of the Values iterator ([74075d65](https://github.com/kbknapp/clap-rs/commit/74075d65e8db1ddb5e2a4558009a5729d749d1b6))
* Updates readme to hint that the `wrap_help` feature is a thing ([fc7ab227](https://github.com/kbknapp/clap-rs/commit/66a78f2972786f5fe7c07937a1ac23da2542afd2))

### Improvements

*  Cargo.toml: use codegen-units = 1 in release and bench profiles ([19f425ea](https://github.com/kbknapp/clap-rs/commit/66a78f2972786f5fe7c07937a1ac23da2542afd2))
*  Adds WASM support (clap now compiles on WASM!) ([689949e5](https://github.com/kbknapp/clap-rs/commit/689949e57d390bb61bc69f3ed91f60a2105738d0))
*  Uses the short help tool-tip for PowerShell completion scripts ([ecda22ce](https://github.com/kbknapp/clap-rs/commit/ecda22ce7210ce56d7b2d1a5445dd1b8a2959656))


<a name="v2.30.0"></a>
## v2.30.0 (2018-02-13)

#### Bug Fixes

* **YAML:** Adds a missing conversion from  `Arg::last` when instantiating from a YAML file ([aab77c81a5](https://github.com/kbknapp/clap-rs/pull/1175/commits/aab77c81a519b045f95946ae0dd3e850f9b93070), closes [#1160](https://github.com/kbknapp/clap-rs/issues/1173))

#### Improvements

* **Bash Completions:**  instead of completing a generic option name, all bash completions fall back to file completions UNLESS `Arg::possible_values` was used ([872f02ae](https://github.com/kbknapp/clap-rs/commit/872f02aea900ffa376850a279eb164645e1234fa))
* **Deps:**  No longer needlessly compiles `ansi_term` on Windows since its not used ([b57ee946](https://github.com/kbknapp/clap-rs/commit/b57ee94609da3ddc897286cfba968f26ff961491), closes [#1155](https://github.com/kbknapp/clap-rs/issues/1155))
* **Help Message:** changes the `[values: foo bar baz]` array to `[possible values: foo bar baz]` for consistency with the API ([414707e4e97](https://github.com/kbknapp/clap-rs/pull/1176/commits/414707e4e979d07bfe555247e5d130c546673708), closes [#1160](https://github.com/kbknapp/clap-rs/issues/1160))


<a name="v2.29.4"></a>
### v2.29.4 (2018-02-06)


#### Bug Fixes

* **Overrides Self:**  fixes a bug where options with multiple values couldnt ever have multiple values ([d95907cf](https://github.com/kbknapp/clap-rs/commit/d95907cff6d011a901fe35fa00b0f4e18547a1fb))



<a name="v2.29.3"></a>
### v2.29.3 (2018-02-05)


#### Improvements

* **Overrides:**  clap now supports arguments which override with themselves ([6c7a0010](https://github.com/kbknapp/clap-rs/commit/6c7a001023ca1eac1cc6ffe6c936b4c4a2aa3c45), closes [#976](https://github.com/kbknapp/clap-rs/issues/976))

#### Bug Fixes

* **Requirements:**  fixes an issue where conflicting args would still show up as required ([e06cefac](https://github.com/kbknapp/clap-rs/commit/e06cefac97083838c0a4e1444dcad02a5c3f911e), closes [#1158](https://github.com/kbknapp/clap-rs/issues/1158))
* Fixes a bug which disallows proper nesting of `--` ([73993fe](https://github.com/kbknapp/clap-rs/commit/73993fe30d135f682e763ec93dcb0814ed518011), closes [#1161](https://github.com/kbknapp/clap-rs/issues/1161))

#### New Settings

* **AllArgsOverrideSelf:**  adds a new convenience setting to allow all args to override themselves ([4670325d](https://github.com/kbknapp/clap-rs/commit/4670325d1bf0369addec2ae2bcb56f1be054c924))



<a name="v2.29.2"></a>
### v2.29.2 (2018-01-16)


#### Features

* **completions/zsh.rs:**
  *  Escape possible values for options ([25561dec](https://github.com/kbknapp/clap-rs/commit/25561decf147d329b64634a14d9695673c2fc78f))
  *  Implement postional argument possible values completion ([f3b0afd2](https://github.com/kbknapp/clap-rs/commit/f3b0afd2bef8b7be97162f8a7802ddf7603dff36))
  *  Complete positional arguments properly ([e39aeab8](https://github.com/kbknapp/clap-rs/commit/e39aeab8487596046fbdbc6a226e5c8820585245))

#### Bug Fixes

* **completions/zsh.rs:**
  *  Add missing autoload for is-at-least ([a6522607](https://github.com/kbknapp/clap-rs/commit/a652260795d1519f6ec2a7a09ccc1258499cad7b))
  *  Don't pass -S to _arguments if Zsh is too old ([16b4f143](https://github.com/kbknapp/clap-rs/commit/16b4f143ff466b7ef18a267bc44ade0f9639109b))
  *  Maybe fix completions with mixed positionals and subcommands ([1146f0da](https://github.com/kbknapp/clap-rs/commit/1146f0da154d6796fbfcb09db8efa3593cb0d898))
* **completions/zsh.zsh:**  Remove redundant code from output ([0e185b92](https://github.com/kbknapp/clap-rs/commit/0e185b922ed1e0fd653de00b4cd8d567d72ff68e), closes [#1142](https://github.com/kbknapp/clap-rs/issues/1142))



<a name="2.29.1"></a>
### 2.29.1 (2018-01-09)


#### Documentation

*   fixes broken links. ([56e734b8](https://github.com/kbknapp/clap-rs/commit/56e734b839303d733d2e5baf7dac39bd7b97b8e4))
*   updates contributors list ([e1313a5a](https://github.com/kbknapp/clap-rs/commit/e1313a5a0f69d8f4016f73b860a63af8318a6676))

#### Performance

*   further debloating by removing generics from error cases ([eb8d919e](https://github.com/kbknapp/clap-rs/commit/eb8d919e6f3443db279ba0c902f15d76676c02dc))
*   debloats clap by deduplicating logic and refactors ([03e413d7](https://github.com/kbknapp/clap-rs/commit/03e413d7175d35827cd7d8908d47dbae15a849a3))

#### Bug Fixes

*   fixes the ripgrep benchmark by adding a value to a flag that expects it ([d26ab2b9](https://github.com/kbknapp/clap-rs/commit/d26ab2b97cf9c0ea675b440b7b0eaf6ac3ad01f4))
* **bash completion:**  Change the bash completion script code generation to support hyphens. ([ba7f1d18](https://github.com/kbknapp/clap-rs/commit/ba7f1d18eba7a07ce7f57e0981986f66c994b639))
* **completions/zsh.rs:**  Fix completion of long option values ([46365cf8](https://github.com/kbknapp/clap-rs/commit/46365cf8be5331ba04c895eb183e2f230b5aad51))


<a name="2.29.0"></a>
## 2.29.0 (2017-12-02)


#### API Additions

* **Arg:**  adds Arg::hide_env_values(bool) which allows one to hide any current env values and display only the key in help messages ([fb41d062](https://github.com/kbknapp/clap-rs/commit/fb41d062eedf37cb4f805c90adca29909bd197d7))



<a name="2.28.0"></a>
## 2.28.0 (2017-11-28)

The minimum required Rust is now 1.20. This was done to start using bitflags 1.0 and having >1.0 deps is a *very good* thing!

#### Documentation

*   changes the demo version to 2.28 to stay in sync ([ce6ca492](https://github.com/kbknapp/clap-rs/commit/ce6ca492c7510ab6474075806360b96081b021a9))
*   Fix URL path to github hosted files ([ce72aada](https://github.com/kbknapp/clap-rs/commit/ce72aada56a9581d4a6cb4bf9bdb861c3906f8df), closes [#1106](https://github.com/kbknapp/clap-rs/issues/1106))
*   fix typo ([002b07fc](https://github.com/kbknapp/clap-rs/commit/002b07fc98a1c85acb66296b1eec0b2aba906125))
* **README.md:**  updates the readme and pulls out some redundant sections ([db6caf86](https://github.com/kbknapp/clap-rs/commit/db6caf8663747e679d2f4ed3bd127f33476754aa))

#### Improvements

*   adds '[SUBCOMMAND]' to usage strings with only AppSettings::AllowExternalSubcommands is used with no other subcommands ([e78bb757](https://github.com/kbknapp/clap-rs/commit/e78bb757a3df16e82d539e450c06767a6bfcf859), closes [#1093](https://github.com/kbknapp/clap-rs/issues/1093))

#### API Additions

*   Adds Arg::case_insensitive(bool) which allows matching Arg::possible_values without worrying about ASCII case ([1fec268e](https://github.com/kbknapp/clap-rs/commit/1fec268e51736602e38e67c76266f439e2e0ef12), closes [#1118](https://github.com/kbknapp/clap-rs/issues/1118))
*   Adds the traits to be used with the clap-derive crate to be able to use Custom Derive ([6f4c3412](https://github.com/kbknapp/clap-rs/commit/6f4c3412415e882f5ca2cc3fbd6d4dce79440828))

#### Bug Fixes

*   Fixes a regression where --help couldn't be overridden ([a283d69f](https://github.com/kbknapp/clap-rs/commit/a283d69fc08aa016ae1bf9ba010012abecc7ba69), closes [#1112](https://github.com/kbknapp/clap-rs/issues/1112))
*   fixes a bug that allowed options to pass parsing when no value was provided ([2fb75821](https://github.com/kbknapp/clap-rs/commit/2fb758219c7a60d639da67692e100b855a8165ac), closes [#1105](https://github.com/kbknapp/clap-rs/issues/1105))
*   ignore PropagateGlobalValuesDown deprecation warning ([f61ce3f5](https://github.com/kbknapp/clap-rs/commit/f61ce3f55fe65e16b3db0bd4facdc4575de22767), closes [#1086](https://github.com/kbknapp/clap-rs/issues/1086))

#### Deps

*  Updates `bitflags` to 1.0



<a name="v2.27.1"></a>
## v2.27.1 (2017-10-24)


#### Bug Fixes

* Adds `term_size` as an optional dependency (with feature `wrap_help`) to fix compile bug

<a name="v2.27.0"></a>
## v2.27.0 (2017-10-24)

** This release raises the minimum required version of Rust to 1.18 **

** This release also contains a very minor breaking change to fix a bug **

The only CLIs affected will be those using unrestrained multiple values and subcommands where the
subcommand name can coincide with one of the multiple values.

See the commit [0c223f54](https://github.com/kbknapp/clap-rs/commit/0c223f54ed46da406bc8b43a5806e0b227863b31) for full details.


#### Bug Fixes

*   Values from global args are now propagated UP and DOWN!
*   fixes a bug where using AppSettings::AllowHyphenValues would allow invalid arguments even when there is no way for them to be valid ([77ed4684](https://github.com/kbknapp/clap-rs/commit/77ed46841fc0263d7aa32fcc5cc49ef703b37c04), closes [#1066](https://github.com/kbknapp/clap-rs/issues/1066))
*   when an argument requires a value and that value happens to match a subcommand name, its parsed as a value ([0c223f54](https://github.com/kbknapp/clap-rs/commit/0c223f54ed46da406bc8b43a5806e0b227863b31), closes [#1031](https://github.com/kbknapp/clap-rs/issues/1031), breaks [#](https://github.com/kbknapp/clap-rs/issues/), [#](https://github.com/kbknapp/clap-rs/issues/))
*   fixes a bug that prevented number_of_values and default_values to be used together ([5eb342a9](https://github.com/kbknapp/clap-rs/commit/5eb342a99dde07b0f011048efde3e283bc1110fc), closes [#1050](https://github.com/kbknapp/clap-rs/issues/1050), [#1056](https://github.com/kbknapp/clap-rs/issues/1056))
*   fixes a bug that didn't allow args with default values to have conflicts ([58b5b4be](https://github.com/kbknapp/clap-rs/commit/58b5b4be315280888d50d9b15119b91a9028f050), closes [#1071](https://github.com/kbknapp/clap-rs/issues/1071))
*   fixes a panic when using global args and calling App::get_matches_from_safe_borrow multiple times ([d86ec797](https://github.com/kbknapp/clap-rs/commit/d86ec79742c77eb3f663fb30e225954515cf25bb), closes [#1076](https://github.com/kbknapp/clap-rs/issues/1076))
*   fixes issues and potential regressions with global args values not being propagated properly or at all ([a43f9dd4](https://github.com/kbknapp/clap-rs/commit/a43f9dd4aaf1864dd14a3c28dec89ccdd70c61e5), closes [#1010](https://github.com/kbknapp/clap-rs/issues/1010), [#1061](https://github.com/kbknapp/clap-rs/issues/1061), [#978](https://github.com/kbknapp/clap-rs/issues/978))
*   fixes a bug where default values are not applied if the option supports zero values ([9c248cbf](https://github.com/kbknapp/clap-rs/commit/9c248cbf7d8a825119bc387c23e9a1d1989682b0), closes [#1047](https://github.com/kbknapp/clap-rs/issues/1047))

#### Documentation

*   adds addtional blurbs about using multiples with subcommands ([03455b77](https://github.com/kbknapp/clap-rs/commit/03455b7751a757e7b2f6ffaf2d16168539c99661))
*   updates the docs to reflect changes to global args and that global args values can now be propagated back up the stack ([ead076f0](https://github.com/kbknapp/clap-rs/commit/ead076f03ada4c322bf3e34203925561ec496d87))
*   add html_root_url attribute ([e67a061b](https://github.com/kbknapp/clap-rs/commit/e67a061bcf567c6518d6c2f58852e01f02764b22))
*   sync README version numbers with crate version ([5536361b](https://github.com/kbknapp/clap-rs/commit/5536361bcda29887ed86bb68e43d0b603cbc423f))

#### Improvements

*   args that have require_delimiter(true) is now reflected in help and usage strings ([dce61699](https://github.com/kbknapp/clap-rs/commit/dce616998ed9bd95e8ed3bec1f09a4883da47b85), closes [#1052](https://github.com/kbknapp/clap-rs/issues/1052))
*   if all subcommands are hidden, the subcommands section of the help message is no longer displayed ([4ae7b046](https://github.com/kbknapp/clap-rs/commit/4ae7b0464750bc07ec80ece38e43f003fdd1b8ae), closes [#1046](https://github.com/kbknapp/clap-rs/issues/1046))

#### Breaking Changes

*   when an argument requires a value and that value happens to match a subcommand name, its parsed as a value ([0c223f54](https://github.com/kbknapp/clap-rs/commit/0c223f54ed46da406bc8b43a5806e0b227863b31), closes [#1031](https://github.com/kbknapp/clap-rs/issues/1031), breaks [#](https://github.com/kbknapp/clap-rs/issues/), [#](https://github.com/kbknapp/clap-rs/issues/))

#### Deprecations

* **AppSettings::PropagateGlobalValuesDown:**  this setting is no longer required to propagate values down or up ([2bb5ddce](https://github.com/kbknapp/clap-rs/commit/2bb5ddcee61c791ca1aaca494fbeb4bd5e277488))



<a name="v2.26.2"></a>
### v2.26.2 (2017-09-14)


#### Improvements

*   if all subcommands are hidden, the subcommands section of the help message is no longer displayed ([4ae7b046](https://github.com/kbknapp/clap-rs/commit/4ae7b0464750bc07ec80ece38e43f003fdd1b8ae), closes [#1046](https://github.com/kbknapp/clap-rs/issues/1046))

#### Bug Fixes

*   fixes a bug where default values are not applied if the option supports zero values ([9c248cbf](https://github.com/kbknapp/clap-rs/commit/9c248cbf7d8a825119bc387c23e9a1d1989682b0), closes [#1047](https://github.com/kbknapp/clap-rs/issues/1047))



<a name="v2.26.1"></a>
### v2.26.1 (2017-09-14)


#### Bug Fixes

*   fixes using require_equals(true) and min_values(0) together ([10ae208f](https://github.com/kbknapp/clap-rs/commit/10ae208f68518eff6e98166724065745f4083174), closes [#1044](https://github.com/kbknapp/clap-rs/issues/1044))
*   escape special characters in zsh and fish completions ([87e019fc](https://github.com/kbknapp/clap-rs/commit/87e019fc84ba6193a8c4ddc26c61eb99efffcd25))
*   avoid panic generating default help msg if term width set to 0 due to bug in textwrap 0.7.0 ([b3eadb0d](https://github.com/kbknapp/clap-rs/commit/b3eadb0de516106db4e08f078ad32e8f6d6e7a57))
*   Change `who's` -> `whose` ([53c1ffe8](https://github.com/kbknapp/clap-rs/commit/53c1ffe87f38b05d8804a0f7832412a952845349))
*   adds a debug assertion to ensure all args added to groups actually exist ([7ad123e2](https://github.com/kbknapp/clap-rs/commit/7ad123e2c02577e3ca30f7e205181e896b157d11), closes [#917](https://github.com/kbknapp/clap-rs/issues/917))
*   fixes a bug where args that allow values to start with a hyphen couldnt contain a double hyphen -- as a value ([ab2f4c9e](https://github.com/kbknapp/clap-rs/commit/ab2f4c9e563e36ec739a4b55d5a5b76fdb9e9fa4), closes [#960](https://github.com/kbknapp/clap-rs/issues/960))
*   fixes a bug where positional argument help text is misaligned ([54c16836](https://github.com/kbknapp/clap-rs/commit/54c16836dea4651806a2cfad53146a83fa3abf21))
* **Help Message:**  fixes long_about not being usable ([a8257ea0](https://github.com/kbknapp/clap-rs/commit/a8257ea0ffb812e552aca256c4a3d2aebfd8065b), closes [#1043](https://github.com/kbknapp/clap-rs/issues/1043))
* **Suggestions:**  output for flag after subcommand ([434ea5ba](https://github.com/kbknapp/clap-rs/commit/434ea5ba71395d8c1afcf88e69f0b0d8339b01a1))



<a name="v2.26.0"></a>
## v2.26.0 (2017-07-29)

Minimum version of Rust is now v1.13.0 (Stable)


#### Improvements

*   bumps unicode-segmentation to v1.2 ([cd7b40a2](https://github.com/kbknapp/clap-rs/commit/cd7b40a21c77bae17ba453c5512cb82b7d1ce474))


#### Performance

*   update textwrap to version 0.7.0 ([c2d4e637](https://github.com/kbknapp/clap-rs/commit/c2d4e63756a6f070e38c16dff846e9b0a53d6f93))




<a name="v2.25.1"></a>
### v2.25.1 (2017-07-21)

#### Improvements

* impl Default for Values + OsValues for any lifetime. ([fb7d6231f1](https://github.com/kbknapp/clap-rs/commit/fb7d6231f13a2f79f411e62dca210b7dc9994c18))

#### Documentation

* Various documentation typos and grammar fixes

<a name="v2.25.0"></a>
### v2.25.0 (2017-06-20)


#### Features

*   use textwrap crate for wrapping help texts ([b93870c1](https://github.com/kbknapp/clap-rs/commit/b93870c10ae3bd90d233c586a33e086803117285))

#### Improvements

* **Suggestions:**  suggests to use flag after subcommand when applicable ([2671ca72](https://github.com/kbknapp/clap-rs/commit/2671ca7260119d4311d21c4075466aafdd9da734))
* Bumps bitflags crate to v0.9

#### Documentation

*   Change `who's` -> `whose` ([53c1ffe8](https://github.com/kbknapp/clap-rs/commit/53c1ffe87f38b05d8804a0f7832412a952845349))

#### Documentation

* **App::template:**  adds details about the necessity to use AppSettings::UnifiedHelpMessage when using {unified} tags in the help template ([cbea3d5a](https://github.com/kbknapp/clap-rs/commit/cbea3d5acf3271a7a734498c4d99c709941c331e), closes [#949](https://github.com/kbknapp/clap-rs/issues/949))
* **Arg::allow_hyphen_values:**  updates the docs to include warnings for allow_hyphen_values and multiple(true) used together ([f9b0d657](https://github.com/kbknapp/clap-rs/commit/f9b0d657835d3f517f313d70962177dc30acf4a7))
* **README.md:**
  *  added a warning about using ~ deps ([821929b5](https://github.com/kbknapp/clap-rs/commit/821929b51bd60213955705900a436c9a64fcb79f), closes [#964](https://github.com/kbknapp/clap-rs/issues/964))
* **clap_app!:**  adds using the @group specifier to the macro docs ([826048cb](https://github.com/kbknapp/clap-rs/commit/826048cb3cbc0280169303f1498ff0a2b7395883), closes [#932](https://github.com/kbknapp/clap-rs/issues/932))



<a name="v2.24.2"></a>
### v2.24.2 (2017-05-15)


#### Bug Fixes

*   adds a debug assertion to ensure all args added to groups actually exist ([14f6b8f3](https://github.com/kbknapp/clap-rs/commit/14f6b8f3a2f6df73aeeec9c54a54909b1acfc158), closes [#917](https://github.com/kbknapp/clap-rs/issues/917))
*   fixes a bug where args that allow values to start with a hyphen couldnt contain a double hyphen -- as a value ([ebf73a09](https://github.com/kbknapp/clap-rs/commit/ebf73a09db6f3c03c19cdd76b1ba6113930e1643), closes [#960](https://github.com/kbknapp/clap-rs/issues/960))
*   fixes a bug where positional argument help text is misaligned ([54c16836](https://github.com/kbknapp/clap-rs/commit/54c16836dea4651806a2cfad53146a83fa3abf21))

#### Documentation

* **App::template:**  adds details about the necessity to use AppSettings::UnifiedHelpMessage when using {unified} tags in the help template ([cf569438](https://github.com/kbknapp/clap-rs/commit/cf569438f309c199800bb8e46c9f140187de69d7), closes [#949](https://github.com/kbknapp/clap-rs/issues/949))
* **Arg::allow_hyphen_values:**  updates the docs to include warnings for allow_hyphen_values and multiple(true) used together ([ded5a2f1](https://github.com/kbknapp/clap-rs/commit/ded5a2f15474d4a5bd46a67b130ccb8b6781bd01))
* **clap_app!:**  adds using the @group specifier to the macro docs ([fe85fcb1](https://github.com/kbknapp/clap-rs/commit/fe85fcb1772b61f13b20b7ea5290e2437a76190c), closes [#932](https://github.com/kbknapp/clap-rs/issues/932))



<a name="v2.24.0"></a>
### v2.24.0 (2017-05-07)


#### Bug Fixes

*   fixes a bug where args with last(true) and required(true) set were not being printed in the usage string ([3ac533fe](https://github.com/kbknapp/clap-rs/commit/3ac533fedabf713943eedf006f830a5a486bbe80), closes [#944](https://github.com/kbknapp/clap-rs/issues/944))
*   fixes a bug that was printing the arg name, instead of value name when Arg::last(true) was used ([e1fe8ac3](https://github.com/kbknapp/clap-rs/commit/e1fe8ac3bc1f9cf4e36df0d881f8419755f1787b), closes [#940](https://github.com/kbknapp/clap-rs/issues/940))
*   fixes a bug where flags were parsed as flags AND positional values when specific combinations of settings were used ([20f83292](https://github.com/kbknapp/clap-rs/commit/20f83292d070038b8cee2a6b47e91f6b0a2f7871), closes [#946](https://github.com/kbknapp/clap-rs/issues/946))



<a name="v2.24.0"></a>
## v2.24.0 (2017-05-05)


#### Documentation

* **README.md:**  fix some typos ([fa34deac](https://github.com/kbknapp/clap-rs/commit/fa34deac079f334c3af97bb7fb151880ba8887f8))

#### API Additions

* **Arg:**  add `default_value_os` ([d5ef8955](https://github.com/kbknapp/clap-rs/commit/d5ef8955414b1587060f7218385256105b639c88))
* **arg_matches.rs:**  Added a Default implementation for Values and OsValues iterators. ([0a4384e3](https://github.com/kbknapp/clap-rs/commit/0a4384e350eed74c2a4dc8964c203f21ac64897f))


<a name="v2.23.2"></a>
### v2.23.2 (2017-04-19)


#### Bug Fixes

* **PowerShell Completions:**  fixes a bug where powershells completions cant be used if no subcommands are defined ([a8bce558](https://github.com/kbknapp/clap-rs/commit/a8bce55837dc4e0fb187dc93180884a40ae09c6f), closes [#931](https://github.com/kbknapp/clap-rs/issues/931))

#### Improvements

*   bumps term_size to take advantage of better terminal dimension handling ([e05100b7](https://github.com/kbknapp/clap-rs/commit/e05100b73d74066a90876bf38f952adf5e8ee422))
* **PowerShell Completions:**  massively dedups subcommand names in the generate script to make smaller scripts that are still functionally equiv ([85b0e1cc](https://github.com/kbknapp/clap-rs/commit/85b0e1cc4b9755dda75a93d898d79bc38631552b))

#### Documentation

*   Fix a typo the minimum rust version required ([71dabba3](https://github.com/kbknapp/clap-rs/commit/71dabba3ea0a17c88b0e2199c9d99f0acbf3bc17))

<a name="v2.23.1"></a>
### v2.23.1 (2017-04-05)


#### Bug Fixes

*   fixes a missing newline character in the autogenerated help and version messages in some instances ([5ae9007d](https://github.com/kbknapp/clap-rs/commit/5ae9007d984ae94ae2752df51bcbaeb0ec89bc15))


<a name="v2.23.0"></a>
## v2.23.0 (2017-04-05)


#### API Additions

* `App::long_about`
* `App::long_version`
* `App::print_long_help`
* `App::write_long_help`
* `App::print_long_version`
* `App::write_long_version`
* `Arg::long_help`

#### Features

*   allows distinguishing between short and long version messages (-V/short or --version/long) ([59272b06](https://github.com/kbknapp/clap-rs/commit/59272b06cc213289dc604dbc694cb95d383a5d68))
*   allows distinguishing between short and long help with subcommands in the same manner as args ([6b371891](https://github.com/kbknapp/clap-rs/commit/6b371891a1702173a849d1e95f9fecb168bf6fc4))
*   allows specifying a short help vs a long help (i.e. varying levels of detail depending on if -h or --help was used) ([ef1b24c3](https://github.com/kbknapp/clap-rs/commit/ef1b24c3a0dff2f58c5e2e90880fbc2b69df20ee))
* **clap_app!:**  adds support for arg names with hyphens similar to longs with hyphens ([f7a88779](https://github.com/kbknapp/clap-rs/commit/f7a8877978c8f90e6543d4f0d9600c086cf92cd7), closes [#869](https://github.com/kbknapp/clap-rs/issues/869))

#### Bug Fixes

*   fixes a bug that wasn't allowing help and version to be properly overridden ([8b2ceb83](https://github.com/kbknapp/clap-rs/commit/8b2ceb8368bcb70689fadf1c7f4b9549184926c1), closes [#922](https://github.com/kbknapp/clap-rs/issues/922))

#### Documentation

* **clap_app!:**  documents the `--("some-arg")` method for using args with hyphens inside them ([bc08ef3e](https://github.com/kbknapp/clap-rs/commit/bc08ef3e185393073d969d301989b6319c616c1f), closes [#919](https://github.com/kbknapp/clap-rs/issues/919))



<a name="v2.22.2"></a>
### v2.22.2 (2017-03-30)


#### Bug Fixes

* **Custom Usage Strings:**  fixes the usage string regression when using help templates ([0e4fd96d](https://github.com/kbknapp/clap-rs/commit/0e4fd96d74280d306d09e60ac44f938a82321769))



<a name="v2.22.1"></a>
### v2.22.1 (2017-03-24)


#### Bug Fixes

* **usage:**  fixes a big regression with custom usage strings ([2c41caba](https://github.com/kbknapp/clap-rs/commit/2c41caba3c7d723a2894e315d04da796b0e97759))

<a name="v2.22.0"></a>
## v2.22.0 (2017-03-23)

#### API Additions

* **App::name:**  adds the ability to change the name of the App instance after creation ([d49e8292](https://github.com/kbknapp/clap-rs/commit/d49e8292b026b06e2b70447cd9f08299f4fcba76), closes [#908](https://github.com/kbknapp/clap-rs/issues/908))
* **Arg::hide_default_value:**  adds ability to hide the default value of an argument from the help string ([89e6ea86](https://github.com/kbknapp/clap-rs/commit/89e6ea861e16a1ad56757ca12f6b32d02253e44a), closes [#902](https://github.com/kbknapp/clap-rs/issues/902))


<a name="v2.21.3"></a>
### v2.21.3 (2017-03-23)

#### Bug Fixes

* **yaml:**  adds support for loading author info from yaml ([e04c390c](https://github.com/kbknapp/clap-rs/commit/e04c390c597a55fa27e724050342f16c42f1c5c9))


<a name="v2.21.2"></a>
### v2.21.2 (2017-03-17)


#### Improvements

*   add fish subcommand help support ([f8f68cf8](https://github.com/kbknapp/clap-rs/commit/f8f68cf8251669aef4539a25a7c1166f0ac81ea6))
*   options that use `require_equals(true)` now display the equals sign in help messages, usage strings, and errors" ([c8eb0384](https://github.com/kbknapp/clap-rs/commit/c8eb0384d394d2900ccdc1593099c97808a3fa05), closes [#903](https://github.com/kbknapp/clap-rs/issues/903))


#### Bug Fixes

*  setting the max term width now correctly propagates down through child subcommands



<a name="v2.21.1"></a>
### v2.21.1 (2017-03-12)


#### Bug Fixes

* **ArgRequiredElseHelp:**  fixes the precedence of this error to prioritize over other error messages ([74b751ff](https://github.com/kbknapp/clap-rs/commit/74b751ff2e3631e337b7946347c1119829a41c53), closes [#895](https://github.com/kbknapp/clap-rs/issues/895))
* **Positionals:**  fixes some regression bugs resulting from old asserts in debug mode. ([9a3bc98e](https://github.com/kbknapp/clap-rs/commit/9a3bc98e9b55e7514b74b73374c5ac8b6e5e0508), closes [#896](https://github.com/kbknapp/clap-rs/issues/896))



<a name="v2.21.0"></a>
## v2.21.0 (2017-03-09)

#### Performance

*   doesn't run `arg_post_processing` on multiple values anymore ([ec516182](https://github.com/kbknapp/clap-rs/commit/ec5161828729f6a53f0fccec8648f71697f01f78))
*   changes internal use of `VecMap` to `Vec` for matched values of `Arg`s ([22bf137a](https://github.com/kbknapp/clap-rs/commit/22bf137ac581684c6ed460d2c3c640c503d62621))
*   vastly reduces the amount of cloning when adding non-global args minus when they're added from `App::args` which is forced to clone ([8da0303b](https://github.com/kbknapp/clap-rs/commit/8da0303bc02db5fe047cfc0631a9da41d9dc60f7))
*   refactor to remove unneeded vectors and allocations and checks for significant performance increases ([0efa4119](https://github.com/kbknapp/clap-rs/commit/0efa4119632f134fc5b8b9695b007dd94b76735d))

#### Documentation

*   Fix examples link in CONTRIBUTING.md ([60cf875d](https://github.com/kbknapp/clap-rs/commit/60cf875d67a252e19bb85054be57696fac2c57a1))

#### Improvements

*   when `AppSettings::SubcommandsNegateReqs` and `ArgsNegateSubcommands` are used, a new more accurate double line usage string is shown ([50f02300](https://github.com/kbknapp/clap-rs/commit/50f02300d81788817acefef0697e157e01b6ca32), closes [#871](https://github.com/kbknapp/clap-rs/issues/871))

#### API Additions

* **Arg::last:**  adds the ability to mark a positional argument as 'last' which means it should be used with `--` syntax and can be accessed early ([6a7aea90](https://github.com/kbknapp/clap-rs/commit/6a7aea9043b83badd9ab038b4ecc4c787716147e), closes [#888](https://github.com/kbknapp/clap-rs/issues/888))
*   provides `default_value_os` and `default_value_if[s]_os` ([0f2a3782](https://github.com/kbknapp/clap-rs/commit/0f2a378219a6930748d178ba350fe5925be5dad5), closes [#849](https://github.com/kbknapp/clap-rs/issues/849))
*   provides `App::help_message` and `App::version_message` which allows one to override the auto-generated help/version flag associated help ([389c413](https://github.com/kbknapp/clap-rs/commit/389c413b7023dccab8c76aa00577ea1d048e7a99), closes [#889](https://github.com/kbknapp/clap-rs/issues/889))

#### New Settings

* **InferSubcommands:**  adds a setting to allow one to infer shortened subcommands or aliases (i.e. for subcommmand "test", "t", "te", or "tes" would be allowed assuming no other ambiguities) ([11602032](https://github.com/kbknapp/clap-rs/commit/11602032f6ff05881e3adf130356e37d5e66e8f9), closes [#863](https://github.com/kbknapp/clap-rs/issues/863))

#### Bug Fixes

*   doesn't print the argument sections in the help message if all args in that section are hidden ([ce5ee5f5](https://github.com/kbknapp/clap-rs/commit/ce5ee5f5a76f838104aeddd01c8ec956dd347f50))
*   doesn't include the various [ARGS] [FLAGS] or [OPTIONS] if the only ones available are hidden ([7b4000af](https://github.com/kbknapp/clap-rs/commit/7b4000af97637703645c5fb2ac8bb65bd546b95b), closes [#882](https://github.com/kbknapp/clap-rs/issues/882))
*   now correctly shows subcommand as required in the usage string when AppSettings::SubcommandRequiredElseHelp is used ([8f0884c1](https://github.com/kbknapp/clap-rs/commit/8f0884c1764983a49b45de52a1eddf8d721564d8))
*   fixes some memory leaks when an error is detected and clap exits ([8c2dd287](https://github.com/kbknapp/clap-rs/commit/8c2dd28718262ace4ae0db98563809548e02a86b))
*   fixes a trait that's marked private accidentlly, but should be crate internal public ([1ae21108](https://github.com/kbknapp/clap-rs/commit/1ae21108015cea87e5360402e1747025116c7878))
* **Completions:**   fixes a bug that tried to propogate global args multiple times when generating multiple completion scripts ([5e9b9cf4](https://github.com/kbknapp/clap-rs/commit/5e9b9cf4dd80fa66a624374fd04e6545635c1f94), closes [#846](https://github.com/kbknapp/clap-rs/issues/846))

#### Features

* **Options:**  adds the ability to require the equals syntax with options --opt=val ([f002693d](https://github.com/kbknapp/clap-rs/commit/f002693dec6a6959c4e9590cb7b7bfffd6d6e5bc), closes [#833](https://github.com/kbknapp/clap-rs/issues/833))



<a name="v2.20.5"></a>
### v2.20.5 (2017-02-18)


#### Bug Fixes

* **clap_app!:**   fixes a critical bug of a missing fragment specifier when using `!property` style tags. ([5635c1f94](https://github.com/kbknapp/clap-rs/commit/5e9b9cf4dd80fa66a624374fd04e6545635c1f94))


<a name="v2.20.4"></a>
### v2.20.4 (2017-02-15)


#### Bug Fixes

* **Completions:**   fixes a bug that tried to propogate global args multiple times when generating multiple completion scripts ([5e9b9cf4](https://github.com/kbknapp/clap-rs/commit/5e9b9cf4dd80fa66a624374fd04e6545635c1f94), closes [#846](https://github.com/kbknapp/clap-rs/issues/846))

#### Documentation

*   Fix examples link in CONTRIBUTING.md ([60cf875d](https://github.com/kbknapp/clap-rs/commit/60cf875d67a252e19bb85054be57696fac2c57a1))


<a name="v2.20.3"></a>
### v2.20.3 (2017-02-03)


#### Documentation

* **Macros:**  adds a warning about changing values in Cargo.toml not triggering a rebuild automatically ([112aea3e](https://github.com/kbknapp/clap-rs/commit/112aea3e42ae9e0c0a2d33ebad89496dbdd95e5d), closes [#838](https://github.com/kbknapp/clap-rs/issues/838))

#### Bug Fixes

*   fixes a println->debugln typo ([279aa62e](https://github.com/kbknapp/clap-rs/commit/279aa62eaf08f56ce090ba16b937bc763cbb45be))
*   fixes bash completions for commands that have an underscore in the name ([7f5cfa72](https://github.com/kbknapp/clap-rs/commit/7f5cfa724f0ac4e098f5fe466c903febddb2d994), closes [#581](https://github.com/kbknapp/clap-rs/issues/581))
*   fixes a bug where ZSH completions would panic if the binary name had an underscore in it ([891a2a00](https://github.com/kbknapp/clap-rs/commit/891a2a006f775e92c556dda48bb32fac9807c4fb), closes [#581](https://github.com/kbknapp/clap-rs/issues/581))
*   allow final word to be wrapped in wrap_help ([564c5f0f](https://github.com/kbknapp/clap-rs/commit/564c5f0f1730f4a2c1cdd128664f1a981c31dcd4), closes [#828](https://github.com/kbknapp/clap-rs/issues/828))
* fixes a bug where global args weren't included in the generated completion scripts ([9a1e006e](https://github.com/kbknapp/clap-rs/commit/9a1e006eb75ad5a6057ebd119aa90f7e06c0ace8), closes [#841](https://github.com/kbknapp/clap-rs/issues/841))



<a name="v2.20.2"></a>
### v2.20.2 (2017-02-03)

#### Bug Fixes

*   fixes a critical bug where subcommand settings were being propogated too far ([74648c94](https://github.com/kbknapp/clap-rs/commit/74648c94b893df542bfa5bb595e68c7bb8167e36), closes [#832](https://github.com/kbknapp/clap-rs/issues/832))


#### Improvements

*   adds ArgGroup::multiple to the supported YAML fields for building ArgGroups from YAML ([d8590037](https://github.com/kbknapp/clap-rs/commit/d8590037ce07dafd8cd5b26928aa4a9fd3018288), closes [#840](https://github.com/kbknapp/clap-rs/issues/840))

<a name="v2.20.1"></a>
### v2.20.1 (2017-01-31)

#### Bug Fixes

*   allow final word to be wrapped in wrap_help ([564c5f0f](https://github.com/kbknapp/clap-rs/commit/564c5f0f1730f4a2c1cdd128664f1a981c31dcd4), closes [#828](https://github.com/kbknapp/clap-rs/issues/828))
*   actually show character in debug output ([84d8c547](https://github.com/kbknapp/clap-rs/commit/84d8c5476de95b7f37d61888bc4f13688b712434))
*   include final character in line lenght ([aff4ba18](https://github.com/kbknapp/clap-rs/commit/aff4ba18da8147e1259b04b0bfbc1fcb5c78a3c0))

#### Improvements

*   updates libc and term_size deps for the libc version conflict ([6802ac4a](https://github.com/kbknapp/clap-rs/commit/6802ac4a59c142cda9ec55ca0c45ae5cb9a6ab55))

#### Documentation

*   fix link from app_from_crate! to crate_authors! (#822) ([5b29be9b](https://github.com/kbknapp/clap-rs/commit/5b29be9b073330ab1f7227cdd19fe4aab39d5dcb))
*   fix spelling of "guaranteed" ([4f30a65b](https://github.com/kbknapp/clap-rs/commit/4f30a65b9c03eb09607eb91a929a6396637dc105))

<a name="v2.20.0"></a>

#### New Settings

* **ArgsNegateSubcommands:**  disables args being allowed between subcommands ([5e2af8c9](https://github.com/kbknapp/clap-rs/commit/5e2af8c96adb5ab75fa2d1536237ebcb41869494), closes [#793](https://github.com/kbknapp/clap-rs/issues/793))
* **DontCollapseArgsInUsage:** disables the collapsing of positional args into `[ARGS]` in the usage string  ([c2978afc](https://github.com/kbknapp/clap-rs/commit/c2978afc61fb46d5263ab3b2d87ecde1c9ce1553), closes [#769](https://github.com/kbknapp/clap-rs/issues/769))
* **DisableHelpSubcommand:**  disables building the `help` subcommand  ([a10fc859](https://github.com/kbknapp/clap-rs/commit/a10fc859ee20159fbd9ff4337be59b76467a64f2))
* **AllowMissingPositional:**  allows one to implement `$ prog [optional] <required>` style CLIs where the second postional argument is required, but the first is optional ([1110fdc7](https://github.com/kbknapp/clap-rs/commit/1110fdc7a345c108820dc45783a9bf893fa4c214), closes [#636](https://github.com/kbknapp/clap-rs/issues/636))
* **PropagateGlobalValuesDown:**  automatically propagats global arg's values down through *used* subcommands ([985536c8](https://github.com/kbknapp/clap-rs/commit/985536c8ebcc09af98aac835f42a8072ad58c262), closes [#694](https://github.com/kbknapp/clap-rs/issues/694))

#### API Additions

##### Arg

* **Arg::value_terminator:**  adds the ability to terminate multiple values with a given string or char ([be64ce0c](https://github.com/kbknapp/clap-rs/commit/be64ce0c373efc106384baca3f487ea99fe7b8cf), closes [#782](https://github.com/kbknapp/clap-rs/issues/782))
* **Arg::default_value_if[s]:**  adds new methods for *conditional* default values (such as a particular value from another argument was used) ([eb4010e7](https://github.com/kbknapp/clap-rs/commit/eb4010e7b21724447ef837db11ac441915728f22))
* **Arg::requires_if[s]:**  adds the ability to *conditionally* require additional args (such as if a particular value was used) ([198449d6](https://github.com/kbknapp/clap-rs/commit/198449d64393c265f0bc327aaeac23ec4bb97226))
* **Arg::required_if[s]:**  adds the ability for an arg to be *conditionally* required (i.e. "arg X is only required if arg Y was used with value Z") ([ee9cfddf](https://github.com/kbknapp/clap-rs/commit/ee9cfddf345a6b5ae2af42ba72aa5c89e2ca7f59))
* **Arg::validator_os:**  adds ability to validate values which may contain invalid UTF-8 ([47232498](https://github.com/kbknapp/clap-rs/commit/47232498a813db4f3366ccd3e9faf0bff56433a4))

##### Macros

* **crate_description!:** Uses the `Cargo.toml` description field to fill in the `App::about` method at compile time ([4d9a82db](https://github.com/kbknapp/clap-rs/commit/4d9a82db8e875e9b64a9c2a5c6e22c25afc1279d), closes [#778](https://github.com/kbknapp/clap-rs/issues/778))
* **crate_name!:** Uses the `Cargo.toml` name field to fill in the `App::new` method at compile time ([4d9a82db](https://github.com/kbknapp/clap-rs/commit/4d9a82db8e875e9b64a9c2a5c6e22c25afc1279d), closes [#778](https://github.com/kbknapp/clap-rs/issues/778))
* **app_from_crate!:** Combines `crate_version!`, `crate_name!`, `crate_description!`, and `crate_authors!` into a single macro call to build a default `App` instance from the `Cargo.toml` fields ([4d9a82db](https://github.com/kbknapp/clap-rs/commit/4d9a82db8e875e9b64a9c2a5c6e22c25afc1279d), closes [#778](https://github.com/kbknapp/clap-rs/issues/778))


#### Features

* **no_cargo:**  adds a `no_cargo` feature to disable Cargo-env-var-dependent macros for those *not* using `cargo` to build their crates (#786) ([6fdd2f9d](https://github.com/kbknapp/clap-rs/commit/6fdd2f9d693aaf1118fc61bd362273950703f43d))

#### Bug Fixes

* **Options:**  fixes a critical bug where options weren't forced to have a value ([5a5f2b1e](https://github.com/kbknapp/clap-rs/commit/5a5f2b1e9f598a0d0280ef3e98abbbba2bc41132), closes [#665](https://github.com/kbknapp/clap-rs/issues/665))
*   fixes a bug where calling the help of a subcommand wasn't ignoring required args of parent commands ([d3d34a2b](https://github.com/kbknapp/clap-rs/commit/d3d34a2b51ef31004055b0ab574f766d801c3adf), closes [#789](https://github.com/kbknapp/clap-rs/issues/789))
* **Help Subcommand:**  fixes a bug where the help subcommand couldn't be overriden ([d34ec3e0](https://github.com/kbknapp/clap-rs/commit/d34ec3e032d03e402d8e87af9b2942fe2819b2da), closes [#787](https://github.com/kbknapp/clap-rs/issues/787))
* **Low Index Multiples:**  fixes a bug which caused combinations of LowIndexMultiples and `Arg::allow_hyphen_values` to fail parsing ([26c670ca](https://github.com/kbknapp/clap-rs/commit/26c670ca16d2c80dc26d5c1ce83380ace6357318))

#### Improvements

* **Default Values:**  improves the error message when default values are involved ([1f33de54](https://github.com/kbknapp/clap-rs/commit/1f33de545036e7fd2f80faba251fca009bd519b8), closes [#774](https://github.com/kbknapp/clap-rs/issues/774))
* **YAML:**  adds conditional requirements and conditional default values to YAML ([9a4df327](https://github.com/kbknapp/clap-rs/commit/9a4df327893486adb5558ffefba790c634ccdc6e), closes [#764](https://github.com/kbknapp/clap-rs/issues/764))
*  Support `--("some-arg-name")` syntax for defining long arg names when using `clap_app!` macro ([f41ec962](https://github.com/kbknapp/clap-rs/commit/f41ec962c243a5ffff8b1be1ae2ad63970d3d1d4))
*  Support `("some app name")` syntax for defining app names when using `clap_app!` macro ([9895b671](https://github.com/kbknapp/clap-rs/commit/9895b671cff784f35cf56abcd8270f7c2ba09699), closes [#759](https://github.com/kbknapp/clap-rs/issues/759))
* **Help Wrapping:**  long app names (with spaces), authors, and descriptions are now wrapped appropriately ([ad4691b7](https://github.com/kbknapp/clap-rs/commit/ad4691b71a63e951ace346318238d8834e04ad8a), closes [#777](https://github.com/kbknapp/clap-rs/issues/777))


#### Documentation

* **Conditional Default Values:**  fixes the failing doc tests of Arg::default_value_ifs ([4ef09101](https://github.com/kbknapp/clap-rs/commit/4ef091019c083b4db1a0c13f1c1e95ac363259f2))
* **Conditional Requirements:**  adds docs for Arg::requires_ifs ([7f296e29](https://github.com/kbknapp/clap-rs/commit/7f296e29db7d9036e76e5dbcc9c8b20dfe7b25bd))
* **README.md:**  fix some typos ([f22c21b4](https://github.com/kbknapp/clap-rs/commit/f22c21b422d5b287d1a1ac183a379ee02eebf54f))
* **src/app/mod.rs:**  fix some typos ([5c9b0d47](https://github.com/kbknapp/clap-rs/commit/5c9b0d47ca78dea285c5b9dec79063d24c3e451a))

<a name="v2.19.3"></a>
### v2.19.3 (2016-12-28)


#### Bug Fixes

*   fixes a bug where calling the help of a subcommand wasn't ignoring required args of parent commands ([a0ee4993](https://github.com/kbknapp/clap-rs/commit/a0ee4993015ea97b06b5bc9f378d8bcb18f1c51c), closes [#789](https://github.com/kbknapp/clap-rs/issues/789))



<a name="v2.19.2"></a>
### v2.19.2 (2016-12-08)

#### Bug Fixes

* **ZSH Completions:**  escapes square brackets in ZSH completions ([7e17d5a3](https://github.com/kbknapp/clap-rs/commit/7e17d5a36b2cc2cc77e7b15796b14d639ed3cbf7), closes [#771](https://github.com/kbknapp/clap-rs/issues/771))

#### Documentation

* **Examples:**  adds subcommand examples ([0e0f3354](https://github.com/kbknapp/clap-rs/commit/0e0f33547a6901425afc1d9fbe19f7ae3832d9a4), closes [#766](https://github.com/kbknapp/clap-rs/issues/766))
* **README.md:**  adds guidance on when to use ~ in version pinning, and clarifies breaking change policy ([591eaefc](https://github.com/kbknapp/clap-rs/commit/591eaefc7319142ba921130e502bb0729feed907), closes [#765](https://github.com/kbknapp/clap-rs/issues/765))



<a name="v2.19.1"></a>
### v2.19.1 (2016-12-01)


#### Bug Fixes

* **Help Messages:**  fixes help message alignment when specific settings are used on options ([cd94b318](https://github.com/kbknapp/clap-rs/commit/cd94b3188d63b63295a319e90e826bca46befcd2), closes [#760](https://github.com/kbknapp/clap-rs/issues/760))

#### Improvements

* **Bash Completion:**  allows bash completion to fall back to traidtional bash completion upon no matching completing function ([b1b16d56](https://github.com/kbknapp/clap-rs/commit/b1b16d56d8fddf819bdbe24b3724bb6a9f3fa613)))


<a name="v2.19.0"></a>
## v2.19.0 (2016-11-21)

#### Features

*   allows specifying AllowLeadingHyphen style values, but only for specific args vice command wide ([c0d70feb](https://github.com/kbknapp/clap-rs/commit/c0d70febad9996a77a54107054daf1914c50d4ef), closes [#742](https://github.com/kbknapp/clap-rs/issues/742))

#### Bug Fixes

* **Required Unless:**  fixes a bug where having required_unless set doesn't work when conflicts are also set ([d20331b6](https://github.com/kbknapp/clap-rs/commit/d20331b6f7940ac3a4e919999f8bb4780875125d), closes [#753](https://github.com/kbknapp/clap-rs/issues/753))
* **ZSH Completions:**  fixes an issue where zsh completions caused panics if there were no subcommands ([49e7cdab](https://github.com/kbknapp/clap-rs/commit/49e7cdab76dd1ccc07221e360f07808ec62648aa), closes [#754](https://github.com/kbknapp/clap-rs/issues/754))

#### Improvements

* **Validators:**  improves the error messages for validators ([65eb3385](https://github.com/kbknapp/clap-rs/commit/65eb33859d3ff53e7d3277f02a9d3fd9038a9dfb), closes [#744](https://github.com/kbknapp/clap-rs/issues/744))

#### Documentation

*   updates the docs landing page ([01e1e33f](https://github.com/kbknapp/clap-rs/commit/01e1e33f377934099a4a725fab5cd6c5ff50eaa2))
*   adds the macro version back to the readme ([45eb9bf1](https://github.com/kbknapp/clap-rs/commit/45eb9bf130329c3f3853aba0342c2fe3c64ff80f))
*   fix broken docs links ([808e7cee](https://github.com/kbknapp/clap-rs/commit/808e7ceeb86d4a319bdc270f51c23a64621dbfb3))
* **Compatibility Policy:**  adds an official compatibility policy to ([760d66dc](https://github.com/kbknapp/clap-rs/commit/760d66dc17310b357f257776624151da933cd25d), closes [#740](https://github.com/kbknapp/clap-rs/issues/740))
* **Contributing:**  updates the readme to improve the readability and contributing sections ([eb51316c](https://github.com/kbknapp/clap-rs/commit/eb51316cdfdc7258d287ba13b67ef2f42bd2b8f6))

<a name="v2.18.0"></a>
## v2.18.0 (2016-11-05)


#### Features

* **Completions:**  adds completion support for PowerShell. ([cff82c88](https://github.com/kbknapp/clap-rs/commit/cff82c880e21064fca63351507b80350df6caadf), closes [#729](https://github.com/kbknapp/clap-rs/issues/729))



<a name="v2.17.1"></a>
### v2.17.1 (2016-11-02)


#### Bug Fixes

* **Low Index Multiples:**  fixes a bug where using low index multiples was propagated to subcommands ([33924e88](https://github.com/kbknapp/clap-rs/commit/33924e884461983c4e6b5ea1330fecc769a4ade7), closes [#725](https://github.com/kbknapp/clap-rs/issues/725))



<a name="v2.17.0"></a>
## v2.17.0 (2016-11-01)


#### Features

* **Positional Args:**  allows specifying the second to last positional argument as multiple(true) ([1ced2a74](https://github.com/kbknapp/clap-rs/commit/1ced2a7433ea8937a1b260ea65d708f32ca7c95e), closes [#725](https://github.com/kbknapp/clap-rs/issues/725))



<a name="v2.16.4"></a>
### v2.16.4 (2016-10-31)


#### Improvements

* **Error Output:**  conflicting errors are now symetrical, meaning more consistent and less confusing ([3d37001d](https://github.com/kbknapp/clap-rs/commit/3d37001d1dc647d73cc597ff172f1072d4beb80d), closes [#718](https://github.com/kbknapp/clap-rs/issues/718))

#### Documentation

*   Fix typo in example `13a_enum_values_automatic` ([c22fbc07](https://github.com/kbknapp/clap-rs/commit/c22fbc07356e556ffb5d1a79ec04597d149b915e))
* **README.md:**  fixes failing yaml example (#715) ([21fba9e6](https://github.com/kbknapp/clap-rs/commit/21fba9e6cd8c163012999cd0ce271ec8780c5695))

#### Bug Fixes

* **ZSH Completions:**  fixes bug that caused panic on subcommands with aliases ([5c70e1a0](https://github.com/kbknapp/clap-rs/commit/5c70e1a01bc977e44c10015d18bb8e215c32dfc8), closes [#714](https://github.com/kbknapp/clap-rs/issues/714))
* **debug:**  fixes the debug feature (#716) ([6c11ccf4](https://github.com/kbknapp/clap-rs/commit/6c11ccf443d46258d51f7cda33fbcc81e7fe8e90))



<a name="v2.16.3"></a>
### v2.16.3 (2016-10-28)


#### Bug Fixes

*   Derive display order after propagation ([9cb6facf](https://github.com/kbknapp/clap-rs/commit/9cb6facf507aff7cddd124b8c29714d2b0e7bd13), closes [#706](https://github.com/kbknapp/clap-rs/issues/706))
* **yaml-example:**  inconsistent args ([847f7199](https://github.com/kbknapp/clap-rs/commit/847f7199219ead5065561d91d64780d99ae4b587))



<a name="v2.15.1"></a>
### v2.16.2 (2016-10-25)


#### Bug Fixes

* **Fish Completions:**  fixes a bug where single quotes are not escaped ([780b4a18](https://github.com/kbknapp/clap-rs/commit/780b4a18281b6f7f7071e1b9db2290fae653c406), closes [#704](https://github.com/kbknapp/clap-rs/issues/704))


<a name="v2.16.1"></a>
### v2.16.1 (2016-10-24)


#### Bug Fixes

* **Help Message:**  fixes a regression bug where args with multiple(true) threw off alignment ([ebddac79](https://github.com/kbknapp/clap-rs/commit/ebddac791f3ceac193d5ad833b4b734b9643a7af), closes [#702](https://github.com/kbknapp/clap-rs/issues/702))



<a name="v2.16.0"></a>
## v2.16.0 (2016-10-23)


#### Features

* **Completions:**  adds ZSH completion support ([3e36b0ba](https://github.com/kbknapp/clap-rs/commit/3e36b0bac491d3f6194aee14604caf7be26b3d56), closes [#699](https://github.com/kbknapp/clap-rs/issues/699))



<a name="v2.15.0"></a>
## v2.15.0 (2016-10-21)


#### Features

* **AppSettings:**  adds new setting `AppSettings::AllowNegativeNumbers` ([ab064546](https://github.com/kbknapp/clap-rs/commit/ab06454677fb6aa9b9f804644fcca2168b1eaee3), closes [#696](https://github.com/kbknapp/clap-rs/issues/696))

#### Documentation

* **app/settings.rs:**  moves variants to roughly alphabetical order ([9ed4d4d7](https://github.com/kbknapp/clap-rs/commit/9ed4d4d7957a23357aef60081e45639ab9e3905f))


<a name="v2.14.1"></a>
### v2.14.1 (2016-10-20)


#### Documentation

*   Improve documentation around features ([4ee85b95](https://github.com/kbknapp/clap-rs/commit/4ee85b95d2d16708a016a3ba4e6e2c93b89b7fad))
*   reword docs for ErrorKind and app::Settings ([3ccde7a4](https://github.com/kbknapp/clap-rs/commit/3ccde7a4b8f7a2ea8b916a5415c04a8ff4b5cb7a))
*   fix tests that fail when the "suggestions" feature is disabled ([996fc381](https://github.com/kbknapp/clap-rs/commit/996fc381763a48d125c7ea8a58fed057fd0b4ac6))
*   fix the OsString-using doc-tests ([af9e1a39](https://github.com/kbknapp/clap-rs/commit/af9e1a393ce6cdda46a03c8a4f48df222b015a24))
*   tag non-rust code blocks as such instead of ignoring them ([0ba9f4b1](https://github.com/kbknapp/clap-rs/commit/0ba9f4b123f281952581b6dec948f7e51dd22890))
* **ErrorKind:**  improve some errors about subcommands ([9f6217a4](https://github.com/kbknapp/clap-rs/commit/9f6217a424da823343d7b801b9c350dee3cd1906))
* **yaml:**  make sure the doc-tests don't fail before "missing file" ([8c0f5551](https://github.com/kbknapp/clap-rs/commit/8c0f55516f4910c78c9f8a2bdbd822729574f95b))

#### Improvements

*   Stabilize clap_app! ([cd516006](https://github.com/kbknapp/clap-rs/commit/cd516006e35c37b005f329338560a0a53d1f3e00))
* **with_defaults:**  Deprecate App::with_defaults() ([26085409](https://github.com/kbknapp/clap-rs/commit/2608540940c8bb66e517b65706bc7dea55510682), closes [#638](https://github.com/kbknapp/clap-rs/issues/638))

#### Bug Fixes

*   fixes a bug that made determining when to auto-wrap long help messages inconsistent ([468baadb](https://github.com/kbknapp/clap-rs/commit/468baadb8398fc1d37897b0c49374aef4cf97dca), closes [#688](https://github.com/kbknapp/clap-rs/issues/688))
* **Completions:**  fish completions for nested subcommands ([a61eaf8a](https://github.com/kbknapp/clap-rs/commit/a61eaf8aade76cfe90ccc0f7125751ebf60e3254))
* **features:**  Make lints not enable other nightly-requiring features ([835f75e3](https://github.com/kbknapp/clap-rs/commit/835f75e3ba20999117363ed9f916464d777f36ef))



<a name="v2.14.0"></a>
## v2.14.0 (2016-10-05)


#### Features

* **arg_aliases:**  Ability to alias arguments ([33b5f6ef](https://github.com/kbknapp/clap-rs/commit/33b5f6ef2c9612ecabb31f96b824793e46bfd3dd), closes [#669](https://github.com/kbknapp/clap-rs/issues/669))
* **flag_aliases:**  Ability to alias flags ([40d6dac9](https://github.com/kbknapp/clap-rs/commit/40d6dac973927dded6ab423481634ef47ee7bfd7))

#### Bug Fixes

* **UsageParser:**  Handle non-ascii names / options. ([1d6a7c6e](https://github.com/kbknapp/clap-rs/commit/1d6a7c6e7e6aadc527346aa822f19d8587f714f3), closes [#664](https://github.com/kbknapp/clap-rs/issues/664))

#### Documentation

*   typo ([bac417fa](https://github.com/kbknapp/clap-rs/commit/bac417fa1cea3d32308334c7cccfcf54546cd9d8))


<a name="v2.13.0"></a>
## v2.13.0 (2016-09-18)


#### Documentation

*   updates README.md with new website information and updated video tutorials info ([0c19c580](https://github.com/kbknapp/clap-rs/commit/0c19c580cf50f1b82ff32f70b36708ae2bcac132))
*   updates the docs about removing implicit value_delimiter(true) ([c81bc722](https://github.com/kbknapp/clap-rs/commit/c81bc722ebb8a86d22be89b5aec98df9fe222a08))
* **Default Values:**  adds better examples on using default values ([57a8d9ab](https://github.com/kbknapp/clap-rs/commit/57a8d9abb2f973c235a8a14f8fc031673d7a7460), closes [#418](https://github.com/kbknapp/clap-rs/issues/418))

#### Bug Fixes

* **Value Delimiters:**  fixes the confusion around implicitly setting value delimiters. (default is now `false`) ([09d4d0a9](https://github.com/kbknapp/clap-rs/commit/09d4d0a9038d7ce2df55c2aec95e16f36189fcee), closes [#666](https://github.com/kbknapp/clap-rs/issues/666))



<a name="v2.12.1"></a>
### v2.12.1 (2016-09-13)


#### Bug Fixes

* **Help Wrapping:**  fixes a regression-bug where the old {n} newline char stopped working ([92ac353b](https://github.com/kbknapp/clap-rs/commit/92ac353b48b7caa2511ad2a046d94da93c236cf6), closes [#661](https://github.com/kbknapp/clap-rs/issues/661))



<a name="v2.12.0"></a>
## v2.12.0 (2016-09-13)


#### Features

* **Help:**  adds ability to hide the possible values on a per argument basis ([9151ef73](https://github.com/kbknapp/clap-rs/commit/9151ef739871f2e74910c342299c0de196b95dec), closes [#640](https://github.com/kbknapp/clap-rs/issues/640))
* **help:**  allow for limiting detected terminal width ([a43e28af](https://github.com/kbknapp/clap-rs/commit/a43e28af85c9a9deaedd5ef735f4f13008daab29), closes [#653](https://github.com/kbknapp/clap-rs/issues/653))

#### Documentation

* **Help Wrapping:**  removes the verbiage about using `'{n}'` to insert newlines in help text ([c5a2b352](https://github.com/kbknapp/clap-rs/commit/c5a2b352ca600f5b802290ad945731066cd53611))
* **Value Delimiters:**  updates the docs for the Arg::multiple method WRT value delimiters and default settings ([f9d17a06](https://github.com/kbknapp/clap-rs/commit/f9d17a060aa53f10d0a6e1a7eed5d989d1a59533))
* **appsettings:**  Document AppSetting::DisableVersion ([94501965](https://github.com/kbknapp/clap-rs/commit/945019654d2ca67eb2b1d6014fdf80b84d528d30), closes [#589](https://github.com/kbknapp/clap-rs/issues/589))

#### Bug Fixes

* **AllowLeadingHyphen:**  fixes a bug where valid args aren't recognized with this setting ([a9699e4d](https://github.com/kbknapp/clap-rs/commit/a9699e4d7cdc9a06e73b845933ff1fe6d76f016a), closes [#588](https://github.com/kbknapp/clap-rs/issues/588))

#### Improvements

* **Help Wrapping:**
  *  clap now ignores hard newlines in help messages and properly re-aligns text, but still wraps if the term width is too small ([c7678523](https://github.com/kbknapp/clap-rs/commit/c76785239fd42adc8ca04f9202b6fec615aa9f14), closes [#617](https://github.com/kbknapp/clap-rs/issues/617))
  *  makes some minor changes to when next line help is automatically used ([01cae799](https://github.com/kbknapp/clap-rs/commit/01cae7990a33167ac35103fb36c811b4fe6eb98f))
* **Value Delimiters:**  changes the default value delimiter rules ([f9e69254](https://github.com/kbknapp/clap-rs/commit/f9e692548e8c94de15f909432de301407d6bb834), closes [#655](https://github.com/kbknapp/clap-rs/issues/655))
* **YAML:**  supports setting Arg::require_delimiter from YAML ([b9b55a39](https://github.com/kbknapp/clap-rs/commit/b9b55a39dfebcdbdc05dca2692927e503db50816))

#### Performance

* **help:**  fix redundant contains() checks ([a8afed74](https://github.com/kbknapp/clap-rs/commit/a8afed7428bf0733f8e93bb11ad6c00d9e970fcc))



<a name="v2.11.3"></a>
### v2.11.3 (2016-09-07)


#### Documentation

* **Help Wrapping:**  removes the verbiage about using `'{n}'` to insert newlines in help text ([c5a2b352](https://github.com/kbknapp/clap-rs/commit/c5a2b352ca600f5b802290ad945731066cd53611))

#### Improvements

* **Help Wrapping:**
  *  clap now ignores hard newlines in help messages and properly re-aligns text, but still wraps if the term width is too small ([c7678523](https://github.com/kbknapp/clap-rs/commit/c76785239fd42adc8ca04f9202b6fec615aa9f14), closes [#617](https://github.com/kbknapp/clap-rs/issues/617))
  *  makes some minor changes to when next line help is automatically used ([01cae799](https://github.com/kbknapp/clap-rs/commit/01cae7990a33167ac35103fb36c811b4fe6eb98f))
* **YAML:**  supports setting Arg::require_delimiter from YAML ([b9b55a39](https://github.com/kbknapp/clap-rs/commit/b9b55a39dfebcdbdc05dca2692927e503db50816))




<a name="v2.11.2"></a>
### v2.11.2 (2016-09-06)

#### Improvements

* **Help Wrapping:**  makes some minor changes to when next line help is automatically used ([5658b117](https://github.com/kbknapp/clap-rs/commit/5658b117aec3e03adff9c8c52a4c4bc1fcb4e1ff))


<a name="v2.11.1"></a>
### v2.11.1 (2016-09-05)


#### Bug Fixes

* **Settings:**  fixes an issue where settings weren't propogated down through grand-child subcommands ([b3efc107](https://github.com/kbknapp/clap-rs/commit/b3efc107515d78517b20798ff3890b8a2b04498e), closes [#638](https://github.com/kbknapp/clap-rs/issues/638))

#### Features

* **Errors:**  Errors with custom description ([58512f2f](https://github.com/kbknapp/clap-rs/commit/58512f2fcb430745f1ee6ee8f1c67f62dc216c73))

#### Improvements

* **help:**  use term_size instead of home-grown solution ([fc7327e9](https://github.com/kbknapp/clap-rs/commit/fc7327e9dcf4258ef2baebf0a8714d9c0622855b))



<a name="v2.11.0"></a>
### v2.11.0 (2016-08-28)


#### Bug Fixes

* **Groups:**  fixes some usage strings that contain both args in groups and ones that conflict with each other ([3d782def](https://github.com/kbknapp/clap-rs/commit/3d782def57725e2de26ca5a5bc5cc2e40ddebefb), closes [#616](https://github.com/kbknapp/clap-rs/issues/616))

#### Documentation

*   moves docs to docs.rs ([03209d5e](https://github.com/kbknapp/clap-rs/commit/03209d5e1300906f00bafec1869c2047a92e5071), closes [#634](https://github.com/kbknapp/clap-rs/issues/634))

#### Improvements

* **Completions:**  uses standard conventions for bash completion files, namely '{bin}.bash-completion' ([27f5bbfb](https://github.com/kbknapp/clap-rs/commit/27f5bbfbcc9474c2f57c2b92b1feb898ae46ee70), closes [#567](https://github.com/kbknapp/clap-rs/issues/567))
* **Help:**  automatically moves help text to the next line and wraps when term width is determined to be too small, or help text is too long ([150964c4](https://github.com/kbknapp/clap-rs/commit/150964c4e7124d54476c9d9b4b3f2406f0fd00e5), closes [#597](https://github.com/kbknapp/clap-rs/issues/597))
* **YAML Errors:**  vastly improves error messages when using YAML ([f43b7c65](https://github.com/kbknapp/clap-rs/commit/f43b7c65941c53adc0616b8646a21dc255862eb2), closes [#574](https://github.com/kbknapp/clap-rs/issues/574))

#### Features

*   adds App::with_defaults to automatically use crate_authors! and crate_version! macros ([5520bb01](https://github.com/kbknapp/clap-rs/commit/5520bb012c127dfd299fd55699443c744d8dcd5b), closes [#600](https://github.com/kbknapp/clap-rs/issues/600))



<a name="v2.10.4"></a>
### v2.10.4 (2016-08-25)


#### Bug Fixes

* **Help Wrapping:**  fixes a bug where help is wrapped incorrectly and causing a panic with some non-English characters ([d0b442c7](https://github.com/kbknapp/clap-rs/commit/d0b442c7beeecac9764406bc3bd171ced0b8825e), closes [#626](https://github.com/kbknapp/clap-rs/issues/626))



<a name="v2.10.3"></a>
### v2.10.3 (2016-08-25)

#### Features

* **Help:**  adds new short hand way to use source formatting and ignore term width in help messages ([7dfdaf20](https://github.com/kbknapp/clap-rs/commit/7dfdaf200ebb5c431351a045b48f5e0f0d3f31db), closes [#625](https://github.com/kbknapp/clap-rs/issues/625))

#### Documentation

* **Term Width:**  adds details about set_term_width(0) ([00b8205d](https://github.com/kbknapp/clap-rs/commit/00b8205d22639d1b54b9c453c55c785aace52cb2))

#### Bug Fixes

* **Unicode:**  fixes two bugs where non-English characters were stripped or caused a panic with help wrapping ([763a5c92](https://github.com/kbknapp/clap-rs/commit/763a5c920e23efc74d190af0cb8b5dd714b2d67a), closes [#626](https://github.com/kbknapp/clap-rs/issues/626))



<a name="v2.10.2"></a>
### v2.10.2 (2016-08-22)


#### Bug Fixes

*   fixes a bug where the help is printed twice ([a643fb28](https://github.com/kbknapp/clap-rs/commit/a643fb283acd9905dc727c4579c5c9fa2ceaa7e7), closes [#623](https://github.com/kbknapp/clap-rs/issues/623))



<a name="v2.10.1"></a>
### v2.10.1 (2016-08-21)


#### Bug Fixes

* **Help Subcommand:**  fixes misleading usage string when using multi-level subcommmands ([e203515e](https://github.com/kbknapp/clap-rs/commit/e203515e3ac495b405dbba4f78fb6af148fd282e), closes [#618](https://github.com/kbknapp/clap-rs/issues/618))

#### Features

* **YAML:**  allows using lists or single values with arg declarations ([9ade2cd4](https://github.com/kbknapp/clap-rs/commit/9ade2cd4b268d6d7fe828319ce6a523c641b9c38), closes [#614](https://github.com/kbknapp/clap-rs/issues/614), [#613](https://github.com/kbknapp/clap-rs/issues/613))



<a name="v2.10.0"></a>
## v2.10.0 (2016-07-29)


#### Features

* **Completions:**  one can generate a basic fish completions script at compile time ([1979d2f2](https://github.com/kbknapp/clap-rs/commit/1979d2f2f3216e57d02a97e624a8a8f6cf867ed9))

#### Bug Fixes

* **parser:**  preserve external subcommand name ([875df243](https://github.com/kbknapp/clap-rs/commit/875df24316c266920a073c13bbefbf546bc1f635))

#### Breaking Changes

* **parser:**  preserve external subcommand name ([875df243](https://github.com/kbknapp/clap-rs/commit/875df24316c266920a073c13bbefbf546bc1f635))

#### Documentation

* **YAML:**  fixes example 17's incorrect reference to arg_groups instead of groups ([b6c99e13](https://github.com/kbknapp/clap-rs/commit/b6c99e1377f918e78c16c8faced70a71607da931), closes [#601](https://github.com/kbknapp/clap-rs/issues/601))



<a name="2.9.3"></a>
### 2.9.3 (2016-07-24)


#### Bug Fixes

*   fixes bug where only first arg in list of required_unless_one is recognized ([1fc3b55b](https://github.com/kbknapp/clap-rs/commit/1fc3b55bd6c8653b02e7c4253749c6b77737d2ac), closes [#575](https://github.com/kbknapp/clap-rs/issues/575))
* **Settings:**  fixes typo subcommandsrequired->subcommandrequired ([fc72cdf5](https://github.com/kbknapp/clap-rs/commit/fc72cdf591d30f5d9375d0b5cc2a2ff3e812f9f6), closes [#593](https://github.com/kbknapp/clap-rs/issues/593))

#### Features

* **Completions:**  adds the ability to generate completions to io::Write object ([9f62cf73](https://github.com/kbknapp/clap-rs/commit/9f62cf7378ba5acb5ce8c5bac89b4aa60c30755f))
* **Settings:**  Add unset_setting and unset_settings fns to App (#598) ([0ceba231](https://github.com/kbknapp/clap-rs/commit/0ceba231c6767cd6d88fdb1feeeea41deadf77ff), closes [#590](https://github.com/kbknapp/clap-rs/issues/590))


<a name="2.9.2"></a>
### 2.9.2 (2016-07-03)


#### Documentation

* **Completions:**  fixes the formatting of the Cargo.toml excerpt in the completions example ([722f2607](https://github.com/kbknapp/clap-rs/commit/722f2607beaef56b6a0e433db5fd09492d9f028c))

#### Bug Fixes

* **Completions:**  fixes bug where --help and --version short weren't added to the completion list ([e9f2438e](https://github.com/kbknapp/clap-rs/commit/e9f2438e2ce99af0ae570a2eaf541fc7f55b771b), closes [#536](https://github.com/kbknapp/clap-rs/issues/536))



<a name="2.9.1"></a>
### 2.9.1 (2016-07-02)


#### Improvements

* **Completions:**  allows multiple completions to be built by namespacing with bin name ([57484b2d](https://github.com/kbknapp/clap-rs/commit/57484b2daeaac01c1026e8c84efc8bf099e0eb31))


<a name="v2.9.0"></a>
## v2.9.0 (2016-07-01)


#### Documentation

* **Completions:**
  *  fixes some errors in the completion docs ([9b359bf0](https://github.com/kbknapp/clap-rs/commit/9b359bf06255d3dad8f489308044b60a9d1e6a87))
  *  adds documentation for completion scripts ([c6c519e4](https://github.com/kbknapp/clap-rs/commit/c6c519e40efd6c4533a9ef5efe8e74fd150391b7))

#### Features

* **Completions:**
  *  one can now generate a bash completions script at compile time! ([e75b6c7b](https://github.com/kbknapp/clap-rs/commit/e75b6c7b75f729afb9eb1d2a2faf61dca7674634), closes [#376](https://github.com/kbknapp/clap-rs/issues/376))
  *  completions now include aliases to subcommands, including all subcommand options ([0ab9f840](https://github.com/kbknapp/clap-rs/commit/0ab9f84052a8cf65b5551657f46c0c270841e634), closes [#556](https://github.com/kbknapp/clap-rs/issues/556))
  *  completions now continue completing even after first completion ([18fc2e5b](https://github.com/kbknapp/clap-rs/commit/18fc2e5b5af63bf54a94b72cec5e1223d49f4806))
  *  allows matching on possible values in options ([89cc2026](https://github.com/kbknapp/clap-rs/commit/89cc2026ba9ac69cf44c5254360bbf99236d4f89), closes [#557](https://github.com/kbknapp/clap-rs/issues/557))

#### Bug Fixes

* **AllowLeadingHyphen:**  fixes an issue where  isn't ignored like it should be with this setting ([96c24c9a](https://github.com/kbknapp/clap-rs/commit/96c24c9a8fa1f85e06138d3cdd133e51659e19d2), closes [#558](https://github.com/kbknapp/clap-rs/issues/558))

<a name="v2.8.0"></a>
## v2.8.0 (2016-06-30)


#### Features

* **Arg:**  adds new setting `Arg::require_delimiter` which requires val delimiter to parse multiple values ([920b5595](https://github.com/kbknapp/clap-rs/commit/920b5595ed72abfb501ce054ab536067d8df2a66))

#### Bug Fixes

*   Declare term::Winsize as repr(C) ([5d663d90](https://github.com/kbknapp/clap-rs/commit/5d663d905c9829ce6e7a164f1f0896cdd70236dd))

#### Documentation

* **Arg:**  adds docs for ([49af4e38](https://github.com/kbknapp/clap-rs/commit/49af4e38a5dae2ab0a7fc3b4147e2c053d532484))



<a name="v2.7.1"></a>
### v2.7.1 (2016-06-29)


#### Bug Fixes

* **Options:**
  *  options with multiple values and using delimiters no longer parse additional values after a trailing space ([cdc500bd](https://github.com/kbknapp/clap-rs/commit/cdc500bdde6abe238c36ade406ddafc2bafff583))
  *  using options with multiple values and with an = no longer parse args after the trailing space as values ([290f61d0](https://github.com/kbknapp/clap-rs/commit/290f61d07177413cf082ada55526d83405f6d011))



<a name="v2.7.0"></a>
## v2.7.0 (2016-06-28)


#### Documentation

*   fix typos ([43b3d40b](https://github.com/kbknapp/clap-rs/commit/43b3d40b8c38b1571da75af86b5088be96cccec2))
* **ArgGroup:**  vastly improves ArgGroup docs by adding better examples ([9e5f4f5d](https://github.com/kbknapp/clap-rs/commit/9e5f4f5d734d630bca5535c3a0aa4fd4f9db3e39), closes [#534](https://github.com/kbknapp/clap-rs/issues/534))

#### Features

* **ArgGroup:**  one can now specify groups which require AT LEAST one of the args ([33689acc](https://github.com/kbknapp/clap-rs/commit/33689acc689b217a8c0ee439f1b1225590c38355), closes [#533](https://github.com/kbknapp/clap-rs/issues/533))

#### Bug Fixes

* **App:**  using `App::print_help` now prints the same as would have been printed by `--help` or the like ([e84cc018](https://github.com/kbknapp/clap-rs/commit/e84cc01836bbe0527e97de6db9889bd9e0fd6ba1), closes [#536](https://github.com/kbknapp/clap-rs/issues/536))
* **Help:**
  *  prevents invoking <cmd> help help and displaying incorrect help message ([e3d2893f](https://github.com/kbknapp/clap-rs/commit/e3d2893f377942a2d4cf3c6ff04524d0346e6fdb), closes [#538](https://github.com/kbknapp/clap-rs/issues/538))
  *  subcommand help messages requested via <cmd> help <sub> now correctly match <cmd> <sub> --help ([08ad1cff](https://github.com/kbknapp/clap-rs/commit/08ad1cff4fec57224ea957a2891a057b323c01bc), closes [#539](https://github.com/kbknapp/clap-rs/issues/539))

#### Improvements

* **ArgGroup:**  Add multiple ArgGroups per Arg ([902e182f](https://github.com/kbknapp/clap-rs/commit/902e182f7a58aff11ff01e0a452abcdbdb2262aa), closes [#426](https://github.com/kbknapp/clap-rs/issues/426))
* **Usage Strings:**  `[FLAGS]` and `[ARGS]` are no longer blindly added to usage strings ([9b2e45b1](https://github.com/kbknapp/clap-rs/commit/9b2e45b170aff567b038d8b3368880b6046c10c6), closes [#537](https://github.com/kbknapp/clap-rs/issues/537))
* **arg_enum!:**  allows using meta items like repr(C) with arg_enum!s ([edf9b233](https://github.com/kbknapp/clap-rs/commit/edf9b2331c17a2cbcc13f961add4c55c2778e773), closes [#543](https://github.com/kbknapp/clap-rs/issues/543))



<a name="v2.6.0"></a>
## v2.6.0 (2016-06-14)


#### Improvements

*   removes extra newline from help output ([86e61d19](https://github.com/kbknapp/clap-rs/commit/86e61d19a748fb9870fcf1175308984e51ca1115))
*   allows printing version to any io::Write object ([921f5f79](https://github.com/kbknapp/clap-rs/commit/921f5f7916597f1d028cd4a65bfe76a01c801724))
*   removes extra newline when printing version ([7e2e2cbb](https://github.com/kbknapp/clap-rs/commit/7e2e2cbb4a8a0f050bb8072a376f742fc54b8589))
* **Aliases:**  improves readability of asliases in help messages ([ca511de7](https://github.com/kbknapp/clap-rs/commit/ca511de71f5b8c2ac419f1b188658e8c63b67846), closes [#526](https://github.com/kbknapp/clap-rs/issues/526), [#529](https://github.com/kbknapp/clap-rs/issues/529))
* **Usage Strings:**  improves the default usage string when only a single positional arg is present ([ec86f2da](https://github.com/kbknapp/clap-rs/commit/ec86f2dada1545a63fc72355e22fcdc4c466c215), closes [#518](https://github.com/kbknapp/clap-rs/issues/518))

#### Features

* **Help:**  allows wrapping at specified term width (Even on Windows!) ([1761dc0d](https://github.com/kbknapp/clap-rs/commit/1761dc0d27d0d621229d792be40c36fbf65c3014), closes [#451](https://github.com/kbknapp/clap-rs/issues/451))
* **Settings:**
  *  adds new setting to stop delimiting values with -- or TrailingVarArg ([fc3e0f5a](https://github.com/kbknapp/clap-rs/commit/fc3e0f5afda6d24cdb3c4676614beebe13e1e870), closes [#511](https://github.com/kbknapp/clap-rs/issues/511))
  *  one can now set an AppSetting which is propogated down through child subcommands ([e2341835](https://github.com/kbknapp/clap-rs/commit/e23418351a3b98bf08dfd7744bc14377c70d59ee), closes [#519](https://github.com/kbknapp/clap-rs/issues/519))
* **Subcommands:**  adds support for visible aliases ([7b10e7f8](https://github.com/kbknapp/clap-rs/commit/7b10e7f8937a07fdb8d16a6d8df79ce78d080cd3), closes [#522](https://github.com/kbknapp/clap-rs/issues/522))

#### Bug Fixes

*   fixes bug where args are printed out of order with templates ([05abb534](https://github.com/kbknapp/clap-rs/commit/05abb534864764102031a0d402e64ac65867aa87))
*   fixes bug where one can't override version or help flags ([90d7d6a2](https://github.com/kbknapp/clap-rs/commit/90d7d6a2ea8240122dd9bf8d82d3c4f5ebb5c703), closes [#514](https://github.com/kbknapp/clap-rs/issues/514))
*   fixes issue where before_help wasn't printed ([b3faff60](https://github.com/kbknapp/clap-rs/commit/b3faff6030f76a23f26afcfa6a90169002ed7106))
* **Help:**  `App::before_help` and `App::after_help` now correctly wrap ([1f4da767](https://github.com/kbknapp/clap-rs/commit/1f4da7676e6e71aa8dda799f3eeefad105a47819), closes [#516](https://github.com/kbknapp/clap-rs/issues/516))
* **Settings:**  fixes bug where new color settings couldn't be converted from strs ([706a7c11](https://github.com/kbknapp/clap-rs/commit/706a7c11b0900be594de6d5a3121938eff197602))
* **Subcommands:**  subcommands with aliases now display help of the aliased subcommand ([5354d14b](https://github.com/kbknapp/clap-rs/commit/5354d14b51f189885ba110e01e6b76cca3752992), closes [#521](https://github.com/kbknapp/clap-rs/issues/521))
* **Windows:**  fixes a failing windows build ([01e7dfd6](https://github.com/kbknapp/clap-rs/commit/01e7dfd6c07228c0be6695b3c7bf9370d82860d4))
* **YAML:**  adds missing YAML methods for App and Arg ([e468faf3](https://github.com/kbknapp/clap-rs/commit/e468faf3f05950fd9f72d84b69aa2061e91c6c64), closes [#528](https://github.com/kbknapp/clap-rs/issues/528))



<a name="v2.5.2"></a>
### v2.5.2 (2016-05-31)


#### Improvements

*   removes extra newline from help output ([86e61d19](https://github.com/kbknapp/clap-rs/commit/86e61d19a748fb9870fcf1175308984e51ca1115))
*   allows printing version to any io::Write object ([921f5f79](https://github.com/kbknapp/clap-rs/commit/921f5f7916597f1d028cd4a65bfe76a01c801724))
*   removes extra newline when printing version ([7e2e2cbb](https://github.com/kbknapp/clap-rs/commit/7e2e2cbb4a8a0f050bb8072a376f742fc54b8589))

#### Bug Fixes

*   fixes bug where args are printed out of order with templates ([3935431d](https://github.com/kbknapp/clap-rs/commit/3935431d5633f577c0826ae2142794b301f4b8ca))
*   fixes bug where one can't override version or help flags ([90d7d6a2](https://github.com/kbknapp/clap-rs/commit/90d7d6a2ea8240122dd9bf8d82d3c4f5ebb5c703), closes [#514](https://github.com/kbknapp/clap-rs/issues/514))
*   fixes issue where before_help wasn't printed ([b3faff60](https://github.com/kbknapp/clap-rs/commit/b3faff6030f76a23f26afcfa6a90169002ed7106))

#### Documentation

*   inter-links all types and pages ([3312893d](https://github.com/kbknapp/clap-rs/commit/3312893ddaef3f44d68d8d26ed3d08010be50d97), closes [#505](https://github.com/kbknapp/clap-rs/issues/505))
*   makes all publicly available types viewable in docs ([52ca6505](https://github.com/kbknapp/clap-rs/commit/52ca6505b4fec7b5c2d53d160c072d395eb21da6))

<a name="v2.5.1"></a>
### v2.5.1 (2016-05-11)


#### Bug Fixes

* **Subcommand Aliases**: fixes lifetime issue when setting multiple aliases at once ([ac42f6cf0](https://github.com/kbknapp/clap-rs/commit/ac42f6cf0de6c4920f703807d63061803930b18d))

<a name="v2.5.0"></a>
## v2.5.0 (2016-05-10)


#### Improvements

* **SubCommand Aliases:**  adds feature to yaml configs too ([69592195](https://github.com/kbknapp/clap-rs/commit/695921954dde46dfd483399dcdef482c9dd7f34a))

#### Features

* **SubCommands:**  adds support for subcommand aliases ([66b4dea6](https://github.com/kbknapp/clap-rs/commit/66b4dea65c44d8f77ff522238a9237aed1bcab6d), closes [#469](https://github.com/kbknapp/clap-rs/issues/469))


<a name="v2.4.3"></a>
### v2.4.3 (2016-05-10)


#### Bug Fixes

* **Usage Strings:**
  *  now properly dedups args that are also in groups ([3ca0947c](https://github.com/kbknapp/clap-rs/commit/3ca0947c166b4f8525752255e3a4fa6565eb9689), closes [#498](https://github.com/kbknapp/clap-rs/issues/498))
  *  removes duplicate groups from usage strings ([f574fb8a](https://github.com/kbknapp/clap-rs/commit/f574fb8a7cde4d4a2fa4c4481d59be2d0f135427))

#### Improvements

* **Groups:**  formats positional args in groups in a better way ([fef11154](https://github.com/kbknapp/clap-rs/commit/fef11154fb7430d1cbf04a672aabb366e456a368))
* **Help:**
  *  moves positionals to standard <> formatting ([03dfe5ce](https://github.com/kbknapp/clap-rs/commit/03dfe5ceff1d63f172788ff688567ddad9fe119b))
  *  default help subcommand string has been shortened ([5b7fe8e4](https://github.com/kbknapp/clap-rs/commit/5b7fe8e4161e43ab19e2e5fcf55fbe46791134e9), closes [#494](https://github.com/kbknapp/clap-rs/issues/494))

<a name="v2.4.2"></a>
### v2.4.3 (2016-05-10)

* Ghost Release

<a name="v2.4.1"></a>
### v2.4.3 (2016-05-10)

* Ghost Release

<a name="v2.4.0"></a>
## v2.4.0 (2016-05-02)


#### Features

* **Help:**  adds support for displaying info before help message ([29fbfa3b](https://github.com/kbknapp/clap-rs/commit/29fbfa3b963f2f3ca7704bf5d3e1201531baa373))
* **Required:**  adds allowing args that are required unless certain args are present ([af1f7916](https://github.com/kbknapp/clap-rs/commit/af1f79168390ea7da4074d0d9777de458ea64971))

#### Documentation

*   hides formatting from docs ([cb708093](https://github.com/kbknapp/clap-rs/commit/cb708093a7cd057f08c98b7bd1ed54c2db86ae7e))
* **required_unless:**  adds docs and examples for required_unless ([ca727b52](https://github.com/kbknapp/clap-rs/commit/ca727b52423b9883acd88b2f227b2711bc144573))

#### Bug Fixes

* **Required Args:**  fixes issue where missing required args are sometimes duplicatd in error messages ([3beebd81](https://github.com/kbknapp/clap-rs/commit/3beebd81e7bc2faa4115ac109cf570e512c5477f), closes [#492](https://github.com/kbknapp/clap-rs/issues/492))


<a name="v2.3.0"></a>
## v2.3.0 (2016-04-18)


#### Improvements

* **macros.rs:**  Added write_nspaces macro (a new version of write_spaces) ([9d757e86](https://github.com/kbknapp/clap-rs/commit/9d757e8678e334e5a740ac750c76a9ed4e785cba))
* **parser.rs:**
  *  Provide a way to create a usage string without the USAGE: title ([a91d378b](https://github.com/kbknapp/clap-rs/commit/a91d378ba0c91b5796457f8c6e881b13226ab735))
  *  Make Parser's create_usage public allowing to have function outside the parser to generate the help ([d51945f8](https://github.com/kbknapp/clap-rs/commit/d51945f8b82ebb0963f4f40b384a9e8335783091))
  *  Expose Parser's flags, opts and positionals argument as iterators ([9b23e7ee](https://github.com/kbknapp/clap-rs/commit/9b23e7ee40e51f7a823644c4496be955dc6c9d3a))
* **src/args:**  Exposes argument display order by introducing a new Trait ([1321630e](https://github.com/kbknapp/clap-rs/commit/1321630ef56955f152c73376d4d85cceb0bb4a12))
* **srs/args:**  Added longest_filter to AnyArg trait ([65b3f667](https://github.com/kbknapp/clap-rs/commit/65b3f667532685f854c699ddd264d326599cf7e5))

#### Features

* **Authors Macro:**  adds a crate_authors macro ([38fb59ab](https://github.com/kbknapp/clap-rs/commit/38fb59abf480eb2b6feca269097412f8b00b5b54), closes [#447](https://github.com/kbknapp/clap-rs/issues/447))
* **HELP:**
  *  implements optional colored help messages ([abc8f669](https://github.com/kbknapp/clap-rs/commit/abc8f669c3c8193ffc3a3b0ac6c3ac2198794d4f), closes [#483](https://github.com/kbknapp/clap-rs/issues/483))
  *  Add a Templated Help system. ([81e121ed](https://github.com/kbknapp/clap-rs/commit/81e121edd616f7285593f11120c63bcccae0d23e))

#### Bug Fixes

* **HELP:**  Adjust Help to semantic changes introduced in 6933b84 ([8d23806b](https://github.com/kbknapp/clap-rs/commit/8d23806bd67530ad412c34a1dcdcb1435555573d))

<a name="v2.2.6"></a>
### v2.2.6 (2016-04-11)

#### Bug Fixes

* **Arg Groups**: fixes bug where arg name isn't printed properly ([3019a685](https://github.com/kbknapp/clap-rs/commit/3019a685eee747ccbe6be09ad5dddce0b1d1d4db), closes [#476](https://github.com/kbknapp/clap-rs/issues/476))


<a name="v2.2.5"></a>
### v2.2.5 (2016-04-03)


#### Bug Fixes

* **Empty Values:**  fixes bug where empty values weren't stored ([885d166f](https://github.com/kbknapp/clap-rs/commit/885d166f04eb3fb581898ae5818c6c8032e5a686), closes [#470](https://github.com/kbknapp/clap-rs/issues/470))
* **Help Message:**  fixes bug where arg name is printed twice ([71acf1d5](https://github.com/kbknapp/clap-rs/commit/71acf1d576946658b8bbdb5ae79e6716c43a030f), closes [#472](https://github.com/kbknapp/clap-rs/issues/472))


<a name="v2.2.4"></a>
### v2.2.4 (2016-03-30)


#### Bug Fixes

*   fixes compiling with debug cargo feature ([d4b55450](https://github.com/kbknapp/clap-rs/commit/d4b554509928031ac0808076178075bb21f8c1da))
* **Empty Values:**  fixes bug where empty values weren't stored ([885d166f](https://github.com/kbknapp/clap-rs/commit/885d166f04eb3fb581898ae5818c6c8032e5a686), closes [#470](https://github.com/kbknapp/clap-rs/issues/470))



<a name="v2.2.3"></a>
### v2.2.3 (2016-03-28)


#### Bug Fixes

* **Help Subcommand:**  fixes issue where help and version flags weren't properly displayed ([205b07bf](https://github.com/kbknapp/clap-rs/commit/205b07bf2e6547851f1290f8cd6b169145e144f1), closes [#466](https://github.com/kbknapp/clap-rs/issues/466))

<a name="v2.2.2"></a>
### v2.2.2 (2016-03-27)


#### Bug Fixes

* **Help Message:**  fixes bug with wrapping in the middle of a unicode sequence ([05365ddc](https://github.com/kbknapp/clap-rs/commit/05365ddcc252e4b49e7a75e199d6001a430bd84d), closes [#456](https://github.com/kbknapp/clap-rs/issues/456))
* **Usage Strings:**  fixes small bug where -- would appear needlessly in usage strings ([6933b849](https://github.com/kbknapp/clap-rs/commit/6933b8491c2a7e28cdb61b47dcf10caf33c2f78a), closes [#461](https://github.com/kbknapp/clap-rs/issues/461))


<a name="2.2.1"></a>
### 2.2.1 (2016-03-16)


#### Features

* **Help Message:**  wraps and aligns the help message of subcommands ([813d75d0](https://github.com/kbknapp/clap-rs/commit/813d75d06fbf077c65762608c0fa5e941cfc393c), closes [#452](https://github.com/kbknapp/clap-rs/issues/452))

#### Bug Fixes

* **Help Message:**  fixes a bug where small terminal sizes causing a loop ([1d73b035](https://github.com/kbknapp/clap-rs/commit/1d73b0355236923aeaf6799abc759762ded7e1d0), closes [#453](https://github.com/kbknapp/clap-rs/issues/453))


<a name="v2.2.0"></a>
## v2.2.0 (2016-03-15)


#### Features

* **Help Message:**  can auto wrap and aligning help text to term width ([e36af026](https://github.com/kbknapp/clap-rs/commit/e36af0266635f23e85e951b9088d561e9a5d1bf6), closes [#428](https://github.com/kbknapp/clap-rs/issues/428))
* **Help Subcommand:**  adds support passing additional subcommands to help subcommand ([2c12757b](https://github.com/kbknapp/clap-rs/commit/2c12757bbdf34ce481f3446c074e24c09c2e60fd), closes [#416](https://github.com/kbknapp/clap-rs/issues/416))
* **Opts and Flags:**  adds support for custom ordering in help messages ([9803b51e](https://github.com/kbknapp/clap-rs/commit/9803b51e799904c0befaac457418ee766ccc1ab9))
* **Settings:**  adds support for automatically deriving custom display order of args ([ad86e433](https://github.com/kbknapp/clap-rs/commit/ad86e43334c4f70e86909689a088fb87e26ff95a), closes [#444](https://github.com/kbknapp/clap-rs/issues/444))
* **Subcommands:**  adds support for custom ordering in help messages ([7d2a2ed4](https://github.com/kbknapp/clap-rs/commit/7d2a2ed413f5517d45988eef0765cdcd663b6372), closes [#442](https://github.com/kbknapp/clap-rs/issues/442))

#### Bug Fixes

* **From Usage:**  fixes a bug where adding empty lines werent ignored ([c5c58c86](https://github.com/kbknapp/clap-rs/commit/c5c58c86b9c503d8de19da356a5a5cffb59fbe84))

#### Documentation

* **Groups:**  explains required ArgGroups better ([4ff0205b](https://github.com/kbknapp/clap-rs/commit/4ff0205b85a45151b59bbaf090a89df13438380f), closes [#439](https://github.com/kbknapp/clap-rs/issues/439))

<a name="v2.1.2"></a>
### v2.1.2 (2016-02-24)

#### Bug Fixes

* **Nightly:**  fixes failing nightly build ([d752c170](https://github.com/kbknapp/clap-rs/commit/d752c17029598b19037710f204b7943f0830ae75), closes [#434](https://github.com/kbknapp/clap-rs/issues/434))


<a name="v2.1.1"></a>
### v2.1.1 (2016-02-19)


#### Documentation

* **AppSettings:**  clarifies that AppSettings do not propagate ([3c8db0e9](https://github.com/kbknapp/clap-rs/commit/3c8db0e9be1d24edaad364359513cbb02abb4186), closes [#429](https://github.com/kbknapp/clap-rs/issues/429))
* **Arg Examples:**  adds better examples ([1e79cccc](https://github.com/kbknapp/clap-rs/commit/1e79cccc12937bc0e7cd2aad8e404410798e9fff))

#### Improvements

* **Help:**  adds setting for next line help by arg ([066df748](https://github.com/kbknapp/clap-rs/commit/066df7486e684cf50a8479a356a12ba972c34ce1), closes [#427](https://github.com/kbknapp/clap-rs/issues/427))


<a name="v2.1.0"></a>
## v2.1.0 (2016-02-10)


#### Features

* **Defult Values:**  adds support for default values in args ([73211952](https://github.com/kbknapp/clap-rs/commit/73211952964a79d97b434dd567e6d7d34be7feb5), closes [#418](https://github.com/kbknapp/clap-rs/issues/418))

#### Documentation

* **Default Values:**  adds better examples and notes for default values ([9facd74f](https://github.com/kbknapp/clap-rs/commit/9facd74f843ef3807c5d35259558a344e6c25905))


<a name="v2.0.6"></a>
### v2.0.6 (2016-02-09)


#### Improvements

* **Positional Arguments:**  now displays value name if appropriate ([f0a99916](https://github.com/kbknapp/clap-rs/commit/f0a99916c59ce675515c6dcdfe9a40b130510908), closes [#420](https://github.com/kbknapp/clap-rs/issues/420))


<a name="v2.0.5"></a>
### v2.0.5 (2016-02-05)


#### Bug Fixes

* **Multiple Values:**  fixes bug where number_of_values wasnt respected ([72c387da](https://github.com/kbknapp/clap-rs/commit/72c387da0bb8a6f526f863770f08bb8ca0d3de03))


<a name="v2.0.4"></a>
### v2.0.4 (2016-02-04)


#### Bug Fixes

*   adds support for building ArgGroups from standalone YAML ([fcbc7e12](https://github.com/kbknapp/clap-rs/commit/fcbc7e12f5d7b023b8f30cba8cad28a01cf6cd26))
*   Stop lonely hyphens from causing panic ([85b11468](https://github.com/kbknapp/clap-rs/commit/85b11468b0189d5cc15f1cfac5db40d17a0077dc), closes [#410](https://github.com/kbknapp/clap-rs/issues/410))
* **AppSettings:**  fixes bug where subcmds didn't receive parent ver ([a62e4527](https://github.com/kbknapp/clap-rs/commit/a62e452754b3b0e3ac9a15aa8b5330636229ead1))

<a name="v2.0.3"></a>
### v2.0.3 (2016-02-02)


#### Improvements

* **values:**  adds support for up to u64::max values per arg ([c7abf7d7](https://github.com/kbknapp/clap-rs/commit/c7abf7d7611e317b0d31d97632e3d2e13570947c))
* **occurrences:**  Allow for more than 256 occurrences of an argument. ([3731ddb3](https://github.com/kbknapp/clap-rs/commit/3731ddb361163f3d6b86844362871e48c80fa530))

#### Features

* **AppSettings:**  adds HidePossibleValuesInHelp to skip writing those values ([cdee7a0e](https://github.com/kbknapp/clap-rs/commit/cdee7a0eb2beeec723cb98acfacf03bf629c1da3))

#### Bug Fixes

* **value_t_or_exit:**  fixes typo which causes value_t_or_exit to return a Result ([ee96baff](https://github.com/kbknapp/clap-rs/commit/ee96baffd306cb8d20ddc5575cf739bb1a6354e8))


<a name="v2.0.2"></a>
### v2.0.2 (2016-01-31)


#### Improvements

* **arg_enum:**  enum declared with arg_enum returns [&'static str; #] instead of Vec ([9c4b8a1a](https://github.com/kbknapp/clap-rs/commit/9c4b8a1a6b12949222f17d1074578ad7676b9c0d))

#### Bug Fixes

*   clap_app! should be gated by unstable, not nightly feature ([0c8b84af](https://github.com/kbknapp/clap-rs/commit/0c8b84af6161d5baf683688eafc00874846f83fa))
* **SubCommands:**  fixed where subcmds weren't recognized after mult args ([c19c17a8](https://github.com/kbknapp/clap-rs/commit/c19c17a8850602990e24347aeb4427cf43316223), closes [#405](https://github.com/kbknapp/clap-rs/issues/405))
* **Usage Parser:**  fixes a bug where literal single quotes weren't allowed in help strings ([0bcc7120](https://github.com/kbknapp/clap-rs/commit/0bcc71206478074769e311479b34a9f74fe80f5c), closes [#406](https://github.com/kbknapp/clap-rs/issues/406))


<a name="v2.0.1"></a>
### v2.0.1 (2016-01-30)


#### Bug Fixes

*   fixes cargo features to NOT require nightly with unstable features ([dcbcc60c](https://github.com/kbknapp/clap-rs/commit/dcbcc60c9ba17894be636472ea4b07a82d86a9db), closes [#402](https://github.com/kbknapp/clap-rs/issues/402))


<a name="v2.0.0"></a>
## v2.0.0 (2016-01-28)


#### Improvements

* **From Usage:**  vastly improves the usage parser ([fa3a2f86](https://github.com/kbknapp/clap-rs/commit/fa3a2f86bd674c5eb07128c95098fab7d1437247), closes [#350](https://github.com/kbknapp/clap-rs/issues/350))

#### Features

*   adds support for external subcommands ([177fe5cc](https://github.com/kbknapp/clap-rs/commit/177fe5cce745c2164a8e38c23be4c4460d2d7211), closes [#372](https://github.com/kbknapp/clap-rs/issues/372))
*   adds support values with a leading hyphen ([e4d429b9](https://github.com/kbknapp/clap-rs/commit/e4d429b9d52e95197bd0b572d59efacecf305a59), closes [#385](https://github.com/kbknapp/clap-rs/issues/385))
*   adds support for turning off the value delimiter ([508db850](https://github.com/kbknapp/clap-rs/commit/508db850a87c2e251cf6b6ddead9ad56b29f9e57), closes [#352](https://github.com/kbknapp/clap-rs/issues/352))
*   adds support changing the value delimiter ([dafeae8a](https://github.com/kbknapp/clap-rs/commit/dafeae8a526162640f6a68da434370c64d190889), closes [#353](https://github.com/kbknapp/clap-rs/issues/353))
*   adds support for comma separated values ([e69da6af](https://github.com/kbknapp/clap-rs/commit/e69da6afcd2fe48a3c458ca031db40997f860eda), closes [#348](https://github.com/kbknapp/clap-rs/issues/348))
*   adds support with options with optional values ([4555736c](https://github.com/kbknapp/clap-rs/commit/4555736cad01441dcde4ea84a285227e0844c16e), closes [#367](https://github.com/kbknapp/clap-rs/issues/367))
* **UTF-8:**  adds support for invalid utf8 in values ([c5c59dec](https://github.com/kbknapp/clap-rs/commit/c5c59dec0bc33b86b2e99d30741336f17ec84282), closes [#269](https://github.com/kbknapp/clap-rs/issues/269))
* **v2:**  implementing the base of 2.x ([a3536054](https://github.com/kbknapp/clap-rs/commit/a3536054512ba833533dc56615ce3663d884381c))

#### Bug Fixes

*   fixes nightly build with new lints ([17599195](https://github.com/kbknapp/clap-rs/commit/175991956c37dc83ba9c49396e927a1cb65c5b11))
*   fixes Windows build for 2x release ([674c9b48](https://github.com/kbknapp/clap-rs/commit/674c9b48c7c92079cb180cc650a9e39f34781c32), closes [#392](https://github.com/kbknapp/clap-rs/issues/392))
*   fixes yaml build for 2x base ([adceae64](https://github.com/kbknapp/clap-rs/commit/adceae64c8556d00ab715677377b216f9f468ad7))

#### Documentation

*   updates examples for 2x release ([1303b360](https://github.com/kbknapp/clap-rs/commit/1303b3607468f362ab1b452d5614c1a064dc69b4), closes [#394](https://github.com/kbknapp/clap-rs/issues/394))
*   updates examples for 2x release ([0a011f31](https://github.com/kbknapp/clap-rs/commit/0a011f3142aec338d388a6c8bfe22fa7036021bb), closes [#394](https://github.com/kbknapp/clap-rs/issues/394))
*   updates documentation for v2 release ([8d51724e](https://github.com/kbknapp/clap-rs/commit/8d51724ef73dfde5bb94fb9466bc5463a1cc1502))
*   updating docs for 2x release ([576d0e0e](https://github.com/kbknapp/clap-rs/commit/576d0e0e2c7b8f386589179bbf7419b93abacf1c))
* **README.md:**
  *  updates readme for v2 release ([acaba01a](https://github.com/kbknapp/clap-rs/commit/acaba01a353c12144b9cd9a3ce447400691849b0), closes [#393](https://github.com/kbknapp/clap-rs/issues/393))
  *  fix typo and make documentation conspicuous ([07b9f614](https://github.com/kbknapp/clap-rs/commit/07b9f61495d927f69f7abe6c0d85253f0f4e6107))

#### BREAKING CHANGES

* **Fewer liftimes! Yay!**
 * `App<'a, 'b, 'c, 'd, 'e, 'f>` => `App<'a, 'b>`
 * `Arg<'a, 'b, 'c, 'd, 'e, 'f>` => `Arg<'a, 'b>`
 * `ArgMatches<'a, 'b>` => `ArgMatches<'a>`
* **Simply Renamed**
 * `App::arg_group` => `App::group`
 * `App::arg_groups` => `App::groups`
 * `ArgGroup::add` => `ArgGroup::arg`
 * `ArgGroup::add_all` => `ArgGroup::args`
 * `ClapError` => `Error`
  * struct field `ClapError::error_type` => `Error::kind`
 * `ClapResult` => `Result`
 * `ClapErrorType` => `ErrorKind`
* **Removed Deprecated Functions and Methods**
 * `App::subcommands_negate_reqs`
 * `App::subcommand_required`
 * `App::arg_required_else_help`
 * `App::global_version(bool)`
 * `App::versionless_subcommands`
 * `App::unified_help_messages`
 * `App::wait_on_error`
 * `App::subcommand_required_else_help`
 * `SubCommand::new`
 * `App::error_on_no_subcommand`
 * `Arg::new`
 * `Arg::mutually_excludes`
 * `Arg::mutually_excludes_all`
 * `Arg::mutually_overrides_with`
 * `simple_enum!`
* **Renamed Error Variants**
 * `InvalidUnicode` => `InvalidUtf8`
 * `InvalidArgument` => `UnknownArgument`
* **Usage Parser**
 * Value names can now be specified inline, i.e. `-o, --option <FILE> <FILE2> 'some option which takes two files'`
 * **There is now a priority of order to determine the name** - This is perhaps the biggest breaking change. See the documentation for full details. Prior to this change, the value name took precedence. **Ensure your args are using the proper names (i.e. typically the long or short and NOT the value name) throughout the code**
* `ArgMatches::values_of` returns an `Values` now which implements `Iterator` (should not break any code)
* `crate_version!` returns `&'static str` instead of `String`
* Using the `clap_app!` macro requires compiling with the `unstable` feature because the syntax could change slightly in the future


<a name="v1.5.5"></a>
### v1.5.5 (2016-01-04)


#### Bug Fixes

*   fixes an issue where invalid short args didn't cause an error ([c9bf7e44](https://github.com/kbknapp/clap-rs/commit/c9bf7e4440bd2f9b524ea955311d433c40a7d1e0))
*   prints the name in version and help instead of binary name ([8f3817f6](https://github.com/kbknapp/clap-rs/commit/8f3817f665c0cab6726bc16c56a53b6a61e44448), closes [#368](https://github.com/kbknapp/clap-rs/issues/368))
*   fixes an intentional panic issue discovered via clippy ([ea83a3d4](https://github.com/kbknapp/clap-rs/commit/ea83a3d421ea8856d4cac763942834d108b71406))


<a name="v1.5.4"></a>
### v1.5.4 (2015-12-18)


#### Examples

* **17_yaml:**  conditinonally compile 17_yaml example ([575de089](https://github.com/kbknapp/clap-rs/commit/575de089a3e240c398cb10e6cf5a5c6b68662c01))

#### Improvements

*   clippy improvements ([99cdebc2](https://github.com/kbknapp/clap-rs/commit/99cdebc23da3a45a165f14b27bebeb2ed828a2ce))

#### Bug Fixes


* **errors:**  return correct error type in WrongNumValues error builder ([5ba8ba9d](https://github.com/kbknapp/clap-rs/commit/5ba8ba9dcccdfa74dd1c44260e64b359bbb36be6))
*   ArgRequiredElseHelp setting now takes precedence over missing required args ([faad83fb](https://github.com/kbknapp/clap-rs/commit/faad83fbef6752f3093b6e98fca09a9449b830f4), closes [#362](https://github.com/kbknapp/clap-rs/issues/362))


<a name="v1.5.3"></a>
### v1.5.3 (2015-11-20)


#### Bug Fixes

* **Errors:**  fixes some instances when errors are missing a final newline ([c4d2b171](https://github.com/kbknapp/clap-rs/commit/c4d2b1711994479ad64ee52b6b49d2ceccbf2118))




<a name="v1.5.2"></a>
### v1.5.2 (2015-11-14)


#### Bug Fixes

* **Errors:**  fixes a compiling bug when built on Windows or without the color feature ([a35f7634](https://github.com/kbknapp/clap-rs/commit/a35f76346fe6ecc88dda6a1eb13627186e7ce185))



<a name="v1.5.1"></a>
### v1.5.1 (2015-11-13)


#### Bug Fixes

* **Required Args:**  fixes a bug where required args are not correctly accounted for ([f03b88a9](https://github.com/kbknapp/clap-rs/commit/f03b88a9766b331a63879bcd747687f2e5a2661b), closes [#343](https://github.com/kbknapp/clap-rs/issues/343))



<a name="v1.5.0"></a>
## v1.5.0 (2015-11-13)


#### Bug Fixes

*   fixes a bug with required positional args in usage strings ([c6858f78](https://github.com/kbknapp/clap-rs/commit/c6858f78755f8e860204323c828c8355a066dc83))

#### Documentation

* **FAQ:**  updates readme with slight changes to FAQ ([a4ef0fab](https://github.com/kbknapp/clap-rs/commit/a4ef0fab73c8dc68f1b138965d1340459c113398))

#### Improvements

*   massive errors overhaul ([cdc29175](https://github.com/kbknapp/clap-rs/commit/cdc29175bc9c53e5b4aec86cbc04c1743154dae6))
* **ArgMatcher:**  huge refactor and deduplication of code ([8988853f](https://github.com/kbknapp/clap-rs/commit/8988853fb8825e8f841fde349834cc12cdbad081))
* **Errors:**  errors have been vastly improved ([e59bc0c1](https://github.com/kbknapp/clap-rs/commit/e59bc0c16046db156a88ba71a037db05028e995c))
* **Traits:**  refactoring some configuration into traits ([5800cdec](https://github.com/kbknapp/clap-rs/commit/5800cdec6dce3def4242b9f7bd136308afb19685))

#### Performance

* **App:**
  *  more BTreeMap->Vec, Opts and SubCmds ([bc4495b3](https://github.com/kbknapp/clap-rs/commit/bc4495b32ec752b6c4b29719e831c043ef2a26ce))
  *  changes flags BTreeMap->Vec ([d357640f](https://github.com/kbknapp/clap-rs/commit/d357640fab55e5964fe83efc3c771e53aa3222fd))
  *  removed unneeded BTreeMap ([78971fd6](https://github.com/kbknapp/clap-rs/commit/78971fd68d7dc5c8e6811b4520cdc54e4188f733))
  *  changes BTreeMap to VecMap in some instances ([64b921d0](https://github.com/kbknapp/clap-rs/commit/64b921d087fdd03775c95ba0bcf65d3f5d36f812))
  *  removed excess clones ([ec0089d4](https://github.com/kbknapp/clap-rs/commit/ec0089d42ed715d293fb668d3a90b0db0aa3ec39))



<a name="v1.4.7"></a>
### v1.4.7 (2015-11-03)


#### Documentation

*   Clarify behavior of Arg::multiple with options. ([434f497a](https://github.com/kbknapp/clap-rs/commit/434f497ab6d831f8145cf09278c97ca6ee6c6fe7))
*   Fix typos and improve grammar. ([c1f66b5d](https://github.com/kbknapp/clap-rs/commit/c1f66b5de7b5269fbf8760a005ef8c645edd3229))

#### Bug Fixes

* **Error Status:**  fixes bug where --help and --version return non-zero exit code ([89b51fdf](https://github.com/kbknapp/clap-rs/commit/89b51fdf8b1ab67607567344e2317ff1a757cb12))



<a name="v1.4.6"></a>
### v1.4.6 (2015-10-29)


#### Features

*   allows parsing without a binary name for daemons and interactive CLIs ([aff89d57](https://github.com/kbknapp/clap-rs/commit/aff89d579b5b85c3dc81b64f16d5865299ec39a2), closes [#318](https://github.com/kbknapp/clap-rs/issues/318))

#### Bug Fixes

* **Errors:**  tones down quoting in some error messages ([34ce59ed](https://github.com/kbknapp/clap-rs/commit/34ce59ede53bfa2eef722c74881cdba7419fd9c7), closes [#309](https://github.com/kbknapp/clap-rs/issues/309))
* **Help and Version:**  only builds help and version once ([e3be87cf](https://github.com/kbknapp/clap-rs/commit/e3be87cfc095fc41c9811adcdc6d2b079f237d5e))
* **Option Args:**  fixes bug with args and multiple values ([c9a9548a](https://github.com/kbknapp/clap-rs/commit/c9a9548a8f96cef8a3dd9a980948325fbbc1b91b), closes [#323](https://github.com/kbknapp/clap-rs/issues/323))
* **POSIX Overrides:**  fixes bug where required args are overridden ([40ed2b50](https://github.com/kbknapp/clap-rs/commit/40ed2b50c3a9fe88bfdbaa43cef9fd6493ecaa8e))
* **Safe Matches:**  using 'safe' forms of the get_matches family no longer exit the process ([c47025dc](https://github.com/kbknapp/clap-rs/commit/c47025dca2b3305dea0a0acfdd741b09af0c0d05), closes [#256](https://github.com/kbknapp/clap-rs/issues/256))
* **Versionless SubCommands:**  fixes a bug where the -V flag was needlessly built ([27df8b9d](https://github.com/kbknapp/clap-rs/commit/27df8b9d98d13709dad3929a009f40ebff089a1a), closes [#329](https://github.com/kbknapp/clap-rs/issues/329))

#### Documentation

*   adds comparison in readme ([1a8bf31e](https://github.com/kbknapp/clap-rs/commit/1a8bf31e7a6b87ce48a66af2cde1645b2dd5bc95), closes [#325](https://github.com/kbknapp/clap-rs/issues/325))



<a name="v1.4.5"></a>
### v1.4.5 (2015-10-06)


#### Bug Fixes

*   fixes crash on invalid arg error ([c78ce128](https://github.com/kbknapp/clap-rs/commit/c78ce128ebbe7b8f730815f8176c29d76f4ade8c))



<a name="v1.4.4"></a>
### v1.4.4 (2015-10-06)


#### Documentation

*   clean up some formatting ([b7df92d7](https://github.com/kbknapp/clap-rs/commit/b7df92d7ea25835701dd22ddff984b9749f48a00))
*   move the crate-level docs to top of the lib.rs file ([d7233bf1](https://github.com/kbknapp/clap-rs/commit/d7233bf122dbf80ba8fc79e5641be2df8af10e7a))
*   changes doc comments to rustdoc comments ([34b601be](https://github.com/kbknapp/clap-rs/commit/34b601be5fdde76c1a0859385b359b96d66b8732))
*   fixes panic in 14_groups example ([945b00a0](https://github.com/kbknapp/clap-rs/commit/945b00a0c27714b63bdca48d003fe205fcfdc578), closes [#295](https://github.com/kbknapp/clap-rs/issues/295))
*   avoid suggesting star dependencies. ([d33228f4](https://github.com/kbknapp/clap-rs/commit/d33228f40b5fefb84cf3dd51546bfb340dcd9f5a))
* **Rustdoc:**  adds portions of the readme to main rustdoc page ([6f9ee181](https://github.com/kbknapp/clap-rs/commit/6f9ee181e69d90bd4206290e59d6f3f1e8f0cbb2), closes [#293](https://github.com/kbknapp/clap-rs/issues/293))

#### Bug Fixes

*   grammar error in some conflicting option errors ([e73b07e1](https://github.com/kbknapp/clap-rs/commit/e73b07e19474323ad2260da66abbf6a6d4ecbd4f))
* **Unified Help:**  sorts both flags and options as a unified category ([2a223dad](https://github.com/kbknapp/clap-rs/commit/2a223dad82901fa2e74baad3bfc4c7b94509300f))
* **Usage:**  fixes a bug where required args aren't filtered properly ([72b453dc](https://github.com/kbknapp/clap-rs/commit/72b453dc170af3050bb123d35364f6da77fc06d7), closes [#277](https://github.com/kbknapp/clap-rs/issues/277))
* **Usage Strings:**  fixes a bug ordering of elements in usage strings ([aaf0d6fe](https://github.com/kbknapp/clap-rs/commit/aaf0d6fe7aa2403e76096c16204d254a9ee61ee2), closes [#298](https://github.com/kbknapp/clap-rs/issues/298))

#### Features

*   supports -aValue style options ([0e3733e4](https://github.com/kbknapp/clap-rs/commit/0e3733e4fec2015c2d566a51432dcd92cb69cad3))
* **Trailing VarArg:**  adds opt-in setting for final arg being vararg ([27018b18](https://github.com/kbknapp/clap-rs/commit/27018b1821a4bcd5235cfe92abe71b3c99efc24d), closes [#278](https://github.com/kbknapp/clap-rs/issues/278))



<a name="v1.4.3"></a>
### v1.4.3 (2015-09-30)


#### Features

*   allows accessing arg values by group name ([c92a4b9e](https://github.com/kbknapp/clap-rs/commit/c92a4b9eff2d679957f61c0c41ff404b40d38a91))

#### Documentation

*   use links to examples instead of plain text ([bb4fe237](https://github.com/kbknapp/clap-rs/commit/bb4fe237858535627271465147add537e4556b43))

#### Bug Fixes

* **Help Message:**  required args no longer double list in usage ([1412e639](https://github.com/kbknapp/clap-rs/commit/1412e639e0a79df84936d1101a837f90077d1c83), closes [#277](https://github.com/kbknapp/clap-rs/issues/277))
* **Possible Values:**  possible value validation is restored ([f121ae74](https://github.com/kbknapp/clap-rs/commit/f121ae749f8f4bfe754ef2e8a6dfc286504b5b75), closes [#287](https://github.com/kbknapp/clap-rs/issues/287))



<a name="v1.4.2"></a>
### v1.4.2 (2015-09-23)


#### Bug Fixes

* **Conflicts:**  fixes bug with conflicts not removing required args ([e17fcec5](https://github.com/kbknapp/clap-rs/commit/e17fcec53b3216ad047a13dddc6f740473fad1a1), closes [#271](https://github.com/kbknapp/clap-rs/issues/271))



<a name="v1.4.1"></a>
### v1.4.1 (2015-09-22)


#### Examples

*   add clap_app quick example ([4ba6249c](https://github.com/kbknapp/clap-rs/commit/4ba6249c3cf4d2e083370d1fe4dcc7025282c28a))

#### Features

* **Unicode:**  allows non-panicing on invalid unicode characters ([c5bf7ddc](https://github.com/kbknapp/clap-rs/commit/c5bf7ddc8cfb876ec928a5aaf5591232bbb32e5d))

#### Documentation

*   properly names Examples section for rustdoc ([87ba5445](https://github.com/kbknapp/clap-rs/commit/87ba54451d7ec7b1c9b9ef134f90bbe39e6fac69))
*   fixes various typos and spelling ([f85640f9](https://github.com/kbknapp/clap-rs/commit/f85640f9f6d8fd3821a40e9b8b7a34fabb789d02))
* **Arg:**  unhides fields of the Arg struct ([931aea88](https://github.com/kbknapp/clap-rs/commit/931aea88427edf43a3da90d5a500c1ff2b2c3614))

#### Bug Fixes

*   flush the buffer in App::print_version() ([cbc42a37](https://github.com/kbknapp/clap-rs/commit/cbc42a37d212d84d22b1777d08e584ff191934e7))
*   Macro benchmarks ([13712da1](https://github.com/kbknapp/clap-rs/commit/13712da1d36dc7614eec3a10ad488257ba615751))



<a name="v1.4.0"></a>
## v1.4.0 (2015-09-09)


#### Features

*   allows printing help message by library consumers ([56b95f32](https://github.com/kbknapp/clap-rs/commit/56b95f320875c62dda82cb91b29059671e120ed1))
*   allows defining hidden args and subcmds ([2cab4d03](https://github.com/kbknapp/clap-rs/commit/2cab4d0334ea3c2439a1d4bfca5bf9905c7ea9ac), closes [#231](https://github.com/kbknapp/clap-rs/issues/231))
*   Builder macro to assist with App/Arg/Group/SubCommand building ([443841b0](https://github.com/kbknapp/clap-rs/commit/443841b012a8d795cd5c2bd69ae6e23ef9b16477))
* **Errors:**  allows consumers to write to stderr and exit on error ([1e6403b6](https://github.com/kbknapp/clap-rs/commit/1e6403b6a863574fa3cb6946b1fb58f034e8664c))



<a name="v1.3.2"></a>
### v1.3.2 (2015-09-08)


#### Documentation

*   fixed ErrorKind docs ([dd057843](https://github.com/kbknapp/clap-rs/commit/dd05784327fa070eb6ce5ce89a8507e011d8db94))
* **ErrorKind:**  changed examples content ([b9ca2616](https://github.com/kbknapp/clap-rs/commit/b9ca261634b89613bbf3d98fd74d55cefbb31a8c))

#### Bug Fixes

*   fixes a bug where the help subcommand wasn't overridable ([94003db4](https://github.com/kbknapp/clap-rs/commit/94003db4b5eebe552ca337521c1c001295822745))

#### Features

*   adds abiltiy not consume self when parsing matches and/or exit on help ([94003db4](https://github.com/kbknapp/clap-rs/commit/94003db4b5eebe552ca337521c1c001295822745))
* **App:**  Added ability for users to handle errors themselves ([934e6fbb](https://github.com/kbknapp/clap-rs/commit/934e6fbb643b2385efc23444fe6fce31494dc288))



<a name="v1.3.1"></a>
### v1.3.1 (2015-09-04)


#### Examples

* **17_yaml:**  fixed example ([9b848622](https://github.com/kbknapp/clap-rs/commit/9b848622296c8c5c7b9a39b93ddd41f51df790b5))

#### Performance

*   changes ArgGroup HashSets to Vec ([3cb4a48e](https://github.com/kbknapp/clap-rs/commit/3cb4a48ebd15c20692f4f3a2a924284dc7fd5e10))
*   changes BTreeSet for Vec in some instances ([baab2e3f](https://github.com/kbknapp/clap-rs/commit/baab2e3f4060e811abee14b1654cbcd5cf3b5fea))



<a name="v1.3.0"></a>
## v1.3.0 (2015-09-01)


#### Features

* **YAML:**  allows building a CLI from YAML files ([86cf4c45](https://github.com/kbknapp/clap-rs/commit/86cf4c45626a36b8115446952f9069f73c1debc3))
* **ArgGroups:**  adds support for building ArgGroups from yaml ([ecf88665](https://github.com/kbknapp/clap-rs/commit/ecf88665cbff367018b29161a1b75d44a212707d))
* **Subcommands:**  adds support for subcommands from yaml ([e415cf78](https://github.com/kbknapp/clap-rs/commit/e415cf78ba916052d118a8648deba2b9c16b1530))

#### Documentation

* **YAML:**  adds examples for using YAML to build a CLI ([ab41d7f3](https://github.com/kbknapp/clap-rs/commit/ab41d7f38219544750e6e1426076dc498073191b))
* **Args from YAML:**  fixes doc examples ([19b348a1](https://github.com/kbknapp/clap-rs/commit/19b348a10050404cd93888dbbbe4f396681b67d0))
* **Examples:**  adds better usage examples instead of having unused variables ([8cbacd88](https://github.com/kbknapp/clap-rs/commit/8cbacd8883004fe71a8ea036ec4391c7dd8efe94))

#### Examples

*   Add AppSettings example ([12705079](https://github.com/kbknapp/clap-rs/commit/12705079ca96a709b4dd94f7ddd20a833b26838c))

#### Bug Fixes

* **Unified Help Messages:**  fixes a crash from this setting and no opts ([169ffec1](https://github.com/kbknapp/clap-rs/commit/169ffec1003d58d105d7ef2585b3425e57980000), closes [#210](https://github.com/kbknapp/clap-rs/issues/210))



<a name="v1.2.5"></a>
### v1.2.5 (2015-08-27)


#### Examples

*   add custom validator example ([b9997d1f](https://github.com/kbknapp/clap-rs/commit/b9997d1fca74d4d8f93971f2a01bdf9798f913d5))
*   fix indentation ([d4f1b740](https://github.com/kbknapp/clap-rs/commit/d4f1b740ede410fd2528b9ecd89592c2fd8b1e20))

#### Features

* **Args:**  allows opts and args to define a name for help and usage msgs ([ad962ec4](https://github.com/kbknapp/clap-rs/commit/ad962ec478da999c7dba0afdb84c266f4d09b1bd))



<a name="v1.2.4"></a>
### v1.2.4 (2015-08-26)


#### Bug Fixes

* **Possible Values:**  fixes a bug where suggestions arent made when using --long=value format ([3d5e9a6c](https://github.com/kbknapp/clap-rs/commit/3d5e9a6cedb26668839b481c9978e2fbbab8be6f), closes [#192](https://github.com/kbknapp/clap-rs/issues/192))



<a name="v1.2.3"></a>
### v1.2.3 (2015-08-24)


#### Bug Fixes

* **App, Args:**  fixed subcommand reqs negation ([b41afa8c](https://github.com/kbknapp/clap-rs/commit/b41afa8c3ded3d1be12f7a2f8ea06cc44afc9458), closes [#188](https://github.com/kbknapp/clap-rs/issues/188))



<a name="v1.2.2"></a>
### v1.2.2 (2015-08-23)


#### Bug Fixes

*   fixed confusing error message, also added test for it ([fc7a31a7](https://github.com/kbknapp/clap-rs/commit/fc7a31a745efbf1768ee2c62cd3bb72bfe30c708))
* **App:**  fixed requirmets overriding ([9c135eb7](https://github.com/kbknapp/clap-rs/commit/9c135eb790fa16183e5bdb2009ddc3cf9e25f99f))



<a name="v1.2.1"></a>
### v1.2.1 (2015-08-20)


#### Documentation

* **README.md:**  updates for new features ([16cf9245](https://github.com/kbknapp/clap-rs/commit/16cf9245fb5fc4cf6face898e358368bf9961cbb))

#### Features

*   implements posix compatible conflicts for long args ([8c2d48ac](https://github.com/kbknapp/clap-rs/commit/8c2d48acf5473feebd721a9049a9c9b7051e70f9))
*   added overrides to support conflicts in POSIX compatible manner ([0b916a00](https://github.com/kbknapp/clap-rs/commit/0b916a00de26f6941538f6bc5f3365fa302083c1))
* **Args:**  allows defining POSIX compatible argument conflicts ([d715646e](https://github.com/kbknapp/clap-rs/commit/d715646e69759ccd95e01f49b04f489827ecf502))

#### Bug Fixes

*   fixed links in cargo and license buttons ([6d9837ad](https://github.com/kbknapp/clap-rs/commit/6d9837ad9a9e006117cd7372fdc60f9a3889c7e2))

#### Performance

* **Args and Apps:**  changes HashSet->Vec in some instances for increased performance ([d0c3b379](https://github.com/kbknapp/clap-rs/commit/d0c3b379700757e0a9b0c40af709f8af1f5b4949))



<a name="v1.2.0"></a>
### v1.2.0 (2015-08-15)


#### Bug Fixes

*   fixed misspell and enum name ([7df170d7](https://github.com/kbknapp/clap-rs/commit/7df170d7f4ecff06608317655d1e0c4298f62076))
*   fixed use for clap crate ([dc3ada73](https://github.com/kbknapp/clap-rs/commit/dc3ada738667d4b689678f79d14251ee82004ece))

#### Documentation

*   updates docs for new features ([03496547](https://github.com/kbknapp/clap-rs/commit/034965471782d872ca495045b58d34b31807c5b1))
*   fixed docs for previous changes ([ade36778](https://github.com/kbknapp/clap-rs/commit/ade367780c366425de462506d256e0f554ed3b9c))

#### Improvements

* **AppSettings:**  adds ability to add multiple settings at once ([4a00e251](https://github.com/kbknapp/clap-rs/commit/4a00e2510d0ca8d095d5257d51691ba3b61c1374))

#### Features

*   Replace application level settings with enum variants ([618dc4e2](https://github.com/kbknapp/clap-rs/commit/618dc4e2c205bf26bc43146164e65eb1f6b920ed))
* **Args:**  allows for custom argument value validations to be defined ([84ae2ddb](https://github.com/kbknapp/clap-rs/commit/84ae2ddbceda34b5cbda98a6959edaa52fde2e1a), closes [#170](https://github.com/kbknapp/clap-rs/issues/170))



<a name="v1.1.6"></a>
### v1.1.6 (2015-08-01)


#### Bug Fixes

*   fixes two bugs in App when printing newlines in help and subcommands required error ([d63c0136](https://github.com/kbknapp/clap-rs/commit/d63c0136310db9dd2b1c7b4745938311601d8938))



<a name="v1.1.5"></a>
### v1.1.5 (2015-07-29)

#### Performance

*   removes some unneeded allocations ([93e915df](https://github.com/kbknapp/clap-rs/commit/93e915dfe300f7b7d6209ca93323c6a46f89a8c1))

<a name="v1.1.4"></a>
### v1.1.4 (2015-07-20)


#### Improvements

* **Usage Strings**  displays a [--] when it may be helpful ([86c3be85](https://github.com/kbknapp/clap-rs/commit/86c3be85fb6f77f83b5a6d2df40ae60937486984))

#### Bug Fixes

* **Macros**  fixes a typo in a macro generated error message ([c9195c5f](https://github.com/kbknapp/clap-rs/commit/c9195c5f92abb8cd6a37b4f4fbb2f1fee2a8e368))
* **Type Errors**  fixes formatting of error output when failed type parsing ([fe5d95c6](https://github.com/kbknapp/clap-rs/commit/fe5d95c64f3296e6eddcbec0cb8b86659800145f))



<a name="v1.1.3"></a>
### v1.1.3 (2015-07-18)


#### Documentation

*   updates README.md to include lack of color support on Windows ([52f81e17](https://github.com/kbknapp/clap-rs/commit/52f81e17377b18d2bd0f34693b642b7f358998ee))

#### Bug Fixes

*   fixes formatting bug which prevented compiling on windows ([9cb5dceb](https://github.com/kbknapp/clap-rs/commit/9cb5dceb3e5fe5e0e7b24619ff77e5040672b723), closes [#163](https://github.com/kbknapp/clap-rs/issues/163))



<a name="v1.1.2"></a>
### v1.1.2 (2015-07-17)


#### Bug Fixes

*   fixes a bug when parsing multiple {n} newlines inside help strings ([6d214b54](https://github.com/kbknapp/clap-rs/commit/6d214b549a9b7e189a94e5fa2b7c92cc333ca637))



<a name="v1.1.1"></a>
## v1.1.1 (2015-07-17)


#### Bug Fixes

*   fixes a logic bug and allows setting Arg::number_of_values() < 2 ([42b6d1fc](https://github.com/kbknapp/clap-rs/commit/42b6d1fc3c519c92dfb3af15276e7d3b635e6cfe), closes [#161](https://github.com/kbknapp/clap-rs/issues/161))



<a name="v1.1.0"></a>
## v1.1.0 (2015-07-16)


#### Features

*   allows creating unified help messages, a la docopt or getopts ([52bcd892](https://github.com/kbknapp/clap-rs/commit/52bcd892ea51564ce463bc5865acd64f8fe91cb1), closes [#158](https://github.com/kbknapp/clap-rs/issues/158))
*   allows stating all subcommands should *not* have --version flags ([336c476f](https://github.com/kbknapp/clap-rs/commit/336c476f631d512b54ac56fdca6f29ebdc2c00c5), closes [#156](https://github.com/kbknapp/clap-rs/issues/156))
*   allows setting version number to auto-propagate through subcommands ([bc66d3c6](https://github.com/kbknapp/clap-rs/commit/bc66d3c6deedeca62463fff95369ab1cfcdd366b), closes [#157](https://github.com/kbknapp/clap-rs/issues/157))

#### Improvements

* **Help Strings**  properly aligns and handles newlines in long help strings ([f9800a29](https://github.com/kbknapp/clap-rs/commit/f9800a29696dd2cc0b0284bf693b3011831e556f), closes [#145](https://github.com/kbknapp/clap-rs/issues/145))


#### Performance

* **Help Messages**  big performance improvements when printing help messages ([52bcd892](https://github.com/kbknapp/clap-rs/commit/52bcd892ea51564ce463bc5865acd64f8fe91cb1))

#### Documentation

*   updates readme with new features ([8232f7bb](https://github.com/kbknapp/clap-rs/commit/8232f7bb52e88862bc13c3d4f99ee4f56cfe4bc0))
*   fix incorrect code example for `App::subcommand_required` ([8889689d](https://github.com/kbknapp/clap-rs/commit/8889689dc6336ccc45b2c9f2cf8e2e483a639e93))


<a name="v1.0.3"></a>
### v1.0.3 (2015-07-11)


#### Improvements

* **Errors**  writes errors to stderr ([cc76ab8c](https://github.com/kbknapp/clap-rs/commit/cc76ab8c2b77c67b42f4717ded530df7806142cf), closes [#154](https://github.com/kbknapp/clap-rs/issues/154))

#### Documentation

* **README.md**  updates example help message to new format ([0aca29bd](https://github.com/kbknapp/clap-rs/commit/0aca29bd5d6d1a4e9971bdc88d946ffa58606efa))



<a name="v1.0.2"></a>
### v1.0.2 (2015-07-09)


#### Improvements

* **Usage**  re-orders optional arguments and required to natural standard ([dc7e1fce](https://github.com/kbknapp/clap-rs/commit/dc7e1fcea5c85d317018fb201d2a9262249131b4), closes [#147](https://github.com/kbknapp/clap-rs/issues/147))



<a name="v1.0.1"></a>
### v1.0.1 (2015-07-08)


#### Bug Fixes

*   allows empty values when using --long='' syntax ([083f82d3](https://github.com/kbknapp/clap-rs/commit/083f82d333b69720a6ef30074875310921d964d1), closes [#151](https://github.com/kbknapp/clap-rs/issues/151))



<a name="v1.0.0"></a>
## v1.0.0 (2015-07-08)


#### Documentation

* **README.md**  adds new features to what's new list ([938f7f01](https://github.com/kbknapp/clap-rs/commit/938f7f01340f521969376cf4e2e3d9436bca21f7))
* **README.md**  use with_name for subcommands ([28b7e316](https://github.com/kbknapp/clap-rs/commit/28b7e3161fb772e5309042648fe8c3a420645bac))

#### Features

*   args can now be parsed from arbitrary locations, not just std::env::args() ([75312528](https://github.com/kbknapp/clap-rs/commit/753125282b1b9bfff875f1557ce27610edcc59e1))



<a name="v1.0.0"></a>
## v1.0.0-beta (2015-06-30)


#### Features

*   allows waiting for user input on error ([d0da3bdd](https://github.com/kbknapp/clap-rs/commit/d0da3bdd9d1871541907ea9c645322a74d260e07), closes [#140](https://github.com/kbknapp/clap-rs/issues/140))
* **Help**  allows one to fully override the auto-generated help message ([26d5ae3e](https://github.com/kbknapp/clap-rs/commit/26d5ae3e330d1e150811d5b60b2b01a8f8df854e), closes [#141](https://github.com/kbknapp/clap-rs/issues/141))

#### Documentation

*   adds "whats new" section to readme ([ff149a29](https://github.com/kbknapp/clap-rs/commit/ff149a29dd9e179865e6d577cd7dc87c54f8f95c))

#### Improvements

*   removes deprecated functions in prep for 1.0 ([274484df](https://github.com/kbknapp/clap-rs/commit/274484dfd08fff4859cefd7e9bef3b73d3a9cb5f))



<a name="v0.11.0"></a>
## v0.11.0 (2015-06-17) - BREAKING CHANGE


#### Documentation

*   updates docs to new version flag defaults ([ebf442eb](https://github.com/kbknapp/clap-rs/commit/ebf442ebebbcd2ec6bfe2c06566c9d362bccb112))

#### Features

* **Help and Version**  default short for version is now `-V` but can be overridden (only breaks manual documentation) (**BREAKING CHANGE** [eb1d9320](https://github.com/kbknapp/clap-rs/commit/eb1d9320c509c1e4e57d7c7959da82bcfe06ada0))



<a name="v0.10.5"></a>
### v0.10.5 (2015-06-06)


#### Bug Fixes

* **Global Args**  global arguments propogate fully now ([1f377960](https://github.com/kbknapp/clap-rs/commit/1f377960a48c82f54ca5f39eb56bcb393140b046), closes [#137](https://github.com/kbknapp/clap-rs/issues/137))



<a name="v0.10.4"></a>
### v0.10.4 (2015-06-06)


#### Bug Fixes

* **Global Args**  global arguments propogate fully now ([8f2c0160](https://github.com/kbknapp/clap-rs/commit/8f2c0160c8d844daef375a33dbaec7d89de00a00), closes [#137](https://github.com/kbknapp/clap-rs/issues/137))



<a name="v0.10.3"></a>
### v0.10.3 (2015-05-31)


#### Bug Fixes

* **Global Args**  fixes a bug where globals only transfer to one subcommand ([a37842ee](https://github.com/kbknapp/clap-rs/commit/a37842eec1ee3162b86fdbda23420b221cdb1e3b), closes [#135](https://github.com/kbknapp/clap-rs/issues/135))



<a name="v0.10.2"></a>
### v0.10.2 (2015-05-30)


#### Improvements

* **Binary Names**  allows users to override the system determined bin name ([2191fe94](https://github.com/kbknapp/clap-rs/commit/2191fe94bda35771383b52872fb7f5421b178be1), closes [#134](https://github.com/kbknapp/clap-rs/issues/134))

#### Documentation

*   adds contributing guidelines ([6f76bd0a](https://github.com/kbknapp/clap-rs/commit/6f76bd0a07e8b7419b391243ab2d6687cd8a9c5f))



<a name="v0.10.1"></a>
### v0.10.1 (2015-05-26)


#### Features

*   can now specify that an app or subcommand should display help on no args or subcommands ([29ca7b2f](https://github.com/kbknapp/clap-rs/commit/29ca7b2f74376ca0cdb9d8ee3bfa99f7640cc404), closes [#133](https://github.com/kbknapp/clap-rs/issues/133))



<a name="v0.10.0"></a>
## v0.10.0 (2015-05-23)


#### Features

* **Global Args**  allows args that propagate down to child commands ([2bcc6137](https://github.com/kbknapp/clap-rs/commit/2bcc6137a83cb07757771a0afea953e68e692f0b), closes [#131](https://github.com/kbknapp/clap-rs/issues/131))

#### Improvements

* **Colors**  implements more structured colored output ([d6c3ed54](https://github.com/kbknapp/clap-rs/commit/d6c3ed54d21cf7b40d9f130d4280ff5448522fc5), closes [#129](https://github.com/kbknapp/clap-rs/issues/129))

#### Deprecations

* **SubCommand/App**  several methods and functions for stable release ([28b73855](https://github.com/kbknapp/clap-rs/commit/28b73855523ad170544afdb20665db98702fbe70))

#### Documentation

*   updates for deprecations and new features ([743eefe8](https://github.com/kbknapp/clap-rs/commit/743eefe8dd40c1260065ce086d572e9e9358bc4c))



<a name="v0.9.2"></a>
## v0.9.2 (2015-05-20)


#### Bug Fixes

* **help**  allows parent requirements to be ignored with help and version ([52218cc1](https://github.com/kbknapp/clap-rs/commit/52218cc1fdb06a42456c964d98cc2c7ac3432412), closes [#124](https://github.com/kbknapp/clap-rs/issues/124))



<a name="v0.9.1"></a>
## v0.9.1 (2015-05-18)


#### Bug Fixes

* **help**  fixes a bug where requirements are included as program name in help and version ([08ba3f25](https://github.com/kbknapp/clap-rs/commit/08ba3f25cf38b149229ba8b9cb37a5804fe6b789))



<a name="v0.9.0"></a>
## v0.9.0 (2015-05-17)


#### Improvements

* **usage**  usage strings now include parent command requirements ([dd8f21c7](https://github.com/kbknapp/clap-rs/commit/dd8f21c7c15cde348fdcf44fa7c205f0e98d2e4a), closes [#125](https://github.com/kbknapp/clap-rs/issues/125))
* **args**  allows consumer of clap to decide if empty values are allowed or not ([ab4ec609](https://github.com/kbknapp/clap-rs/commit/ab4ec609ccf692b9b72cccef5c9f74f5577e360d), closes [#122](https://github.com/kbknapp/clap-rs/issues/122))

#### Features

* **subcommands**
  *  allows optionally specifying that no subcommand is an error ([7554f238](https://github.com/kbknapp/clap-rs/commit/7554f238fd3afdd60b7e4dcf00ff4a9eccf842c1), closes [#126](https://github.com/kbknapp/clap-rs/issues/126))
  *  subcommands can optionally negate parent requirements ([4a4229f5](https://github.com/kbknapp/clap-rs/commit/4a4229f500e21c350e1ef78dd09ef27559653288), closes [#123](https://github.com/kbknapp/clap-rs/issues/123))



<a name="v0.8.6"></a>
## v0.8.6 (2015-05-17)


#### Bug Fixes

* **args**  `-` can now be parsed as a value for an argument ([bc12e78e](https://github.com/kbknapp/clap-rs/commit/bc12e78eadd7eaf9d008a8469fdd2dfd7990cb5d), closes [#121](https://github.com/kbknapp/clap-rs/issues/121))



<a name="v0.8.5"></a>
## v0.8.5 (2015-05-15)


#### Bug Fixes

* **macros**  makes macro errors consistent with others ([0c264a8c](https://github.com/kbknapp/clap-rs/commit/0c264a8ca57ec1cfdcb74dae79145d766cdc9b97), closes [#118](https://github.com/kbknapp/clap-rs/issues/118))

#### Features

* **macros**
  *  arg_enum! and simple_enum! provide a Vec<&str> of variant names ([30fa87ba](https://github.com/kbknapp/clap-rs/commit/30fa87ba4e0f3189351d8f4f78b72e616a30d0bd), closes [#119](https://github.com/kbknapp/clap-rs/issues/119))
  *  arg_enum! and simple_enum! auto-implement Display ([d1219f0d](https://github.com/kbknapp/clap-rs/commit/d1219f0d1371d872061bd0718057eca4ef47b739), closes [#120](https://github.com/kbknapp/clap-rs/issues/120))



<a name="v0.8.4"></a>
## v0.8.4 (2015-05-12)


#### Bug Fixes

* **suggestions**  --help and --version now get suggestions ([d2b3b1fa](https://github.com/kbknapp/clap-rs/commit/d2b3b1faa0bdc1c5d2350cc4635aba81e02e9d96), closes [#116](https://github.com/kbknapp/clap-rs/issues/116))



<a name="v0.8.3"></a>
## v0.8.3 (2015-05-10)


#### Bug Fixes

* **usage**  groups unfold their members in usage strings ([55d15582](https://github.com/kbknapp/clap-rs/commit/55d155827ea4a6b077a83669701e797ce1ad68f4), closes [#114](https://github.com/kbknapp/clap-rs/issues/114))

#### Performance

* **usage**  removes unneeded allocations ([fd53cd18](https://github.com/kbknapp/clap-rs/commit/fd53cd188555f5c3dc8bc341c5d7eb04b761a70f))



<a name="v0.8.2"></a>
## v0.8.2 (2015-05-08)


#### Bug Fixes

* **usage strings**  positional arguments are presented in index order ([eb0e374e](https://github.com/kbknapp/clap-rs/commit/eb0e374ecf952f1eefbc73113f21e0705936e40b), closes [#112](https://github.com/kbknapp/clap-rs/issues/112))



<a name="v0.8.1"></a>
## v0.8.1 (2015-05-06)


#### Bug Fixes

* **subcommands**  stops parsing multiple values when subcommands are found ([fc79017e](https://github.com/kbknapp/clap-rs/commit/fc79017eced04fd41cc1801331e5054df41fac17), closes [#109](https://github.com/kbknapp/clap-rs/issues/109))

#### Improvements

* **color**  reduces color in error messages ([aab44cca](https://github.com/kbknapp/clap-rs/commit/aab44cca6352f47e280c296e50c535f5d752dd46), closes [#110](https://github.com/kbknapp/clap-rs/issues/110))
* **suggestions**  adds suggested arguments to usage strings ([99447414](https://github.com/kbknapp/clap-rs/commit/994474146e9fb8b701af773a52da71553d74d4b7))



<a name="v0.8.0"></a>
## v0.8.0 (2015-05-06)


#### Bug Fixes

* **did-you-mean**  for review ([0535cfb0](https://github.com/kbknapp/clap-rs/commit/0535cfb0c711331568b4de8080eeef80bd254b68))
* **Positional**  positionals were ignored if they matched a subcmd, even after '--' ([90e7b081](https://github.com/kbknapp/clap-rs/commit/90e7b0818741668b47cbe3becd029bab588e3553))
* **help**  fixes bug where space between arg and help is too long ([632fb115](https://github.com/kbknapp/clap-rs/commit/632fb11514c504999ea86bdce47cdd34f8ebf646))

#### Features

* **from_usage**  adds ability to add value names or num of vals in usage string ([3d581976](https://github.com/kbknapp/clap-rs/commit/3d58197674ed7886ca315efb76e411608a327501), closes [#98](https://github.com/kbknapp/clap-rs/issues/98))
* **did-you-mean**
  *  gate it behind 'suggestions' ([c0e38351](https://github.com/kbknapp/clap-rs/commit/c0e383515d01bdd5ca459af9c2f7e2cf49e2488b))
  *  for possible values ([1cc2deb2](https://github.com/kbknapp/clap-rs/commit/1cc2deb29158e0e4e8b434e4ce26b3d819301a7d))
  *  for long flags (i.e. --long) ([52a0b850](https://github.com/kbknapp/clap-rs/commit/52a0b8505c99354bdf5fd1cd256cf41197ac2d81))
  *  for subcommands ([06e869b5](https://github.com/kbknapp/clap-rs/commit/06e869b5180258047ed3c60ba099de818dd25fff))
* **Flags**  adds sugestions functionality ([8745071c](https://github.com/kbknapp/clap-rs/commit/8745071c3257dd327c497013516f12a823df9530))
* **errors**  colorizes output red on error ([f8b26b13](https://github.com/kbknapp/clap-rs/commit/f8b26b13da82ba3ba9a932d3d1ab4ea45d1ab036))

#### Improvements

* **arg_enum**  allows ascii case insensitivity for enum variants ([b249f965](https://github.com/kbknapp/clap-rs/commit/b249f9657c6921c004764bd80d13ebca81585eec), closes [#104](https://github.com/kbknapp/clap-rs/issues/104))
* **clap-test**  simplified `make test` invocation ([d17dcb29](https://github.com/kbknapp/clap-rs/commit/d17dcb2920637a1f58c61c596b7bd362fd53047c))

#### Documentation

* **README**  adds details about optional and new features ([960389de](https://github.com/kbknapp/clap-rs/commit/960389de02c9872aaee9adabe86987f71f986e39))
* **clap**  fix typos caught by codespell ([8891d929](https://github.com/kbknapp/clap-rs/commit/8891d92917aa1a069cca67272be41b99e548356e))
* **from_usage**  explains new usage strings with multiple values ([05476fc6](https://github.com/kbknapp/clap-rs/commit/05476fc61cd1e5f4a4e750d258c878732a3a9c64))



<a name="v0.7.6"></a>
## v0.7.6 (2015-05-05)


#### Improvements

* **Options**  adds number of values to options in help/usage ([c1c993c4](https://github.com/kbknapp/clap-rs/commit/c1c993c419d18e35c443785053d8de9a2ef88073))

#### Features

* **from_usage**  adds ability to add value names or num of vals in usage string ([ad55748c](https://github.com/kbknapp/clap-rs/commit/ad55748c265cf27935c7b210307d2040b6a09125), closes [#98](https://github.com/kbknapp/clap-rs/issues/98))

#### Bug Fixes

* **MultipleValues**  properly distinguishes between multiple values and multiple occurrences ([dd2a7564](https://github.com/kbknapp/clap-rs/commit/dd2a75640ca68a91b973faad15f04df891356cef), closes [#99](https://github.com/kbknapp/clap-rs/issues/99))
* **help**  fixes tab alignment with multiple values ([847001ff](https://github.com/kbknapp/clap-rs/commit/847001ff6d8f4d9518e810fefb8edf746dd0f31e))

#### Documentation

* **from_usage**  explains new usage strings with multiple values ([5a3a42df](https://github.com/kbknapp/clap-rs/commit/5a3a42dfa3a783537f88dedc0fd5f0edcb8ea372))



<a name="v0.7.5"></a>
## v0.7.5 (2015-05-04)


#### Bug Fixes

* **Options**  fixes bug where options with no value don't error out ([a1fb94be](https://github.com/kbknapp/clap-rs/commit/a1fb94be53141572ffd97aad037295d4ffec82d0))



<a name="v0.7.4"></a>
## v0.7.4 (2015-05-03)


#### Bug Fixes

* **Options**  fixes a bug where option arguments in succession get their values skipped ([f66334d0](https://github.com/kbknapp/clap-rs/commit/f66334d0ce984e2b56e5c19abb1dd536fae9342a))



<a name="v0.7.3"></a>
## v0.7.3 (2015-05-03)


#### Bug Fixes

* **RequiredValues**  fixes a bug where missing values are parsed as missing arguments ([93c4a723](https://github.com/kbknapp/clap-rs/commit/93c4a7231ba1a08152648598f7aa4503ea82e4de))

#### Improvements

* **ErrorMessages**  improves error messages and corrections ([a29c3983](https://github.com/kbknapp/clap-rs/commit/a29c3983c4229906655a29146ec15a0e46dd942d))
* **ArgGroups**  improves requirement and confliction support for groups ([c236dc5f](https://github.com/kbknapp/clap-rs/commit/c236dc5ff475110d2a1b80e62903f80296163ad3))



<a name="v0.7.2"></a>
## v0.7.2 (2015-05-03)


#### Bug Fixes

* **RequiredArgs**  fixes bug where required-by-default arguments are not listed in usage ([12aea961](https://github.com/kbknapp/clap-rs/commit/12aea9612d290845ba86515c240aeeb0a21198db), closes [#96](https://github.com/kbknapp/clap-rs/issues/96))



<a name="v0.7.1"></a>
## v0.7.1 (2015-05-01)


#### Bug Fixes

* **MultipleValues**  stops evaluating values if the max or exact number of values was reached ([86d92c9f](https://github.com/kbknapp/clap-rs/commit/86d92c9fdbf9f422442e9562977bbaf268dbbae1))



<a name="v0.7.0"></a>
## v0.7.0 (2015-04-30) - BREAKING CHANGE


#### Bug Fixes

* **from_usage**  removes bug where usage strings have no help text ([ad4e5451](https://github.com/kbknapp/clap-rs/commit/ad4e54510739aeabf75f0da3278fb0952db531b3), closes [#83](https://github.com/kbknapp/clap-rs/issues/83))

#### Features

* **MultipleValues**
  *  add support for minimum and maximum number of values ([53f6b8c9](https://github.com/kbknapp/clap-rs/commit/53f6b8c9d8dc408b4fa9f833fc3a63683873c42f))
  *  adds support limited number and named values ([ae09f05e](https://github.com/kbknapp/clap-rs/commit/ae09f05e92251c1b39a83d372736fcc7b504e432))
  *  implement shorthand for options with multiple values ([6669f0a9](https://github.com/kbknapp/clap-rs/commit/6669f0a9687d4f668523145d7bd5c007d1eb59a8))
* **arg**  allow other types besides Vec for multiple value settings (**BREAKING CHANGE** [0cc2f698](https://github.com/kbknapp/clap-rs/commit/0cc2f69839b9b1db5d06330771b494783049a88e), closes [#87](https://github.com/kbknapp/clap-rs/issues/87))
* **usage**  implement smart usage strings on errors ([d77048ef](https://github.com/kbknapp/clap-rs/commit/d77048efb1e595ffe831f1a2bea2f2700db53b9f), closes [#88](https://github.com/kbknapp/clap-rs/issues/88))



<a name="v0.6.9"></a>
## v0.6.9 (2015-04-29)


#### Bug Fixes

* **from_usage**  removes bug where usage strings have no help text ([ad4e5451](https://github.com/kbknapp/clap-rs/commit/ad4e54510739aeabf75f0da3278fb0952db531b3), closes [#83](https://github.com/kbknapp/clap-rs/issues/83))



<a name="0.6.8"></a>
## 0.6.8 (2015-04-27)


#### Bug Fixes

* **help**  change long help --long=long -> --long <long> ([1e25abfc](https://github.com/kbknapp/clap-rs/commit/1e25abfc36679ab89eae71bf98ced4de81992d00))
* **RequiredArgs**  required by default args should no longer be required when their exclusions are present ([4bb4c3cc](https://github.com/kbknapp/clap-rs/commit/4bb4c3cc076b49e86720e882bf8c489877199f2d))

#### Features

* **ArgGroups**  add ability to create arg groups ([09eb4d98](https://github.com/kbknapp/clap-rs/commit/09eb4d9893af40c347e50e2b717e1adef552357d))



<a name="v0.6.7"></a>
## v0.6.7 (2015-04-22)


#### Bug Fixes

* **from_usage**  fix bug causing args to not be required ([b76129e9](https://github.com/kbknapp/clap-rs/commit/b76129e9b71a63365d5c77a7f57b58dbd1e94d49))

#### Features

* **apps**  add ability to display additional help info after auto-gen'ed help msg ([65cc259e](https://github.com/kbknapp/clap-rs/commit/65cc259e4559cbe3653c865ec0c4b1e42a389b07))



<a name="v0.6.6"></a>
## v0.6.6 (2015-04-19)


#### Bug Fixes

* **from_usage**  tabs and spaces should be treated equally ([4fd44181](https://github.com/kbknapp/clap-rs/commit/4fd44181d55d8eb88caab1e625231cfa3129e347))

#### Features

* **macros.rs**  add macro to get version from Cargo.toml ([c630969a](https://github.com/kbknapp/clap-rs/commit/c630969aa3bbd386379219cae27ba1305b117f3e))



<a name="v0.6.5"></a>
## v0.6.5 (2015-04-19)


#### Bug Fixes

* **macros.rs**  fix use statements for trait impls ([86e4075e](https://github.com/kbknapp/clap-rs/commit/86e4075eb111937c8a7bdb344e866e350429f042))



<a name="v0.6.4"></a>
## v0.6.4 (2015-04-17)


#### Features

* **macros**  add ability to create enums pub or priv with derives ([2c499f80](https://github.com/kbknapp/clap-rs/commit/2c499f8015a199827cdf1fa3ec4f6f171722f8c7))



<a name="v0.6.3"></a>
## v0.6.3 (2015-04-16)


#### Features

* **macros**  add macro to create custom enums to use as types ([fb672aff](https://github.com/kbknapp/clap-rs/commit/fb672aff561c29db2e343d6c607138f141aca8b6))



<a name="v0.6.2"></a>
## v0.6.2 (2015-04-14)


#### Features

* **macros**
  *  add ability to get multiple typed values or exit ([0b87251f](https://github.com/kbknapp/clap-rs/commit/0b87251fc088234bee51c323c2b652d7254f7a59))
  *  add ability to get a typed multiple values ([e243fe38](https://github.com/kbknapp/clap-rs/commit/e243fe38ddbbf845a46c0b9baebaac3778c80927))
  *  add convenience macro to get a typed value or exit ([4b7cd3ea](https://github.com/kbknapp/clap-rs/commit/4b7cd3ea4947780d9daa39f3e1ddab53ad4c7fef))
  *  add convenience macro to get a typed value ([8752700f](https://github.com/kbknapp/clap-rs/commit/8752700fbb30e89ee68adbce24489ae9a24d33a9))



<a name="v0.6.1"></a>
## v0.6.1 (2015-04-13)


#### Bug Fixes

* **from_usage**  trim all whitespace before parsing ([91d29045](https://github.com/kbknapp/clap-rs/commit/91d2904599bd602deef2e515dfc65dc2863bdea0))



<a name="v0.6.0"></a>
## v0.6.0 (2015-04-13)


#### Bug Fixes

* **tests**  fix failing doc tests ([3710cd69](https://github.com/kbknapp/clap-rs/commit/3710cd69162f87221a62464f63437c1ce843ad3c))

#### Features

* **app**  add support for building args from usage strings ([d5d48bcf](https://github.com/kbknapp/clap-rs/commit/d5d48bcf463a4e494ef758836bd69a4c220bbbb5))
* **args**  add ability to create basic arguments from a usage string ([ab409a8f](https://github.com/kbknapp/clap-rs/commit/ab409a8f1db9e37cc70200f6f4a84a162692e618))



<a name="v0.5.14"></a>
## v0.5.14 (2015-04-10)


#### Bug Fixes

* **usage**
  *  remove unneeded space ([51372789](https://github.com/kbknapp/clap-rs/commit/5137278942121bc2593ce6e5dc224ec2682549e6))
  *  remove warning about unused variables ([ba817b9d](https://github.com/kbknapp/clap-rs/commit/ba817b9d815e37320650973f1bea0e7af3030fd7))

#### Features

* **usage**  add ability to get usage string for subcommands too ([3636afc4](https://github.com/kbknapp/clap-rs/commit/3636afc401c2caa966efb5b1869ef4f1ed3384aa))



<a name="v0.5.13"></a>
## v0.5.13 (2015-04-09)


#### Features

* **SubCommands**  add method to get name and subcommand matches together ([64e53928](https://github.com/kbknapp/clap-rs/commit/64e539280e23e567cf5de393b346eb0ca20e7eb5))
* **ArgMatches**  add method to get default usage string ([02462150](https://github.com/kbknapp/clap-rs/commit/02462150ca750bdc7012627d7e8d96379d494d7f))



<a name="v0.5.12"></a>
## v0.5.12 (2015-04-08)


#### Features

* **help**  sort arguments by name so as to not display a random order ([f4b2bf57](https://github.com/kbknapp/clap-rs/commit/f4b2bf5767386013069fb74862e6e938dacf44d2))



<a name="v0.5.11"></a>
## v0.5.11 (2015-04-08)


#### Bug Fixes

* **flags**  fix bug not allowing users to specify -v or -h ([90e72cff](https://github.com/kbknapp/clap-rs/commit/90e72cffdee321b79eea7a2207119533540062b4))



<a name="v0.5.10"></a>
## v0.5.10 (2015-04-08)


#### Bug Fixes

* **help**  fix spacing when option argument has not long version ([ca17fa49](https://github.com/kbknapp/clap-rs/commit/ca17fa494b68e92da83ee364bf64b0687006824b))



<a name="v0.5.9"></a>
## v0.5.9 (2015-04-08)


#### Bug Fixes

* **positional args**  all previous positional args become required when a latter one is required ([c14c3f31](https://github.com/kbknapp/clap-rs/commit/c14c3f31fd557c165570b60911d8ee483d89d6eb), closes [#50](https://github.com/kbknapp/clap-rs/issues/50))
* **clap**  remove unstable features for Rust 1.0 ([9abdb438](https://github.com/kbknapp/clap-rs/commit/9abdb438e36e364d41550e7f5d44ebcaa8ee6b10))
* **args**  improve error messages for arguments with mutual exclusions ([18dbcf37](https://github.com/kbknapp/clap-rs/commit/18dbcf37024daf2b76ca099a6f118b53827aa339), closes [#51](https://github.com/kbknapp/clap-rs/issues/51))



<a name="v0.5.8"></a>
## v0.5.8 (2015-04-08)


#### Bug Fixes

* **option args**  fix bug in getting the wrong number of occurrences for options ([82ad6ad7](https://github.com/kbknapp/clap-rs/commit/82ad6ad77539cf9f9a03b78db466f575ebd972cc))
* **help**  fix formatting for option arguments with no long ([e8691004](https://github.com/kbknapp/clap-rs/commit/e869100423d93fa3acff03c4620cbcc0d0e790a1))
* **flags**  add assertion to catch flags with specific value sets ([a0a2a40f](https://github.com/kbknapp/clap-rs/commit/a0a2a40fed57f7c5ad9d68970d090e9856306c7d), closes [#52](https://github.com/kbknapp/clap-rs/issues/52))
* **args**  improve error messages for arguments with mutual exclusions ([bff945fc](https://github.com/kbknapp/clap-rs/commit/bff945fc5d03bba4266533340adcffb002508d1b), closes [#51](https://github.com/kbknapp/clap-rs/issues/51))
* **tests**  add missing .takes_value(true) to option2 ([bdb0e88f](https://github.com/kbknapp/clap-rs/commit/bdb0e88f696c8595c3def3bfb0e52d538c7be085))
* **positional args**  all previous positional args become required when a latter one is required ([343d47dc](https://github.com/kbknapp/clap-rs/commit/343d47dcbf83786a45c0d0f01b27fd9dd76725de), closes [#50](https://github.com/kbknapp/clap-rs/issues/50))



<a name="v0.5.7"></a>
## v0.5.7 (2015-04-08)


#### Bug Fixes

* **args**  fix bug in arguments who are required and mutually exclusive ([6ceb88a5](https://github.com/kbknapp/clap-rs/commit/6ceb88a594caae825605abc1cdad95204996bf29))



<a name="v0.5.6"></a>
## v0.5.6 (2015-04-08)


#### Bug Fixes

* **help**  fix formatting of help and usage ([28691b52](https://github.com/kbknapp/clap-rs/commit/28691b52f67e65c599e10e4ea2a0f6f9765a06b8))



<a name="v0.5.5"></a>
## v0.5.5 (2015-04-08)


#### Bug Fixes

* **help**  fix formatting of help for flags and options ([6ec10115](https://github.com/kbknapp/clap-rs/commit/6ec1011563a746f0578a93b76d45e63878e0f9a8))



<a name="v0.5.4"></a>
## v0.5.4 (2015-04-08)


#### Features

* **help**  add '...' to indicate multiple values supported ([297ddba7](https://github.com/kbknapp/clap-rs/commit/297ddba77000e2228762ab0eca50b480f7467386))



<a name="v0.5.3"></a>
## v0.5.3 (2015-04-08)


#### Features

* **positionals**
  *  add assertions for positional args with multiple vals ([b7fa72d4](https://github.com/kbknapp/clap-rs/commit/b7fa72d40f18806ec2042dd67a518401c2cf5681))
  *  add support for multiple values ([80784009](https://github.com/kbknapp/clap-rs/commit/807840094109fbf90b348039ae22669ef27889ba))



<a name="v0.5.2"></a>
## v0.5.2 (2015-04-08)


#### Bug Fixes

* **apps**  allow use of hyphens in application and subcommand names ([da549dcb](https://github.com/kbknapp/clap-rs/commit/da549dcb6c7e0d773044ab17829744483a8b0f7f))



<a name="v0.5.1"></a>
## v0.5.1 (2015-04-08)


#### Bug Fixes

* **args**  determine if the only arguments allowed are also required ([0a09eb36](https://github.com/kbknapp/clap-rs/commit/0a09eb365ced9a03faf8ed24f083ef730acc90e8))



<a name="v0.5.0"></a>
## v0.5.0 (2015-04-08)


#### Features

* **args**  add support for a specific set of allowed values on options or positional arguments ([270eb889](https://github.com/kbknapp/clap-rs/commit/270eb88925b6dc2881bff1f31ee344f085d31809))



<a name="v0.4.18"></a>
## v0.4.18 (2015-04-08)


#### Bug Fixes

* **usage**  display required args in usage, even if only required by others ([1b7316d4](https://github.com/kbknapp/clap-rs/commit/1b7316d4a8df70b0aa584ccbfd33f68966ad2a54))

#### Features

* **subcommands**  properly list subcommands in help and usage ([4ee02344](https://github.com/kbknapp/clap-rs/commit/4ee023442abc3dba54b68138006a52b714adf331))



<a name="v0.4.17"></a>
## v0.4.17 (2015-04-08)


#### Bug Fixes

* **tests**  remove cargo test from claptests makefile ([1cf73817](https://github.com/kbknapp/clap-rs/commit/1cf73817d6fb1dccb5b6a23b46c2efa8b567ad62))



<a name="v0.4.16"></a>
## v0.4.16 (2015-04-08)


#### Bug Fixes

* **option**  fix bug with option occurrence values ([9af52e93](https://github.com/kbknapp/clap-rs/commit/9af52e93cef9e17ac9974963f132013d0b97b946))
* **tests**  fix testing script bug and formatting ([d8f03a55](https://github.com/kbknapp/clap-rs/commit/d8f03a55c4f74d126710ee06aad5a667246a8001))

#### Features

* **arg**  allow lifetimes other than 'static in arguments ([9e8c1fb9](https://github.com/kbknapp/clap-rs/commit/9e8c1fb9406f8448873ca58bab07fe905f1551e5))
