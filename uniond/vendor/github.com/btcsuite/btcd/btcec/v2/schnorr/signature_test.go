// Copyright (c) 2013-2017 The btcsuite developers
// Copyright (c) 2015-2021 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package schnorr

import (
	"encoding/hex"
	"errors"
	"strings"
	"testing"
	"testing/quick"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/davecgh/go-spew/spew"
	secp_ecdsa "github.com/decred/dcrd/dcrec/secp256k1/v4"
	ecdsa_schnorr "github.com/decred/dcrd/dcrec/secp256k1/v4/schnorr"
)

type bip340Test struct {
	secretKey    string
	publicKey    string
	auxRand      string
	message      string
	signature    string
	verifyResult bool
	validPubKey  bool
	expectErr    error
	rfc6979      bool
}

var bip340TestVectors = []bip340Test{
	{
		secretKey:    "0000000000000000000000000000000000000000000000000000000000000003",
		publicKey:    "F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9",
		auxRand:      "0000000000000000000000000000000000000000000000000000000000000000",
		message:      "0000000000000000000000000000000000000000000000000000000000000000",
		signature:    "04E7F9037658A92AFEB4F25BAE5339E3DDCA81A353493827D26F16D92308E49E2A25E92208678A2DF86970DA91B03A8AF8815A8A60498B358DAF560B347AA557",
		verifyResult: true,
		validPubKey:  true,
		rfc6979:      true,
	},
	{
		secretKey:    "0000000000000000000000000000000000000000000000000000000000000003",
		publicKey:    "F9308A019258C31049344F85F89D5229B531C845836F99B08601F113BCE036F9",
		auxRand:      "0000000000000000000000000000000000000000000000000000000000000000",
		message:      "0000000000000000000000000000000000000000000000000000000000000000",
		signature:    "E907831F80848D1069A5371B402410364BDF1C5F8307B0084C55F1CE2DCA821525F66A4A85EA8B71E482A74F382D2CE5EBEEE8FDB2172F477DF4900D310536C0",
		verifyResult: true,
		validPubKey:  true,
	},
	{
		secretKey:    "B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF",
		publicKey:    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
		auxRand:      "0000000000000000000000000000000000000000000000000000000000000001",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "6896BD60EEAE296DB48A229FF71DFE071BDE413E6D43F917DC8DCF8C78DE33418906D11AC976ABCCB20B091292BFF4EA897EFCB639EA871CFA95F6DE339E4B0A",
		verifyResult: true,
		validPubKey:  true,
	},
	{
		secretKey:    "C90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B14E5C9",
		publicKey:    "DD308AFEC5777E13121FA72B9CC1B7CC0139715309B086C960E18FD969774EB8",
		auxRand:      "C87AA53824B4D7AE2EB035A2B5BBBCCC080E76CDC6D1692C4B0B62D798E6D906",
		message:      "7E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C",
		signature:    "5831AAEED7B44BB74E5EAB94BA9D4294C49BCF2A60728D8B4C200F50DD313C1BAB745879A5AD954A72C45A91C3A51D3C7ADEA98D82F8481E0E1E03674A6F3FB7",
		verifyResult: true,
		validPubKey:  true,
	},
	{
		secretKey:    "0B432B2677937381AEF05BB02A66ECD012773062CF3FA2549E44F58ED2401710",
		publicKey:    "25D1DFF95105F5253C4022F628A996AD3A0D95FBF21D468A1B33F8C160D8F517",
		auxRand:      "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		message:      "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
		signature:    "7EB0509757E246F19449885651611CB965ECC1A187DD51B64FDA1EDC9637D5EC97582B9CB13DB3933705B32BA982AF5AF25FD78881EBB32771FC5922EFC66EA3",
		verifyResult: true,
		validPubKey:  true,
	},
	{
		publicKey:    "D69C3509BB99E412E68B0FE8544E72837DFA30746D8BE2AA65975F29D22DC7B9",
		message:      "4DF3C3F68FCC83B27E9D42C90431A72499F17875C81A599B566C9889B9696703",
		signature:    "00000000000000000000003B78CE563F89A0ED9414F5AA28AD0D96D6795F9C6376AFB1548AF603B3EB45C9F8207DEE1060CB71C04E80F593060B07D28308D7F4",
		verifyResult: true,
		validPubKey:  true,
	},
	{
		publicKey:    "EEFDEA4CDB677750A420FEE807EACF21EB9898AE79B9768766E4FAA04A2D4A34",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "6CFF5C3BA86C69EA4B7376F31A9BCB4F74C1976089B2D9963DA2E5543E17776969E89B4C5564D00349106B8497785DD7D1D713A8AE82B32FA79D5F7FC407D39B",
		verifyResult: false,
		validPubKey:  false,
		expectErr:    secp_ecdsa.ErrPubKeyNotOnCurve,
	},
	{
		publicKey:    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "FFF97BD5755EEEA420453A14355235D382F6472F8568A18B2F057A14602975563CC27944640AC607CD107AE10923D9EF7A73C643E166BE5EBEAFA34B1AC553E2",
		verifyResult: false,
		validPubKey:  true,
		expectErr:    ecdsa_schnorr.ErrSigRYIsOdd,
	},
	{
		publicKey:    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "1FA62E331EDBC21C394792D2AB1100A7B432B013DF3F6FF4F99FCB33E0E1515F28890B3EDB6E7189B630448B515CE4F8622A954CFE545735AAEA5134FCCDB2BD",
		verifyResult: false,
		validPubKey:  true,
		expectErr:    ecdsa_schnorr.ErrSigRYIsOdd,
	},
	{
		publicKey:    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "6CFF5C3BA86C69EA4B7376F31A9BCB4F74C1976089B2D9963DA2E5543E177769961764B3AA9B2FFCB6EF947B6887A226E8D7C93E00C5ED0C1834FF0D0C2E6DA6",
		verifyResult: false,
		validPubKey:  true,
		expectErr:    ecdsa_schnorr.ErrUnequalRValues,
	},
	{
		publicKey:    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "0000000000000000000000000000000000000000000000000000000000000000123DDA8328AF9C23A94C1FEECFD123BA4FB73476F0D594DCB65C6425BD186051",
		verifyResult: false,
		validPubKey:  true,
		expectErr:    ecdsa_schnorr.ErrSigRNotOnCurve,
	},
	{
		publicKey:    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "00000000000000000000000000000000000000000000000000000000000000017615FBAF5AE28864013C099742DEADB4DBA87F11AC6754F93780D5A1837CF197",
		verifyResult: false,
		validPubKey:  true,
		expectErr:    ecdsa_schnorr.ErrSigRNotOnCurve,
	},
	{
		publicKey:    "DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "4A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1D69E89B4C5564D00349106B8497785DD7D1D713A8AE82B32FA79D5F7FC407D39B",
		verifyResult: false,
		validPubKey:  true,
		expectErr:    ecdsa_schnorr.ErrUnequalRValues,
	},
	{
		publicKey:    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC30",
		message:      "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
		signature:    "6CFF5C3BA86C69EA4B7376F31A9BCB4F74C1976089B2D9963DA2E5543E17776969E89B4C5564D00349106B8497785DD7D1D713A8AE82B32FA79D5F7FC407D39B",
		verifyResult: false,
		validPubKey:  false,
		expectErr:    secp_ecdsa.ErrPubKeyXTooBig,
	},
}

