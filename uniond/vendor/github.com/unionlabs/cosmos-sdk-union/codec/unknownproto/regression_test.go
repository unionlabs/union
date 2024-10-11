package unknownproto_test

import (
	"encoding/hex"
	"io"
	"testing"

	"github.com/stretchr/testify/require"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
)

// Issue #7739: Catch parse errors resulting from unexpected EOF in
// protowire.ConsumeFieldValue. Discovered from fuzzing.
func TestBadBytesPassedIntoDecoder(t *testing.T) {
	data, _ := hex.DecodeString("0A9F010A9C200A2D2F6962632E636F72652E636F6E6E656374696F6E2E76312E4D7367436F6E6E656374696F584F75656E496E6974126B0A0D6962637A65726F636C69656E74120B6962637A65726F636F6E6E1A1C0A0C6962636F6E65636C69656E74120A6962636F6E65636F6E6E00002205312E302E302A283235454635364341373935313335453430393336384536444238313130463232413442453035433212080A0612040A0208011A40143342993E25DA936CDDC7BE3D8F603CA6E9661518D536D0C482E18A0154AA096E438A6B9BCADFCFC2F0D689DCCAF55B96399D67A8361B70F5DA13091E2F929")
	cfg := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	decoder := cfg.TxConfig.TxDecoder()
	tx, err := decoder(data)

	// TODO: When issue https://github.com/cosmos/cosmos-sdk/issues/7846
	// is addressed, we'll remove this .Contains check.
	require.Contains(t, err.Error(), io.ErrUnexpectedEOF.Error())
	require.Nil(t, tx)
}
