//go:build ledger
// +build ledger

package ledger

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var ac = codectestutil.CodecOptions{}.GetAddressCodec()

func TestPublicKeyUnsafe(t *testing.T) {
	path := *hd.NewFundraiserParams(0, sdk.CoinType, 0)
	priv, err := NewPrivKeySecp256k1Unsafe(path)
	require.NoError(t, err)
	checkDefaultPubKey(t, priv)
}

func checkDefaultPubKey(t *testing.T, priv types.LedgerPrivKey) {
	t.Helper()
	require.NotNil(t, priv)
	expectedPkStr := "PubKeySecp256k1{034FEF9CD7C4C63588D3B03FEB5281B9D232CBA34D6F3D71AEE59211FFBFE1FE87}"
	require.Equal(t, "eb5ae98721034fef9cd7c4c63588d3b03feb5281b9d232cba34d6f3d71aee59211ffbfe1fe87",
		fmt.Sprintf("%x", cdc.Amino.MustMarshalBinaryBare(priv.PubKey())),
		"Is your device using test mnemonic: %s ?", testdata.TestMnemonic)
	require.Equal(t, expectedPkStr, priv.PubKey().String())
	addr, err := ac.BytesToString(priv.PubKey().Address())
	require.NoError(t, err)
	require.Equal(t, "cosmos1w34k53py5v5xyluazqpq65agyajavep2rflq6h",
		addr, "Is your device using test mnemonic: %s ?", testdata.TestMnemonic)
}

func TestPublicKeyUnsafeHDPath(t *testing.T) {
	expectedAnswers := []string{
		"PubKeySecp256k1{034FEF9CD7C4C63588D3B03FEB5281B9D232CBA34D6F3D71AEE59211FFBFE1FE87}",
		"PubKeySecp256k1{0260D0487A3DFCE9228EEE2D0D83A40F6131F551526C8E52066FE7FE1E4A509666}",
		"PubKeySecp256k1{03A2670393D02B162D0ED06A08041E80D86BE36C0564335254DF7462447EB69AB3}",
		"PubKeySecp256k1{033222FC61795077791665544A90740E8EAD638A391A3B8F9261F4A226B396C042}",
		"PubKeySecp256k1{03F577473348D7B01E7AF2F245E36B98D181BC935EC8B552CDE5932B646DC7BE04}",
		"PubKeySecp256k1{0222B1A5486BE0A2D5F3C5866BE46E05D1BDE8CDA5EA1C4C77A9BC48D2FA2753BC}",
		"PubKeySecp256k1{0377A1C826D3A03CA4EE94FC4DEA6BCCB2BAC5F2AC0419A128C29F8E88F1FF295A}",
		"PubKeySecp256k1{031B75C84453935AB76F8C8D0B6566C3FCC101CC5C59D7000BFC9101961E9308D9}",
		"PubKeySecp256k1{038905A42433B1D677CC8AFD36861430B9A8529171B0616F733659F131C3F80221}",
		"PubKeySecp256k1{038BE7F348902D8C20BC88D32294F4F3B819284548122229DECD1ADF1A7EB0848B}",
	}

	const numIters = 10

	privKeys := make([]types.LedgerPrivKey, numIters)

	// Check with device
	for i := uint32(0); i < 10; i++ {
		path := *hd.NewFundraiserParams(0, sdk.CoinType, i)
		t.Logf("Checking keys at %v\n", path)

		priv, err := NewPrivKeySecp256k1Unsafe(path)
		require.NoError(t, err)
		require.NotNil(t, priv)

		// Check other methods
		tmp := priv.(PrivKeyLedgerSecp256k1)
		require.NoError(t, tmp.ValidateKey())
		(&tmp).AssertIsPrivKeyInner()

		// in this test we are chekcking if the generated keys are correct.
		require.Equal(t, expectedAnswers[i], priv.PubKey().String(),
			"Is your device using test mnemonic: %s ?", testdata.TestMnemonic)

		// Store and restore
		serializedPk := priv.Bytes()
		require.NotNil(t, serializedPk)
		require.True(t, len(serializedPk) >= 50)

		privKeys[i] = priv
	}

	// Now check equality
	for i := 0; i < 10; i++ {
		for j := 0; j < 10; j++ {
			require.Equal(t, i == j, privKeys[i].Equals(privKeys[j]))
			require.Equal(t, i == j, privKeys[j].Equals(privKeys[i]))
		}
	}
}

func TestPublicKeySafe(t *testing.T) {
	path := *hd.NewFundraiserParams(0, sdk.CoinType, 0)
	priv, addr, err := NewPrivKeySecp256k1(path, "cosmos")

	require.NoError(t, err)
	require.NotNil(t, priv)
	require.Nil(t, ShowAddress(path, priv.PubKey(), "cosmos"))
	checkDefaultPubKey(t, priv)

	addr2, err := ac.BytesToString(priv.PubKey().Address())
	require.NoError(t, err)
	require.Equal(t, addr, addr2)
}

