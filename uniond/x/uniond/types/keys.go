package types

const (
	// ModuleName defines the module name
	ModuleName = "uniond"

	// StoreKey defines the primary module store key
	StoreKey = ModuleName

	// MemStoreKey defines the in-memory store key
	MemStoreKey = "mem_uniond"
)

var (
	ParamsKey = []byte("p_uniond")
)

func KeyPrefix(p string) []byte {
	return []byte(p)
}
