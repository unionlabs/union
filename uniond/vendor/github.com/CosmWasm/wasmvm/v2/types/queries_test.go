package types

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestDelegationWithEmptyArray(t *testing.T) {
	var del Array[Delegation]
	bz, err := json.Marshal(&del)
	require.NoError(t, err)
	assert.Equal(t, string(bz), `[]`)

	var redel Array[Delegation]
	err = json.Unmarshal(bz, &redel)
	require.NoError(t, err)
	assert.Equal(t, Array[Delegation]{}, redel)
}

func TestDelegationWithData(t *testing.T) {
	del := Array[Delegation]{{
		Validator: "foo",
		Delegator: "bar",
		Amount:    NewCoin(123, "stake"),
	}}
	bz, err := json.Marshal(&del)
	require.NoError(t, err)

	var redel Array[Delegation]
	err = json.Unmarshal(bz, &redel)
	require.NoError(t, err)
	assert.Equal(t, redel, del)
}

func TestValidatorWithEmptyArray(t *testing.T) {
	var val Array[Validator]
	bz, err := json.Marshal(&val)
	require.NoError(t, err)
	assert.Equal(t, string(bz), `[]`)

	var reval Array[Validator]
	err = json.Unmarshal(bz, &reval)
	require.NoError(t, err)
	assert.Equal(t, Array[Validator]{}, reval)
}

func TestValidatorWithData(t *testing.T) {
	val := Array[Validator]{{
		Address:       "1234567890",
		Commission:    "0.05",
		MaxCommission: "0.1",
		MaxChangeRate: "0.02",
	}}
	bz, err := json.Marshal(&val)
	require.NoError(t, err)

	var reval Array[Validator]
	err = json.Unmarshal(bz, &reval)
	require.NoError(t, err)
	assert.Equal(t, reval, val)
}

func TestQueryResultWithEmptyData(t *testing.T) {
	cases := map[string]struct {
		req       QueryResult
		resp      string
		unmarshal bool
	}{
		"ok with data": {
			req: QueryResult{Ok: []byte("foo")},
			// base64-encoded "foo"
			resp:      `{"ok":"Zm9v"}`,
			unmarshal: true,
		},
		"error": {
			req:       QueryResult{Err: "try again later"},
			resp:      `{"error":"try again later"}`,
			unmarshal: true,
		},
		"ok with empty slice": {
			req:       QueryResult{Ok: []byte{}},
			resp:      `{"ok":""}`,
			unmarshal: true,
		},
		"nil data": {
			req:  QueryResult{},
			resp: `{"ok":""}`,
			// Once converted to the Rust enum `ContractResult<Binary>` or
			// its JSON serialization, we cannot differentiate between
			// nil and an empty slice anymore. As a consequence,
			// only this or the above deserialization test can be executed.
			// We prefer empty slice over nil for no reason.
			unmarshal: false,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			data, err := json.Marshal(tc.req)
			require.NoError(t, err)
			require.Equal(t, tc.resp, string(data))

			// if unmarshall, make sure this comes back to the proper state
			if tc.unmarshal {
				var parsed QueryResult
				err = json.Unmarshal(data, &parsed)
				require.NoError(t, err)
				require.Equal(t, tc.req, parsed)
			}
		})
	}
}

func TestWasmQuerySerialization(t *testing.T) {
	var err error

	// ContractInfo
	document := []byte(`{"contract_info":{"contract_addr":"aabbccdd456"}}`)
	var query WasmQuery
	err = json.Unmarshal(document, &query)
	require.NoError(t, err)

	require.Nil(t, query.Smart)
	require.Nil(t, query.Raw)
	require.Nil(t, query.CodeInfo)
	require.NotNil(t, query.ContractInfo)
	require.Equal(t, "aabbccdd456", query.ContractInfo.ContractAddr)

	// CodeInfo
	document = []byte(`{"code_info":{"code_id":70}}`)
	query = WasmQuery{}
	err = json.Unmarshal(document, &query)
	require.NoError(t, err)

	require.Nil(t, query.Smart)
	require.Nil(t, query.Raw)
	require.Nil(t, query.ContractInfo)
	require.NotNil(t, query.CodeInfo)
	require.Equal(t, uint64(70), query.CodeInfo.CodeID)
}

func TestContractInfoResponseSerialization(t *testing.T) {
	document := []byte(`{"code_id":67,"creator":"jane","admin":"king","pinned":true,"ibc_port":"wasm.123"}`)
	var res ContractInfoResponse
	err := json.Unmarshal(document, &res)
	require.NoError(t, err)

	require.Equal(t, ContractInfoResponse{
		CodeID:  uint64(67),
		Creator: "jane",
		Admin:   "king",
		Pinned:  true,
		IBCPort: "wasm.123",
	}, res)
}

func TestDistributionQuerySerialization(t *testing.T) {
	var err error

	// Deserialization
	document := []byte(`{"delegator_withdraw_address":{"delegator_address":"jane"}}`)
	var query DistributionQuery
	err = json.Unmarshal(document, &query)
	require.NoError(t, err)
	require.Equal(t, query, DistributionQuery{
		DelegatorWithdrawAddress: &DelegatorWithdrawAddressQuery{
			DelegatorAddress: "jane",
		},
	})

	// Serialization
	res := DelegatorWithdrawAddressResponse{
		WithdrawAddress: "jane",
	}
	serialized, err := json.Marshal(res)
	require.NoError(t, err)
	require.Equal(t, string(serialized), `{"withdraw_address":"jane"}`)
}

func TestCodeInfoResponseSerialization(t *testing.T) {
	// Deserializaton
	document := []byte(`{"code_id":67,"creator":"jane","checksum":"f7bb7b18fb01bbf425cf4ed2cd4b7fb26a019a7fc75a4dc87e8a0b768c501f00"}`)
	var res CodeInfoResponse
	err := json.Unmarshal(document, &res)
	require.NoError(t, err)
	require.Equal(t, CodeInfoResponse{
		CodeID:   uint64(67),
		Creator:  "jane",
		Checksum: ForceNewChecksum("f7bb7b18fb01bbf425cf4ed2cd4b7fb26a019a7fc75a4dc87e8a0b768c501f00"),
	}, res)

	// Serialization
	myRes := CodeInfoResponse{
		CodeID:   uint64(0),
		Creator:  "sam",
		Checksum: ForceNewChecksum("ea4140c2d8ff498997f074cbe4f5236e52bc3176c61d1af6938aeb2f2e7b0e6d"),
	}
	serialized, err := json.Marshal(&myRes)
	require.NoError(t, err)
	require.Equal(t, `{"code_id":0,"creator":"sam","checksum":"ea4140c2d8ff498997f074cbe4f5236e52bc3176c61d1af6938aeb2f2e7b0e6d"}`, string(serialized))
}
