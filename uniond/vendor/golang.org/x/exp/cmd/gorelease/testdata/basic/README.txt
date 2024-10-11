Module example.com/basic tests basic functionality of gorelease.
It verifies that versions are correctly suggested or verified after
various changes.

All revisions are stored in the mod directory. The same series of
changes is made across three major versions, v0, v1, and v2:

vX.0.1 - simple package
vX.1.0 - compatible change: add a function and a package
vX.1.1 - internal change: function returns different value
vX.1.2 - incompatible change: delete a function and a package
