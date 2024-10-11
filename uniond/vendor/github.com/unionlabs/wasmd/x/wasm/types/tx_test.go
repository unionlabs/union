package types

import (
	"bytes"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	firstCodeID = 1
	badAddress  = "abcd"
)

func TestStoreCodeValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, ContractAddrLen)).String()
	sdk.GetConfig().SetAddressVerifier(VerifyAddressLen())
	cases := map[string]struct {
		msg   MsgStoreCode
		valid bool
	}{
		"empty": {
			msg:   MsgStoreCode{},
			valid: false,
		},
		"correct minimal": {
			msg: MsgStoreCode{
				Sender:       goodAddress,
				WASMByteCode: []byte("foo"),
			},
			valid: true,
		},
		"missing code": {
			msg: MsgStoreCode{
				Sender: goodAddress,
			},
			valid: false,
		},
		"bad sender minimal": {
			msg: MsgStoreCode{
				Sender:       badAddress,
				WASMByteCode: []byte("foo"),
			},
			valid: false,
		},
		"correct maximal": {
			msg: MsgStoreCode{
				Sender:       goodAddress,
				WASMByteCode: []byte("foo"),
			},
			valid: true,
		},
		"invalid InstantiatePermission": {
			msg: MsgStoreCode{
				Sender:                goodAddress,
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{badAddress}},
			},
			valid: false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func TestInstantiateContractValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	sdk.GetConfig().SetAddressVerifier(VerifyAddressLen())

	cases := map[string]struct {
		msg   MsgInstantiateContract
		valid bool
	}{
		"empty": {
			msg:   MsgInstantiateContract{},
			valid: false,
		},
		"correct minimal": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte("{}"),
			},
			valid: true,
		},
		"missing code": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				Label:  "foo",
				Msg:    []byte("{}"),
			},
			valid: false,
		},
		"missing label": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				Msg:    []byte("{}"),
			},
			valid: false,
		},
		"white space ending label": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo ",
				Msg:    []byte("{}"),
			},
			valid: false,
		},
		"non printable chars ending label": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo\v",
				Msg:    []byte("{}"),
			},
			valid: false,
		},
		"non printable chars in label": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "f\voo",
				Msg:    []byte("{}"),
			},
			valid: false,
		},
		"non printable chars beginning label": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "\vfoo",
				Msg:    []byte("{}"),
			},
			valid: false,
		},
		"label too long": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				Label:  strings.Repeat("food", 33),
			},
			valid: false,
		},
		"bad sender minimal": {
			msg: MsgInstantiateContract{
				Sender: badAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte("{}"),
			},
			valid: false,
		},
		"correct maximal": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte(`{"some": "data"}`),
				Funds:  sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(200)}},
			},
			valid: true,
		},
		"negative funds": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte(`{"some": "data"}`),
				// we cannot use sdk.NewCoin() constructors as they panic on creating invalid data (before we can test)
				Funds: sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(-200)}},
			},
			valid: false,
		},
		"non json init msg": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte("invalid-json"),
			},
			valid: false,
		},
		"empty init msg": {
			msg: MsgInstantiateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
			},
			valid: false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func TestInstantiateContract2Validation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	sdk.GetConfig().SetAddressVerifier(VerifyAddressLen())

	cases := map[string]struct {
		msg   MsgInstantiateContract2
		valid bool
	}{
		"empty": {
			msg:   MsgInstantiateContract2{},
			valid: false,
		},
		"correct minimal": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte("{}"),
				Salt:   []byte{0},
			},
			valid: true,
		},
		"missing code": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				Label:  "foo",
				Msg:    []byte("{}"),
				Salt:   []byte{0},
			},
			valid: false,
		},
		"missing label": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				Msg:    []byte("{}"),
				Salt:   []byte{0},
			},
			valid: false,
		},
		"label too long": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				Label:  strings.Repeat("food", 33),
				Salt:   []byte{0},
			},
			valid: false,
		},
		"bad sender minimal": {
			msg: MsgInstantiateContract2{
				Sender: badAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte("{}"),
				Salt:   []byte{0},
			},
			valid: false,
		},
		"correct maximal": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  strings.Repeat("a", MaxLabelSize),
				Msg:    []byte(`{"some": "data"}`),
				Funds:  sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(200)}},
				Salt:   bytes.Repeat([]byte{0}, MaxSaltSize),
				FixMsg: true,
			},
			valid: true,
		},
		"negative funds": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte(`{"some": "data"}`),
				// we cannot use sdk.NewCoin() constructors as they panic on creating invalid data (before we can test)
				Funds: sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(-200)}},
				Salt:  []byte{0},
			},
			valid: false,
		},
		"non json init msg": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte("invalid-json"),
				Salt:   []byte{0},
			},
			valid: false,
		},
		"empty init msg": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Salt:   []byte{0},
			},
			valid: false,
		},
		"empty salt": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte(`{"some": "data"}`),
			},
			valid: false,
		},
		"salt too long": {
			msg: MsgInstantiateContract2{
				Sender: goodAddress,
				CodeID: firstCodeID,
				Label:  "foo",
				Msg:    []byte(`{"some": "data"}`),
				Salt:   bytes.Repeat([]byte{0}, 65),
			},
			valid: false,
		},
	}
	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func TestExecuteContractValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()

	cases := map[string]struct {
		msg   MsgExecuteContract
		valid bool
	}{
		"empty": {
			msg:   MsgExecuteContract{},
			valid: false,
		},
		"correct minimal": {
			msg: MsgExecuteContract{
				Sender:   goodAddress,
				Contract: goodAddress,
				Msg:      []byte("{}"),
			},
			valid: true,
		},
		"correct all": {
			msg: MsgExecuteContract{
				Sender:   goodAddress,
				Contract: goodAddress,
				Msg:      []byte(`{"some": "data"}`),
				Funds:    sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(200)}},
			},
			valid: true,
		},
		"bad sender": {
			msg: MsgExecuteContract{
				Sender:   badAddress,
				Contract: goodAddress,
				Msg:      []byte(`{"some": "data"}`),
			},
			valid: false,
		},
		"empty sender": {
			msg: MsgExecuteContract{
				Contract: goodAddress,
				Msg:      []byte(`{"some": "data"}`),
			},
			valid: false,
		},
		"bad contract": {
			msg: MsgExecuteContract{
				Sender:   goodAddress,
				Contract: badAddress,
				Msg:      []byte(`{"some": "data"}`),
			},
			valid: false,
		},
		"empty contract": {
			msg: MsgExecuteContract{
				Sender: goodAddress,
				Msg:    []byte(`{"some": "data"}`),
			},
			valid: false,
		},
		"negative funds": {
			msg: MsgExecuteContract{
				Sender:   goodAddress,
				Contract: goodAddress,
				Msg:      []byte(`{"some": "data"}`),
				Funds:    sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(-1)}},
			},
			valid: false,
		},
		"duplicate funds": {
			msg: MsgExecuteContract{
				Sender:   goodAddress,
				Contract: goodAddress,
				Msg:      []byte(`{"some": "data"}`),
				Funds:    sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(1)}, sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(1)}},
			},
			valid: false,
		},
		"non json msg": {
			msg: MsgExecuteContract{
				Sender:   goodAddress,
				Contract: goodAddress,
				Msg:      []byte("invalid-json"),
			},
			valid: false,
		},
		"empty msg": {
			msg: MsgExecuteContract{
				Sender:   goodAddress,
				Contract: goodAddress,
			},
			valid: false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func TestMsgUpdateAdministrator(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	otherGoodAddress := sdk.AccAddress(bytes.Repeat([]byte{0x1}, 20)).String()
	anotherGoodAddress := sdk.AccAddress(bytes.Repeat([]byte{0x2}, 20)).String()

	specs := map[string]struct {
		src    MsgUpdateAdmin
		expErr bool
	}{
		"all good": {
			src: MsgUpdateAdmin{
				Sender:   goodAddress,
				NewAdmin: otherGoodAddress,
				Contract: anotherGoodAddress,
			},
		},
		"new admin required": {
			src: MsgUpdateAdmin{
				Sender:   goodAddress,
				Contract: anotherGoodAddress,
			},
			expErr: true,
		},
		"bad sender": {
			src: MsgUpdateAdmin{
				Sender:   badAddress,
				NewAdmin: otherGoodAddress,
				Contract: anotherGoodAddress,
			},
			expErr: true,
		},
		"bad new admin": {
			src: MsgUpdateAdmin{
				Sender:   goodAddress,
				NewAdmin: badAddress,
				Contract: anotherGoodAddress,
			},
			expErr: true,
		},
		"bad contract addr": {
			src: MsgUpdateAdmin{
				Sender:   goodAddress,
				NewAdmin: otherGoodAddress,
				Contract: badAddress,
			},
			expErr: true,
		},
		"new admin same as old admin": {
			src: MsgUpdateAdmin{
				Sender:   goodAddress,
				NewAdmin: goodAddress,
				Contract: anotherGoodAddress,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgClearAdministrator(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	anotherGoodAddress := sdk.AccAddress(bytes.Repeat([]byte{0x2}, 20)).String()

	specs := map[string]struct {
		src    MsgClearAdmin
		expErr bool
	}{
		"all good": {
			src: MsgClearAdmin{
				Sender:   goodAddress,
				Contract: anotherGoodAddress,
			},
		},
		"bad sender": {
			src: MsgClearAdmin{
				Sender:   badAddress,
				Contract: anotherGoodAddress,
			},
			expErr: true,
		},
		"bad contract addr": {
			src: MsgClearAdmin{
				Sender:   goodAddress,
				Contract: badAddress,
			},
			expErr: true,
		},
		"contract missing": {
			src: MsgClearAdmin{
				Sender: goodAddress,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgMigrateContract(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	anotherGoodAddress := sdk.AccAddress(bytes.Repeat([]byte{0x2}, 20)).String()

	specs := map[string]struct {
		src    MsgMigrateContract
		expErr bool
	}{
		"all good": {
			src: MsgMigrateContract{
				Sender:   goodAddress,
				Contract: anotherGoodAddress,
				CodeID:   firstCodeID,
				Msg:      []byte("{}"),
			},
		},
		"bad sender": {
			src: MsgMigrateContract{
				Sender:   badAddress,
				Contract: anotherGoodAddress,
				CodeID:   firstCodeID,
			},
			expErr: true,
		},
		"empty sender": {
			src: MsgMigrateContract{
				Contract: anotherGoodAddress,
				CodeID:   firstCodeID,
			},
			expErr: true,
		},
		"empty code": {
			src: MsgMigrateContract{
				Sender:   goodAddress,
				Contract: anotherGoodAddress,
			},
			expErr: true,
		},
		"bad contract addr": {
			src: MsgMigrateContract{
				Sender:   goodAddress,
				Contract: badAddress,
				CodeID:   firstCodeID,
			},
			expErr: true,
		},
		"empty contract addr": {
			src: MsgMigrateContract{
				Sender: goodAddress,
				CodeID: firstCodeID,
			},
			expErr: true,
		},
		"non json migrateMsg": {
			src: MsgMigrateContract{
				Sender:   goodAddress,
				Contract: anotherGoodAddress,
				CodeID:   firstCodeID,
				Msg:      []byte("invalid json"),
			},
			expErr: true,
		},
		"empty migrateMsg": {
			src: MsgMigrateContract{
				Sender:   goodAddress,
				Contract: anotherGoodAddress,
				CodeID:   firstCodeID,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgUpdateInstantiateConfig(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	anotherGoodAddress := sdk.AccAddress(bytes.Repeat([]byte{0x2}, 20)).String()

	specs := map[string]struct {
		src    MsgUpdateInstantiateConfig
		expErr bool
	}{
		"all good": {
			src: MsgUpdateInstantiateConfig{
				Sender:                   goodAddress,
				CodeID:                   1,
				NewInstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{anotherGoodAddress}},
			},
		},
		"bad sender": {
			src: MsgUpdateInstantiateConfig{
				Sender:                   badAddress,
				CodeID:                   1,
				NewInstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{anotherGoodAddress}},
			},
			expErr: true,
		},
		"invalid NewInstantiatePermission": {
			src: MsgUpdateInstantiateConfig{
				Sender:                   goodAddress,
				CodeID:                   1,
				NewInstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{badAddress}},
			},
			expErr: true,
		},
		"missing code id": {
			src: MsgUpdateInstantiateConfig{
				Sender:                   goodAddress,
				NewInstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{anotherGoodAddress}},
			},
			expErr: true,
		},
		"missing NewInstantiatePermission": {
			src: MsgUpdateInstantiateConfig{
				Sender: goodAddress,
				CodeID: 1,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgUpdateParamsValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()

	specs := map[string]struct {
		src    MsgUpdateParams
		expErr bool
	}{
		"all good": {
			src: MsgUpdateParams{
				Authority: goodAddress,
				Params:    DefaultParams(),
			},
		},
		"bad authority": {
			src: MsgUpdateParams{
				Authority: badAddress,
				Params:    DefaultParams(),
			},
			expErr: true,
		},
		"empty authority": {
			src: MsgUpdateParams{
				Params: DefaultParams(),
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgAddCodeUploadParamsAddressesValidation(t *testing.T) {
	badAddress := "abcd"
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	goodAddress2 := strings.ToUpper(goodAddress)
	require.NotEqual(t, goodAddress, goodAddress2) // sanity check

	tooManyAddresses := make([]string, MaxAddressCount+1)
	for i := range tooManyAddresses {
		tooManyAddresses[i] = sdk.AccAddress(bytes.Repeat([]byte{byte(i)}, 20)).String()
	}

	specs := map[string]struct {
		src    MsgAddCodeUploadParamsAddresses
		expErr bool
	}{
		"all good": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress},
			},
		},
		"all good, uppercase": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress2},
			},
		},
		"bad authority": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: badAddress,
				Addresses: []string{goodAddress},
			},
			expErr: true,
		},
		"empty authority": {
			src: MsgAddCodeUploadParamsAddresses{
				Addresses: []string{goodAddress},
			},
			expErr: true,
		},
		"empty addresses": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: goodAddress,
			},
			expErr: true,
		},
		"invalid addresses": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{badAddress},
			},
			expErr: true,
		},
		"duplicate addresses": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress, goodAddress},
			},
			expErr: true,
		},
		"duplicate addresses, different casing": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress, goodAddress2},
			},
			expErr: true,
		},
		"too many addresses": {
			src: MsgAddCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: tooManyAddresses,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgRemoveCodeUploadParamsAddressesValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	goodAddress2 := strings.ToUpper(goodAddress)
	require.NotEqual(t, goodAddress, goodAddress2) // sanity check

	tooManyAddresses := make([]string, MaxAddressCount+1)
	for i := range tooManyAddresses {
		tooManyAddresses[i] = sdk.AccAddress(bytes.Repeat([]byte{byte(i)}, 20)).String()
	}

	specs := map[string]struct {
		src    MsgRemoveCodeUploadParamsAddresses
		expErr bool
	}{
		"all good": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress},
			},
		},
		"all good, uppercase": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress2},
			},
		},
		"bad authority": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: badAddress,
				Addresses: []string{goodAddress},
			},
			expErr: true,
		},
		"empty authority": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Addresses: []string{goodAddress},
			},
			expErr: true,
		},
		"empty addresses": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: goodAddress,
			},
			expErr: true,
		},
		"invalid addresses": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{badAddress},
			},
			expErr: true,
		},
		"duplicate addresses": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress, goodAddress},
			},
			expErr: true,
		},
		"duplicate addresses, different casing": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: []string{goodAddress, goodAddress2},
			},
			expErr: true,
		},
		"too many addresses": {
			src: MsgRemoveCodeUploadParamsAddresses{
				Authority: goodAddress,
				Addresses: tooManyAddresses,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgPinCodesValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	specs := map[string]struct {
		src    MsgPinCodes
		expErr bool
	}{
		"all good": {
			src: MsgPinCodes{
				Authority: goodAddress,
				CodeIDs:   []uint64{1},
			},
		},
		"bad authority": {
			src: MsgPinCodes{
				Authority: badAddress,
				CodeIDs:   []uint64{1},
			},
			expErr: true,
		},
		"empty authority": {
			src: MsgPinCodes{
				CodeIDs: []uint64{1},
			},
			expErr: true,
		},
		"empty code ids": {
			src: MsgPinCodes{
				Authority: goodAddress,
			},
			expErr: true,
		},
		"exceeds max code ids": {
			src: MsgPinCodes{
				Authority: goodAddress,
				CodeIDs:   genCodeIDs(51),
			},
			expErr: true,
		},
		"duplicate code ids": {
			src: MsgPinCodes{
				Authority: goodAddress,
				CodeIDs:   []uint64{1, 1},
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgUnpinCodesValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()

	specs := map[string]struct {
		src    MsgUnpinCodes
		expErr bool
	}{
		"all good": {
			src: MsgUnpinCodes{
				Authority: goodAddress,
				CodeIDs:   []uint64{1},
			},
		},
		"bad authority": {
			src: MsgUnpinCodes{
				Authority: badAddress,
				CodeIDs:   []uint64{1},
			},
			expErr: true,
		},
		"empty authority": {
			src: MsgUnpinCodes{
				CodeIDs: []uint64{1},
			},
			expErr: true,
		},
		"empty code ids": {
			src: MsgUnpinCodes{
				Authority: goodAddress,
			},
			expErr: true,
		},
		"exceeds max code ids": {
			src: MsgUnpinCodes{
				Authority: goodAddress,
				CodeIDs:   genCodeIDs(51),
			},
			expErr: true,
		},
		"duplicate code ids": {
			src: MsgUnpinCodes{
				Authority: goodAddress,
				CodeIDs:   []uint64{1, 1},
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func genCodeIDs(max int) []uint64 {
	r := make([]uint64, max)
	for i := 0; i < max; i++ {
		r[i] = uint64(i)
	}
	return r
}

func TestMsgSudoContractValidation(t *testing.T) {
	badAddress := "abcd"
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	anotherGoodAddress := sdk.AccAddress(bytes.Repeat([]byte{0x2}, 20)).String()

	specs := map[string]struct {
		src    MsgSudoContract
		expErr bool
	}{
		"all good": {
			src: MsgSudoContract{
				Authority: goodAddress,
				Contract:  anotherGoodAddress,
				Msg:       []byte("{}"),
			},
		},
		"bad authority": {
			src: MsgSudoContract{
				Authority: badAddress,
				Contract:  anotherGoodAddress,
				Msg:       []byte("{}"),
			},
			expErr: true,
		},
		"empty authority": {
			src: MsgSudoContract{
				Contract: anotherGoodAddress,
				Msg:      []byte("{}"),
			},
			expErr: true,
		},
		"bad contract addr": {
			src: MsgSudoContract{
				Authority: goodAddress,
				Contract:  badAddress,
				Msg:       []byte("{}"),
			},
			expErr: true,
		},
		"empty contract addr": {
			src: MsgSudoContract{
				Authority: goodAddress,
				Msg:       []byte("{}"),
			},
			expErr: true,
		},
		"non json sudoMsg": {
			src: MsgSudoContract{
				Authority: goodAddress,
				Contract:  anotherGoodAddress,
				Msg:       []byte("invalid json"),
			},
			expErr: true,
		},
		"empty sudoMsg": {
			src: MsgSudoContract{
				Authority: goodAddress,
				Contract:  anotherGoodAddress,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}

func TestMsgStoreAndInstantiateContractValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	sdk.GetConfig().SetAddressVerifier(VerifyAddressLen())

	cases := map[string]struct {
		msg   MsgStoreAndInstantiateContract
		valid bool
	}{
		"empty": {
			msg:   MsgStoreAndInstantiateContract{},
			valid: false,
		},
		"correct minimal": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        "foo",
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
			},
			valid: true,
		},
		"missing byte code": {
			msg: MsgStoreAndInstantiateContract{
				Authority: goodAddress,
				Label:     "foo",
				Msg:       []byte("{}"),
			},
			valid: false,
		},
		"missing label": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
			},
			valid: false,
		},
		"label too long": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        strings.Repeat("food", 33),
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
			},
			valid: false,
		},
		"bad sender minimal": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    badAddress,
				Label:        "foo",
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
			},
			valid: false,
		},
		"bad admin": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        "foo",
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
				Admin:        badAddress,
			},
			valid: false,
		},
		"correct maximal": {
			msg: MsgStoreAndInstantiateContract{
				Authority:             goodAddress,
				Label:                 "foo",
				Msg:                   []byte(`{"some": "data"}`),
				Funds:                 sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(200)}},
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AllowEverybody,
				UnpinCode:             true,
				Admin:                 goodAddress,
				Source:                "http://source.com",
				Builder:               "builder",
				CodeHash:              []byte("{}"),
			},
			valid: true,
		},
		"invalid source": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        "foo",
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
				Admin:        goodAddress,
				Source:       "source",
			},
			valid: false,
		},
		"empty builder": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        "foo",
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
				Admin:        goodAddress,
				Source:       "http://source.com",
			},
			valid: false,
		},
		"empty code hash": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        "foo",
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
				Admin:        goodAddress,
				Source:       "http://source.com",
				Builder:      "builder",
			},
			valid: false,
		},
		"negative funds": {
			msg: MsgStoreAndInstantiateContract{
				Authority: goodAddress,
				Label:     "foo",
				Msg:       []byte(`{"some": "data"}`),
				// we cannot use sdk.NewCoin() constructors as they panic on creating invalid data (before we can test)
				Funds:        sdk.Coins{sdk.Coin{Denom: "foobar", Amount: sdkmath.NewInt(-200)}},
				WASMByteCode: []byte("foo"),
			},
			valid: false,
		},
		"non json init msg": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        "foo",
				Msg:          []byte("invalid-json"),
				WASMByteCode: []byte("foo"),
			},
			valid: false,
		},
		"empty init msg": {
			msg: MsgStoreAndInstantiateContract{
				Authority:    goodAddress,
				Label:        "foo",
				WASMByteCode: []byte("foo"),
			},
			valid: false,
		},
		"invalid InstantiatePermission": {
			msg: MsgStoreAndInstantiateContract{
				Authority:             goodAddress,
				WASMByteCode:          []byte("foo"),
				Label:                 "foo",
				Msg:                   []byte(`{"some": "data"}`),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{badAddress}},
			},
			valid: false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func TestMsgStoreAndMigrateContractValidation(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	sdk.GetConfig().SetAddressVerifier(VerifyAddressLen())

	cases := map[string]struct {
		msg   MsgStoreAndMigrateContract
		valid bool
	}{
		"all good": {
			msg: MsgStoreAndMigrateContract{
				Authority:             goodAddress,
				Contract:              goodAddress,
				Msg:                   []byte("{}"),
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{goodAddress}},
			},
			valid: true,
		},
		"empty InstantiatePermission": {
			msg: MsgStoreAndMigrateContract{
				Authority:    goodAddress,
				Contract:     goodAddress,
				Msg:          []byte("{}"),
				WASMByteCode: []byte("foo"),
			},
			valid: true,
		},
		"empty": {
			msg:   MsgStoreAndMigrateContract{},
			valid: false,
		},
		"missing byte code": {
			msg: MsgStoreAndMigrateContract{
				Authority:             goodAddress,
				Contract:              goodAddress,
				Msg:                   []byte("{}"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{goodAddress}},
			},
			valid: false,
		},
		"missing contract": {
			msg: MsgStoreAndMigrateContract{
				Authority:             goodAddress,
				Msg:                   []byte("{}"),
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{goodAddress}},
			},
			valid: false,
		},
		"bad contract": {
			msg: MsgStoreAndMigrateContract{
				Authority:             goodAddress,
				Contract:              badAddress,
				Msg:                   []byte("{}"),
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{goodAddress}},
			},
			valid: false,
		},
		"bad authority": {
			msg: MsgStoreAndMigrateContract{
				Authority:             badAddress,
				Contract:              goodAddress,
				Msg:                   []byte("{}"),
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{goodAddress}},
			},
			valid: false,
		},
		"non json msg": {
			msg: MsgStoreAndMigrateContract{
				Authority:             goodAddress,
				Contract:              goodAddress,
				Msg:                   []byte("invalid-json"),
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{goodAddress}},
			},
			valid: false,
		},
		"empty msg": {
			msg: MsgStoreAndMigrateContract{
				Authority:             goodAddress,
				Contract:              goodAddress,
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{goodAddress}},
			},
			valid: false,
		},
		"invalid InstantiatePermission": {
			msg: MsgStoreAndMigrateContract{
				Authority:             goodAddress,
				Contract:              goodAddress,
				Msg:                   []byte("{}"),
				WASMByteCode:          []byte("foo"),
				InstantiatePermission: &AccessConfig{Permission: AccessTypeAnyOfAddresses, Addresses: []string{badAddress}},
			},
			valid: false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.msg.ValidateBasic()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func TestMsgUpdateContractLabel(t *testing.T) {
	// proper address size
	goodAddress := sdk.AccAddress(make([]byte, 20)).String()
	otherGoodAddress := sdk.AccAddress(bytes.Repeat([]byte{0x1}, 20)).String()

	specs := map[string]struct {
		src    MsgUpdateContractLabel
		expErr bool
	}{
		"all good": {
			src: MsgUpdateContractLabel{
				Sender:   goodAddress,
				NewLabel: "new label",
				Contract: otherGoodAddress,
			},
		},
		"new label required": {
			src: MsgUpdateContractLabel{
				Sender:   goodAddress,
				Contract: otherGoodAddress,
			},
			expErr: true,
		},
		"bad sender": {
			src: MsgUpdateContractLabel{
				Sender:   badAddress,
				NewLabel: "new label",
				Contract: otherGoodAddress,
			},
			expErr: true,
		},
		"empty new label": {
			src: MsgUpdateContractLabel{
				Sender:   goodAddress,
				NewLabel: "",
				Contract: otherGoodAddress,
			},
			expErr: true,
		},
		"bad new label": {
			src: MsgUpdateContractLabel{
				Sender:   goodAddress,
				NewLabel: " start with space ",
				Contract: otherGoodAddress,
			},
			expErr: true,
		},
		"bad contract addr": {
			src: MsgUpdateContractLabel{
				Sender:   goodAddress,
				NewLabel: "new label",
				Contract: badAddress,
			},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			err := spec.src.ValidateBasic()
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
		})
	}
}
