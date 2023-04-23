package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	// SubModuleName defines the interchain accounts host module name
	SubModuleName = "icahost"

	// StoreKey is the store key string for the interchain accounts host module
	StoreKey = SubModuleName

	// AllowAllHostMsgs holds the string key that allows all message types on interchain accounts host module
	AllowAllHostMsgs = "*"
)

// ContainsMsgType returns true if the sdk.Msg TypeURL is present in allowMsgs, otherwise false
func ContainsMsgType(allowMsgs []string, msg sdk.Msg) bool {
	// check that wildcard * option for allowing all message types is the only string in the array, if so, return true
	if len(allowMsgs) == 1 && allowMsgs[0] == AllowAllHostMsgs {
		return true
	}

	for _, v := range allowMsgs {
		if v == sdk.MsgTypeURL(msg) {
			return true
		}
	}

	return false
}
