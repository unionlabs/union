Tests in this directory cover scenarios where errors in a package are fixed.

v1.0.0 is used as the base version for all tests.
It has an error: the return type of bad.Broken is undefined.

Each test fixes the error and may make other changes (compatible or not).
Note that fixing a type error in the API appears to be an incompatible change.
