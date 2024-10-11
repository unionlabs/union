package v6

import "cosmossdk.io/collections"

var (
	ValidatorUpdatesKey = collections.NewPrefix(97)
	HistoricalInfoKey   = collections.NewPrefix(80) // prefix for the historical info
)
