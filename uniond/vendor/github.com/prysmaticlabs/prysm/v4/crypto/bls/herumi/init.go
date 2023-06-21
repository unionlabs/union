package herumi

import "github.com/herumi/bls-eth-go-binary/bls"

// HerumiInit allows the required curve orders and appropriate sub-groups to be initialized.
func HerumiInit() {
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
