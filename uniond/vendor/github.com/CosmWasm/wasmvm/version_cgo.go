//go:build cgo

package cosmwasm

import (
	"github.com/CosmWasm/wasmvm/internal/api"
)

func libwasmvmVersionImpl() (string, error) {
	return api.LibwasmvmVersion()
}
