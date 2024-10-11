package iface

import (
	"errors"
)

var (
	// ErrExistingGenesisState is an error when the user attempts to save a different genesis state
	// when one already exists in a database.
	ErrExistingGenesisState = errors.New("genesis state exists already in the DB")
)
