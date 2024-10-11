package endtoend

// This file contains the dependencies required for github.com/ethereum/go-ethereum/cmd/geth.
// Having these dependencies listed here helps go mod understand that these dependencies are
// necessary for end to end tests since we build go-ethereum binary for this test.
import (
	_ "github.com/ethereum/go-ethereum/accounts"          // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/accounts/keystore" // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/cmd/utils"         // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/common"            // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/console"           // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/eth"               // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/eth/downloader"    // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/ethclient"         // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/les"               // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/log"               // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/metrics"           // Required for go-ethereum e2e.
	_ "github.com/ethereum/go-ethereum/node"              // Required for go-ethereum e2e.
)
