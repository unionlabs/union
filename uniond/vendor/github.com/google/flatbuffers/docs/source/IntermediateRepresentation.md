# Flatbuffers Intermediate Representation {#intermediate_representation}

We use [reflection.fbs](https://github.com/google/flatbuffers/blob/master/reflection/reflection.fbs)
as our intermediate representation. `flatc` parses `.fbs` files, checks them for
errors and stores the resulting data in this IR, outputting `.bfbs` files.
Since this IR is a Flatbuffer, you can load and use it at runtime for runtime
reflection purposes.

There are some quirks: 
- Tables and Structs are serialized as `Object`s.
- Unions and Enums are serialized as `Enum`s.
- It is the responsibility of the code generator to check the `advanced_features`
  field of `Schema`. These mark the presence of new, backwards incompatible,
  schema features. Code generators must error if generating a schema with
  unrecognized advanced features.
- Filenames are relative to a "project root" denoted by "//" in the path. This
  may be specified in flatc with `--bfbs-filenames=$PROJECT_ROOT`, or it will be
  inferred to be the directory containing the first provided schema file.


## Invocation 
You can invoke it like so
```{.sh}
flatc -b --schema ${your_fbs_files}
```
This generates `.bfbs` (binary flatbuffer schema) files.

Some information is not included by default. See the `--bfbs-filenames` and
`--bfbs-comments` flags. These may be necessary for code-generators, so they can
add documentation and maybe name generated files (depending on the generator).


TODO(cneo): Flags to output bfbs as flexbuffers or json.

TODO(cneo): Tutorial for building a flatc plugin.
