*September 20, 2024*

This release swaps the "default" DB from goleveldb to pebbledb. There's now a
`goleveldb` build flag that must be used when using goleveldb. If you're using
`pebbledb`, you don't need a build flag anymore.
