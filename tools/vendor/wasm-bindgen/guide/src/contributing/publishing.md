# Publishing New `wasm-bindgen` Releases

1. <input type="checkbox"/> Compile the `publish.rs` script:

   ```
   rustc publish.rs
   ```

2. <input type="checkbox"/> Bump every crate's minor version:

   ```
   # Make sure you are in the root of the wasm-bindgen repo!
   ./publish bump
   ```

3. <input type="checkbox"/> Send a pull request for the version bump.

4. <input type="checkbox"/> After the pull request's CI is green and it has been
   merged, publish to cargo:

   ```
   # Make sure you are in the root of the wasm-bindgen repo!
   ./publish publish
   ```
