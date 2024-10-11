This directory contains tests for modules that aren't at the root
of the repository, which is marked with a .git directory.
We're comparing against an earlier published version with a
trivial package. Nothing has changed except the location of the
module within the repository.

  example.com/sub - corresponds to the root directory. Not a module.
  example.com/sub/v2 - may be in v2 subdirectory.
  example.com/sub/nest - nested module in subdirectory
  example.com/sub/nest/v2 - may be in nest or nest/v2.
