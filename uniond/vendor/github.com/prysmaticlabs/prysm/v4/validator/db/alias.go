package db

import "github.com/prysmaticlabs/prysm/v4/validator/db/iface"

// Database defines the necessary methods for Prysm's validator client backend which may be implemented by any
// key-value or relational database in practice. This is the full database interface which should
// not be used often. Prefer a more restrictive interface in this package.
type Database = iface.ValidatorDB
