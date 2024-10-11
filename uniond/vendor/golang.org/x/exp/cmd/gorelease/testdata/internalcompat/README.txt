Modules example.com/internalcompat/{a,b} are copies. One could be a fork
of the other. An external package p exposes a type from a package q
within the same module.

If gorelease ran apidiff on the two modules instead of on the individual
packages, then it should not report differences between these packages. The types
are distinct, but they correspond (in apidiff terminology), which is the
important property when considering differences between modules. More
specifically, the fully qualified type names are identical modulo the change
to the module path.

But at the time gorelease was written, apidiff did not support module
comparison. If considered at the package level, the two packages
example.com/internalcompat/a/p and example.com/internalcompat/b/p
are incompatible, because the packages they refer to are different.

So case 2 below would apply if whole modules were being diffed, but
it doesn't here because individual packages are being diffed.

There are three use cases to consider:

1. One module substitutes for the other via a `replace` directive.
   Only the replacement module is used, and the package paths are effectively
   identical, so the types are not distinct.
2. One module subsititutes for the other by rewriting `import` statements
   globally. All references to the original type become references to the
   new type, so there is no conflict.
3. One module substitutes for the other by rewriting some `import` statements
   but not others (for example, those within a specific consumer package).
   In this case, the types are distinct, and even if there are no changes,
   the types are not compatible.
