package v2

import "strconv"

const (
	// ModuleName is the name of the module
	ModuleName = "staking"
)

var HistoricalInfoKey = []byte{0x50} // prefix for the historical info

// GetHistoricalInfoKey returns a key prefix for indexing HistoricalInfo objects.
func GetHistoricalInfoKey(height int64) []byte {
	return append(HistoricalInfoKey, []byte(strconv.FormatInt(height, 10))...)
}
