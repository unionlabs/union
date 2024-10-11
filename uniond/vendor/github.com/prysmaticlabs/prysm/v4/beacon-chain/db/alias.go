package db

import "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/iface"

// ReadOnlyDatabase exposes Prysm's Ethereum data backend for read access only, no information about
// head info. For head info, use github.com/prysmaticlabs/prysm/blockchain.HeadFetcher.
type ReadOnlyDatabase = iface.ReadOnlyDatabase

// NoHeadAccessDatabase exposes Prysm's Ethereum data backend for read/write access, no information
// about head info. For head info, use github.com/prysmaticlabs/prysm/blockchain.HeadFetcher.
type NoHeadAccessDatabase = iface.NoHeadAccessDatabase

// HeadAccessDatabase exposes Prysm's Ethereum backend for read/write access with information about
// chain head information. This interface should be used sparingly as the HeadFetcher is the source
// of truth around chain head information while this interface serves as persistent storage for the
// head fetcher.
//
// See github.com/prysmaticlabs/prysm/blockchain.HeadFetcher
type HeadAccessDatabase = iface.HeadAccessDatabase

// Database defines the necessary methods for Prysm's Ethereum backend which may be implemented by any
// key-value or relational database in practice. This is the full database interface which should
// not be used often. Prefer a more restrictive interface in this package.
type Database = iface.Database

// SlasherDatabase defines necessary methods for Prysm's slasher implementation.
type SlasherDatabase = iface.SlasherDatabase

// ErrExistingGenesisState is an error when the user attempts to save a different genesis state
// when one already exists in a database.
var ErrExistingGenesisState = iface.ErrExistingGenesisState
