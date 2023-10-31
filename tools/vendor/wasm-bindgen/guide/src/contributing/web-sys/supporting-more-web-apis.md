# Supporting More Web APIs in `web-sys`

1. Ensure that the `.webidl` file describing the
   interface exists somewhere within the `crates/web-sys/webidls/enabled`
   directory.

   First, check to see whether we have the WebIDL definition file for
   your API:

   ```sh
   grep -rn MyWebApi crates/web-sys/webidls
   ```

   * If your interface is defined in a `.webidl` file that is inside the
     `crates/web-sys/webidls/enabled` directory, skip to step (3).

   * If your interface isn't defined in any file yet, find the WebIDL definition
     in the relevant standard and add it as a new `.webidl` file in
     `crates/web-sys/webidls/enabled`. Make sure that it is a standard Web API!
     We don't want to add non-standard APIs to this crate.

   * If your interface is defined in a `.webidl` file within any of the
     `crates/web-sys/webidls/unavailable_*` directories, you need to move it into
     `crates/web-sys/webidls/enabled`, e.g.:

     ```sh
     cd crates/web-sys
     git mv webidls/unavailable_enum_ident/MyWebApi.webidl webidls/enabled/MyWebApi.webidl
     ```

2. Regenerate the `web-sys` crate auto-generated bindings, which you can do with
   the following commands:

   ```sh
   cd crates/web-sys
   cargo run --release --package wasm-bindgen-webidl -- webidls src/features ./Cargo.toml
   ```

   You can then use `git diff` to ensure the bindings look correct.