// decodeHex decodes the passed hex string and returns the resulting bytes.  It
// panics if an error occurs.  This is only used in the tests as a helper since
// the only way it can fail is if there is an error in the test source code.
func decodeHex(hexStr string) []byte {
	b, err := hex.DecodeString(hexStr)
	if err != nil {
		panic("invalid hex string in test source: err " + err.Error() +
			", hex: " + hexStr)
	}

	return b
}

func TestSchnorrSign(t *testing.T) {
	t.Parallel()

	for i, test := range bip340TestVectors {
		if len(test.secretKey) == 0 {
			continue
		}

		d := decodeHex(test.secretKey)
		privKey, _ := btcec.PrivKeyFromBytes(d)

		var auxBytes [32]byte
		aux := decodeHex(test.auxRand)
		copy(auxBytes[:], aux)

		msg := decodeHex(test.message)

		var signOpts []SignOption
		if !test.rfc6979 {
			signOpts = []SignOption{CustomNonce(auxBytes)}
		}

		sig, err := Sign(privKey, msg, signOpts...)
		if err != nil {
			t.Fatalf("test #%v: sig generation failed: %v", i, err)
		}

		if strings.ToUpper(hex.EncodeToString(sig.Serialize())) != test.signature {
			t.Fatalf("test #%v: got signature %x : "+
				"want %s", i, sig.Serialize(), test.signature)
		}

		pubKeyBytes := decodeHex(test.publicKey)
		err = schnorrVerify(sig, msg, pubKeyBytes)
		if err != nil {
			t.Fail()
		}

		verify := err == nil
		if test.verifyResult != verify {
			t.Fatalf("test #%v: verification mismatch: "+
				"expected %v, got %v", i, test.verifyResult, verify)
		}
	}
}

