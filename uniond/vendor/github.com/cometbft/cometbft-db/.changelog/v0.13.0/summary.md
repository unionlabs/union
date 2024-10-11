*Aug 2, 2024*

This release:
- changes the contract of the Iterator Key() and Value() APIs. Namely, the caller is now responsible for creating a copy of their returned value if they want to modify it.
- removes support for boltDB and clevelDB, which were marked as deprecated in release v0.12.0.