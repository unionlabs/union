# DB

The `db` package contains the key-value database interface and `memdb` implementation. The `memdb` is a simple in-memory key-value store that is used for testing and development purposes.

## Context

The main purpose of the `db` package is to provide decoupling between `cosmos-db` and `iavl` packages. It provides a simple `wrapper` for the old users of `iavl` and `cosmos-db` to use the new `db` package without changing their code. For example:

```go
package main

import (
    "cosmossdk.io/log"
    dbm "github.com/cosmos/cosmos-db"

    "github.com/cosmos/iavl"
    idbm "github.com/cosmos/iavl/db"
)

func main() {
    levelDB, err := dbm.NewDB("application", dbm.GoLevelDBBackend, "test")
 if err != nil {
  panic(err)
 }

 tree := iavl.NewMutableTree(idbm.NewWrapper(dbm.NewPrefixDB(levelDB, []byte("s/k:main/"))), 0, false, NewNopLogger())
}
```