func TestSchnorrVerify(t *testing.T) {
	t.Parallel()

	for i, test := range bip340TestVectors {

		pubKeyBytes := decodeHex(test.publicKey)

		_, err := ParsePubKey(pubKeyBytes)
		switch {
		case !test.validPubKey && err != nil:
			if !errors.Is(err, test.expectErr) {
				t.Fatalf("test #%v: pubkey validation should "+
					"have failed, expected %v, got %v", i,
					test.expectErr, err)
			}

			continue

		case err != nil:
			t.Fatalf("test #%v: unable to parse pubkey: %v", i, err)
		}

		msg := decodeHex(test.message)

		sig, err := ParseSignature(decodeHex(test.signature))
		if err != nil {
			t.Fatalf("unable to parse sig: %v", err)
		}

		err = schnorrVerify(sig, msg, pubKeyBytes)
		if err != nil && test.verifyResult {
			t.Fatalf("test #%v: verification shouldn't have failed: %v", i, err)
		}

		verify := err == nil
		if test.verifyResult != verify {
			t.Fatalf("test #%v: verification mismatch: expected "+
				"%v, got %v", i, test.verifyResult, verify)
		}

		if !test.verifyResult && test.expectErr != nil {
			if !errors.Is(err, test.expectErr) {
				t.Fatalf("test #%v: expect error %v : got %v", i, test.expectErr, err)
			}
		}
	}
}

// TestSchnorrSignNoMutate tests that generating a schnorr signature doesn't
// modify/mutate the underlying private key.
func TestSchnorrSignNoMutate(t *testing.T) {
	t.Parallel()

	// Assert that given a random private key and message, we can generate
	// a signature from that w/o modifying the underlying private key.
	f := func(privBytes, msg [32]byte) bool {
		privBytesCopy := privBytes
		privKey, _ := btcec.PrivKeyFromBytes(privBytesCopy[:])

		// Generate a signature for private key with our message.
		_, err := Sign(privKey, msg[:])
		if err != nil {
			t.Logf("unable to gen sig: %v", err)
			return false
		}

		// We should be able to re-derive the private key from raw
		// bytes and have that match up again.
		privKeyCopy, _ := btcec.PrivKeyFromBytes(privBytes[:])
		if *privKey != *privKeyCopy {
			t.Logf("private doesn't match: expected %v, got %v",
				spew.Sdump(privKeyCopy), spew.Sdump(privKey))
			return false
		}

		return true
	}

	if err := quick.Check(f, nil); err != nil {
		t.Fatalf("private key modified: %v", err)
	}
}
