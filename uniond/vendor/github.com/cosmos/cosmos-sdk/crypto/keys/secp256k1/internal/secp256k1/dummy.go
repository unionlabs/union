//go:build dummy
// +build dummy

// This file is part of a workaround for `go mod vendor` which won't vendor
// C files if there's no Go file in the same directory.
// This would prevent the crypto/secp256k1/libsecp256k1/include/secp256k1.h file to be vendored.
//
// This Go file imports the c directory where there is another dummy.go file which
// is the second part of this workaround.
//
// These two files combined make it so `go mod vendor` behaves correctly.
//
// See this issue for reference: https://github.com/golang/go/issues/26366

package secp256k1

import (
	_ "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1/internal/secp256k1/libsecp256k1/include"
	_ "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1/internal/secp256k1/libsecp256k1/src"
	_ "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1/internal/secp256k1/libsecp256k1/src/modules/recovery"
)
