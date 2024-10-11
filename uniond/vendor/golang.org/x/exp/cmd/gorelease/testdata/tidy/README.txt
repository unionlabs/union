Module example.com/tidy tests versions that do not have tidy
go.mod or go.sum files.

v0.0.1 has a trivial package with no imports. It has no requirements
and no go.sum, so it is tidy. Tests make changes on top of this.
