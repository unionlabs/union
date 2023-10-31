# promptly

A simple, opinionated prompting library

Features include:

- Re-prompt until valid
- Prompts for several types, and extensible
- Sane handling of escapes via rustyline
- Path completion when prompting for paths
- Dead simple to use. Perhaps too simple.

## Usage

Simply call `prompt` or `prompt_default` to prompt for any `Promptable` type:

- `prompt(msg)` - prompt until input can be parsed as the inferred return type. Re-prompts if input is empty.
- `prompt_opt(msg)` - prompt until input can be parsed as the inferred return type. Returns `None` if input is empty.
- `prompt_default(msg, default)` - prompt until input can be parsed as the inferred return type. Uses `default` value if input is empty.


```rust
use promptly::{prompt, prompt_default, prompt_opt};

// Prompt until a non-empty string is provided
let name: String = prompt("Enter your name")?;

// Prompt for other `FromStr` types
let age: u32 = prompt("Enter your age")?;

// Prompt for optional paths with path completion. Returns `None` if empty input.
let photo: Option<PathBuf> = prompt_opt("Enter a path to a profile picture")?;

// Prompt Y/n with a default value when input is empty
let fallback = prompt_default("Would you like to receive marketing emails", true);

// Prompt for a url using the url crate (requires either 'nightly' or 'url' feature)
let website: Url = prompt("Enter a website URL");
```

## More...

The API surface of this crate is opinionated and experimental, but open to fresh ideas.

