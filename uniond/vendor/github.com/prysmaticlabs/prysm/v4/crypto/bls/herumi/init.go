package herumi

import "github.com/herumi/bls-eth-go-binary/bls"

// Init allows the required curve orders and appropriate sub-groups to be initialized.
func Init() {
	if err := bls.Init(bls.BLS12_381); err != nil {
		panic(err)
	}
	if err := bls.SetETHmode(bls.EthModeDraft07); err != nil {
		panic(err)
	}
	// Check subgroup order for pubkeys and signatures.
	bls.VerifyPublicKeyOrder(true)
	bls.VerifySignatureOrder(true)
}