func TestPublicKeyHDPath(t *testing.T) {
	expectedPubKeys := []string{
		"PubKeySecp256k1{034FEF9CD7C4C63588D3B03FEB5281B9D232CBA34D6F3D71AEE59211FFBFE1FE87}",
		"PubKeySecp256k1{0260D0487A3DFCE9228EEE2D0D83A40F6131F551526C8E52066FE7FE1E4A509666}",
		"PubKeySecp256k1{03A2670393D02B162D0ED06A08041E80D86BE36C0564335254DF7462447EB69AB3}",
		"PubKeySecp256k1{033222FC61795077791665544A90740E8EAD638A391A3B8F9261F4A226B396C042}",
		"PubKeySecp256k1{03F577473348D7B01E7AF2F245E36B98D181BC935EC8B552CDE5932B646DC7BE04}",
		"PubKeySecp256k1{0222B1A5486BE0A2D5F3C5866BE46E05D1BDE8CDA5EA1C4C77A9BC48D2FA2753BC}",
		"PubKeySecp256k1{0377A1C826D3A03CA4EE94FC4DEA6BCCB2BAC5F2AC0419A128C29F8E88F1FF295A}",
		"PubKeySecp256k1{031B75C84453935AB76F8C8D0B6566C3FCC101CC5C59D7000BFC9101961E9308D9}",
		"PubKeySecp256k1{038905A42433B1D677CC8AFD36861430B9A8529171B0616F733659F131C3F80221}",
		"PubKeySecp256k1{038BE7F348902D8C20BC88D32294F4F3B819284548122229DECD1ADF1A7EB0848B}",
	}

	expectedAddrs := []string{
		"cosmos1w34k53py5v5xyluazqpq65agyajavep2rflq6h",
		"cosmos19ewxwemt6uahejvwf44u7dh6tq859tkyvarh2q",
		"cosmos1a07dzdjgjsntxpp75zg7cgatgq0udh3pcdcxm3",
		"cosmos1qvw52lmn9gpvem8welghrkc52m3zczyhlqjsl7",
		"cosmos17m78ka80fqkkw2c4ww0v4xm5nsu2drgrlm8mn2",
		"cosmos1ferh9ll9c452d2p8k2v7heq084guygkn43up9e",
		"cosmos10vf3sxmjg96rqq36axcphzfsl74dsntuehjlw5",
		"cosmos1cq83av8cmnar79h0rg7duh9gnr7wkh228a7fxg",
		"cosmos1dszhfrt226jy5rsre7e48vw9tgwe90uerfyefa",
		"cosmos1734d7qsylzrdt05muhqqtpd90j8mp4y6rzch8l",
	}

	const numIters = 10

	privKeys := make([]types.LedgerPrivKey, numIters)

	// Check with device
	for i := 0; i < len(expectedAddrs); i++ {
		path := *hd.NewFundraiserParams(0, sdk.CoinType, uint32(i))
		t.Logf("Checking keys at %s\n", path.String())

		priv, addr, err := NewPrivKeySecp256k1(path, "cosmos")
		require.NoError(t, err)
		require.NotNil(t, addr)
		require.NotNil(t, priv)

		addr2, err := ac.BytesToString(priv.PubKey().Address())
		require.NoError(t, err)

		require.Equal(t, addr2, addr)
		require.Equal(t,
			expectedAddrs[i], addr,
			"Is your device using test mnemonic: %s ?", testdata.TestMnemonic)

		// Check other methods
		tmp := priv.(PrivKeyLedgerSecp256k1)
		require.NoError(t, tmp.ValidateKey())
		(&tmp).AssertIsPrivKeyInner()

		// in this test we are chekcking if the generated keys are correct and stored in a right path.
		require.Equal(t,
			expectedPubKeys[i], priv.PubKey().String(),
			"Is your device using test mnemonic: %s ?", testdata.TestMnemonic)

		// Store and restore
		serializedPk := priv.Bytes()
		require.NotNil(t, serializedPk)
		require.True(t, len(serializedPk) >= 50)

		privKeys[i] = priv
	}

	// Now check equality
	for i := 0; i < 10; i++ {
		for j := 0; j < 10; j++ {
			require.Equal(t, i == j, privKeys[i].Equals(privKeys[j]))
			require.Equal(t, i == j, privKeys[j].Equals(privKeys[i]))
		}
	}
}

func getFakeTx(accountNumber uint32) []byte {
	tmp := fmt.Sprintf(
		`{"account_number":"%d","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"5000"},"memo":"memo","msgs":[[""]],"sequence":"6"}`,
		accountNumber)

	return []byte(tmp)
}

func TestSignaturesHD(t *testing.T) {
	for account := uint32(0); account < 100; account += 30 {
		msg := getFakeTx(account)

		path := *hd.NewFundraiserParams(account, sdk.CoinType, account/5)
		t.Logf("Checking signature at %v    ---   PLEASE REVIEW AND ACCEPT IN THE DEVICE\n", path)

		priv, err := NewPrivKeySecp256k1Unsafe(path)
		require.NoError(t, err)

		pub := priv.PubKey()
		sig, err := priv.Sign(msg)
		require.NoError(t, err)

		valid := pub.VerifySignature(msg, sig)
		require.True(t, valid, "Is your device using test mnemonic: %s ?", testdata.TestMnemonic)
	}
}

func TestRealDeviceSecp256k1(t *testing.T) {
	msg := getFakeTx(50)
	path := *hd.NewFundraiserParams(0, sdk.CoinType, 0)
	priv, err := NewPrivKeySecp256k1Unsafe(path)
	require.NoError(t, err)

	pub := priv.PubKey()
	sig, err := priv.Sign(msg)
	require.NoError(t, err)

	valid := pub.VerifySignature(msg, sig)
	require.True(t, valid)

	// now, let's serialize the public key and make sure it still works
	bs := cdc.Amino.MustMarshalBinaryBare(priv.PubKey())
	pub2, err := legacy.PubKeyFromBytes(bs)
	require.Nil(t, err, "%+v", err)

	// make sure we get the same pubkey when we load from disk
	require.Equal(t, pub, pub2)

	// signing with the loaded key should match the original pubkey
	sig, err = priv.Sign(msg)
	require.NoError(t, err)
	valid = pub.VerifySignature(msg, sig)
	require.True(t, valid)

	// make sure pubkeys serialize properly as well
	bs = legacy.Cdc.MustMarshal(pub)
	bpub, err := legacy.PubKeyFromBytes(bs)
	require.NoError(t, err)
	require.Equal(t, pub, bpub)
}
