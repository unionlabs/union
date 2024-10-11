# Go Coding Style Guide

In order to keep our code looking good with lots of programmers working on it, it helps to have a "style guide", so all
the code generally looks quite similar. This doesn't mean there is only one "right way" to write code, or even that this
standard is better than your style.  But if we agree to a number of stylistic practices, it makes it much easier to read
and modify new code. Please feel free to make suggestions if there's something you would like to add or modify.

We expect all contributors to be familiar with [Effective Go](https://golang.org/doc/effective_go.html)
(and it's recommended reading for all Go programmers anyways). Additionally, we generally agree with the suggestions
 in [Uber's style guide](https://github.com/uber-go/guide/blob/master/style.md) and use that as a starting point.


## Code Structure

Perhaps more key for code readability than good commenting is having the right structure. As a rule of thumb, try to write
in a logical order of importance, taking a little time to think how to order and divide the code such that someone could
scroll down and understand the functionality of it just as well as you do. A loose example of such order would be:

* Constants, global and package-level variables
* Main Struct
* Options (only if they are seen as critical to the struct else they should be placed in another file)
* Initialization / Start and stop of the service
* Msgs/Events
* Public Functions (In order of most important)
* Private/helper functions
* Auxiliary structs and function (can also be above private functions or in a separate file)

## General

* Use `gofmt` (or `goimport`) to format all code upon saving it.  (If you use VIM, check out vim-go).
* Use a linter (see below) and generally try to keep the linter happy (where it makes sense).
* Think about documentation, and try to leave godoc comments, when it will help new developers.
* Every package should have a high level doc.go file to describe the purpose of that package, its main functions, and any other relevant information.
* `TODO` should not be used. If important enough should be recorded as an issue.
* `BUG` / `FIXME` should be used sparingly to guide future developers on some of the vulnerabilities of the code.
* `XXX` can be used in work-in-progress (prefixed with "WIP:" on github) branches but they must be removed before approving a PR.
* Applications (e.g. clis/servers) *should* panic on unexpected unrecoverable errors and print a stack trace.

## Comments

* Use a space after comment deliminter (ex. `// your comment`).
* Many comments are not sentences. These should begin with a lower case letter and end without a period.
* Conversely, sentences in comments should be sentenced-cased and end with a period.

## Linters

These must be applied to all (Go) repos.

* [shellcheck](https://github.com/koalaman/shellcheck)
* [golangci-lint](https://github.com/golangci/golangci-lint) (covers all important linters)
    * See the `.golangci.yml` file in each repo for linter configuration.

## Various

* Reserve "Save" and "Load" for long-running persistence operations. When parsing bytes, use "Encode" or "Decode".
* Maintain consistency across the codebase.
* Functions that return functions should have the suffix `Fn`
* Names should not [stutter](https://blog.golang.org/package-names). For example, a struct generally shouldn’t have
  a field named after itself; e.g., this shouldn't occur:

``` golang
type middleware struct {
	middleware Middleware
}
```

* In comments, use "iff" to mean, "if and only if".
* Product names are capitalized, like "CometBFT", "Basecoin", "Protobuf", etc except in command lines: `cometbft --help`
* Acronyms are all capitalized, like "RPC", "gRPC", "API".  "MyID", rather than "MyId".
* Prefer errors.New() instead of fmt.Errorf() unless you're actually using the format feature with arguments.

## Importing Libraries

Sometimes it's necessary to rename libraries to avoid naming collisions or ambiguity.

* Use [goimports](https://godoc.org/golang.org/x/tools/cmd/goimports)
* Separate imports into blocks - one for the standard lib, one for external libs and one for application libs.
* Here are some common library labels for consistency:
    * dbm "github.com/cometbft/cometbft-db"
    * cmtcmd "github.com/cometbft/cometbft/cmd/cometbft/commands"
    * cmtcfg "github.com/cometbft/cometbft/config"
    * cmttypes "github.com/cometbft/cometbft/types"
* Never use anonymous imports (the `.`), for example, `cmtlibs/common` or anything else.
* When importing a pkg from the `cmt/libs` directory, prefix the pkg alias with cmt.
    * cmtbits "github.com/cometbft/cometbft/libs/bits"
* tip: Use the `_` library import to import a library for initialization effects (side effects)

## Dependencies

* Dependencies should be pinned by a release tag, or specific commit, to avoid breaking `go get` when external dependencies are updated.
* Refer to the [contributing](CONTRIBUTING.md) document for more details

## Testing

* The first rule of testing is: we add tests to our code
* The second rule of testing is: we add tests to our code
* For Golang testing:
    * Make use of table driven testing where possible and not-cumbersome
        * [Inspiration](https://dave.cheney.net/2013/06/09/writing-table-driven-tests-in-go)
    * Make use of [assert](https://godoc.org/github.com/stretchr/testify/assert) and [require](https://godoc.org/github.com/stretchr/testify/require)
* When using mocks, it is recommended to use Testify [mock](<https://pkg.go.dev/github.com/stretchr/testify/mock>
 ) along with [Mockery](https://github.com/vektra/mockery) for autogeneration

## Errors

* Ensure that errors are concise, clear and traceable.
* Use stdlib errors package.
* For wrapping errors, use `fmt.Errorf()` with `%w`.
* Panic is appropriate when an internal invariant of a system is broken, while all other cases (in particular,
  incorrect or invalid usage) should return errors.

## Config

* Currently the TOML filetype is being used for config files
* A good practice is to store per-user config files under `~/.[yourAppName]/config.toml`

## CLI

* When implementing a CLI use [Cobra](https://github.com/spf13/cobra) and [Viper](https://github.com/spf13/viper).
* Helper messages for commands and flags must be all lowercase.
* Instead of using pointer flags (eg. `FlagSet().StringVar`) use Viper to retrieve flag values (eg. `viper.GetString`)
    * The flag key used when setting and getting the flag should always be stored in a
   variable taking the form `FlagXxx` or `flagXxx`.
    * Flag short variable descriptions should always start with a lower case character as to remain consistent with
   the description provided in the default `--help` flag.

## Version

* Every repo should have a version/version.go file that mimics the CometBFT repo
* We read the value of the constant version in our build scripts and hence it has to be a string

## Non-Go Code

* All non-Go code (`*.proto`, `Makefile`, `*.sh`), where there is no common
   agreement on style, should be formatted according to
   [EditorConfig](http://editorconfig.org/) config:

   ```toml
   # top-most EditorConfig file
   root = true

   # Unix-style newlines with a newline ending every file
   [*]
   charset = utf-8
   end_of_line = lf
   insert_final_newline = true
   trim_trailing_whitespace = true

   [Makefile]
   indent_style = tab

   [*.sh]
   indent_style = tab

   [*.proto]
   indent_style = space
   indent_size = 2
   ```

   Make sure the file above (`.editorconfig`) are in the root directory of your
   repo and you have a [plugin for your
   editor](http://editorconfig.org/#download) installed.
