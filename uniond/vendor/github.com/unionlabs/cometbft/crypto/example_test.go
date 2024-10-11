package crypto_test

import (
	"fmt"

	"github.com/cometbft/cometbft/crypto"
)

func ExampleSha256() {
	sum := crypto.Sha256([]byte("This is CometBFT"))
	fmt.Printf("%x\n", sum)
	// Output:
	// ea186526b041852d923b02c91aa04b00c0df258b3d69cb688eaba577f5562758
}
