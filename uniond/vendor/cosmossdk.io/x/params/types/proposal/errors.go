package proposal

import "cosmossdk.io/errors"

// x/params module sentinel errors
var (
	ErrUnknownSubspace  = errors.Register(ModuleName, 2, "unknown subspace")
	ErrSettingParameter = errors.Register(ModuleName, 3, "failed to set parameter")
	ErrEmptyChanges     = errors.Register(ModuleName, 4, "submitted parameter changes are empty")
	ErrEmptySubspace    = errors.Register(ModuleName, 5, "parameter subspace is empty")
	ErrEmptyKey         = errors.Register(ModuleName, 6, "parameter key is empty")
	ErrEmptyValue       = errors.Register(ModuleName, 7, "parameter value is empty")
)
