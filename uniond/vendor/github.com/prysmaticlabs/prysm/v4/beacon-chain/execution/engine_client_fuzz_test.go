//go:build go1.18
// +build go1.18

package execution_test

import (
	"encoding/json"
	"fmt"
	"math"
	"math/big"
	"reflect"
	"testing"

	"github.com/ethereum/go-ethereum/beacon/engine"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	pb "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func FuzzForkChoiceResponse(f *testing.F) {
	valHash := common.Hash([32]byte{0xFF, 0x01})
	payloadID := engine.PayloadID([8]byte{0x01, 0xFF, 0xAA, 0x00, 0xEE, 0xFE, 0x00, 0x00})
	valErr := "asjajshjahsaj"
	seed := &engine.ForkChoiceResponse{
		PayloadStatus: engine.PayloadStatusV1{
			Status:          "INVALID_TERMINAL_BLOCK",
			LatestValidHash: &valHash,
			ValidationError: &valErr,
		},
		PayloadID: &payloadID,
	}
	output, err := json.Marshal(seed)
	assert.NoError(f, err)
	f.Add(output)
	f.Fuzz(func(t *testing.T, jsonBlob []byte) {
		gethResp := &engine.ForkChoiceResponse{}
		prysmResp := &execution.ForkchoiceUpdatedResponse{}
		gethErr := json.Unmarshal(jsonBlob, gethResp)
		prysmErr := json.Unmarshal(jsonBlob, prysmResp)
		assert.Equal(t, gethErr != nil, prysmErr != nil, fmt.Sprintf("geth and prysm unmarshaller return inconsistent errors. %v and %v", gethErr, prysmErr))
		// Nothing to marshal if we have an error.
		if gethErr != nil {
			return
		}
		gethBlob, gethErr := json.Marshal(gethResp)
		prysmBlob, prysmErr := json.Marshal(prysmResp)
		assert.Equal(t, gethErr != nil, prysmErr != nil, "geth and prysm unmarshaller return inconsistent errors")
		newGethResp := &engine.ForkChoiceResponse{}
		newGethErr := json.Unmarshal(prysmBlob, newGethResp)
		assert.NoError(t, newGethErr)
		if newGethResp.PayloadStatus.Status == "UNKNOWN" {
			return
		}

		newGethResp2 := &engine.ForkChoiceResponse{}
		newGethErr = json.Unmarshal(gethBlob, newGethResp2)
		assert.NoError(t, newGethErr)

		assert.DeepEqual(t, newGethResp.PayloadID, newGethResp2.PayloadID)
		assert.DeepEqual(t, newGethResp.PayloadStatus.Status, newGethResp2.PayloadStatus.Status)
		assert.DeepEqual(t, newGethResp.PayloadStatus.LatestValidHash, newGethResp2.PayloadStatus.LatestValidHash)
		isNilOrEmpty := newGethResp.PayloadStatus.ValidationError == nil || (*newGethResp.PayloadStatus.ValidationError == "")
		isNilOrEmpty2 := newGethResp2.PayloadStatus.ValidationError == nil || (*newGethResp2.PayloadStatus.ValidationError == "")
		assert.DeepEqual(t, isNilOrEmpty, isNilOrEmpty2)
		if !isNilOrEmpty {
			assert.DeepEqual(t, *newGethResp.PayloadStatus.ValidationError, *newGethResp2.PayloadStatus.ValidationError)
		}
	})
}

func FuzzExecutionPayload(f *testing.F) {
	logsBloom := [256]byte{'j', 'u', 'n', 'k'}
	execData := &engine.ExecutionPayloadEnvelope{
		ExecutionPayload: &engine.ExecutableData{
			ParentHash:    common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			FeeRecipient:  common.Address([20]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF}),
			StateRoot:     common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			ReceiptsRoot:  common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			LogsBloom:     logsBloom[:],
			Random:        common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			Number:        math.MaxUint64,
			GasLimit:      math.MaxUint64,
			GasUsed:       math.MaxUint64,
			Timestamp:     100,
			ExtraData:     []byte{},
			BaseFeePerGas: big.NewInt(math.MaxInt),
			BlockHash:     common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			Transactions:  [][]byte{{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}, {0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}, {0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}, {0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}},
			Withdrawals:   []*types.Withdrawal{},
		},
		BlockValue: nil,
	}
	output, err := json.Marshal(execData)
	assert.NoError(f, err)
	f.Add(output)
	f.Fuzz(func(t *testing.T, jsonBlob []byte) {
		gethResp := &engine.ExecutionPayloadEnvelope{}
		prysmResp := &pb.ExecutionPayloadCapellaWithValue{}
		gethErr := json.Unmarshal(jsonBlob, gethResp)
		prysmErr := json.Unmarshal(jsonBlob, prysmResp)
		assert.Equal(t, gethErr != nil, prysmErr != nil, fmt.Sprintf("geth and prysm unmarshaller return inconsistent errors. %v and %v", gethErr, prysmErr))
		// Nothing to marshal if we have an error.
		if gethErr != nil {
			return
		}
		gethBlob, gethErr := json.Marshal(gethResp)
		prysmBlob, prysmErr := json.Marshal(prysmResp)
		assert.Equal(t, gethErr != nil, prysmErr != nil, "geth and prysm unmarshaller return inconsistent errors")
		newGethResp := &engine.ExecutionPayloadEnvelope{}
		newGethErr := json.Unmarshal(prysmBlob, newGethResp)
		assert.NoError(t, newGethErr)
		newGethResp2 := &engine.ExecutionPayloadEnvelope{}
		newGethErr = json.Unmarshal(gethBlob, newGethResp2)
		assert.NoError(t, newGethErr)

		assert.DeepEqual(t, newGethResp, newGethResp2)
	})
}

func FuzzExecutionBlock(f *testing.F) {
	f.Skip("Is skipped until false positive rate can be resolved.")
	logsBloom := [256]byte{'j', 'u', 'n', 'k'}
	addr := common.HexToAddress("095e7baea6a6c7c4c2dfeb977efac326af552d87")
	innerData := &types.DynamicFeeTx{
		ChainID:   big.NewInt(math.MaxInt),
		Nonce:     math.MaxUint64,
		GasTipCap: big.NewInt(math.MaxInt),
		GasFeeCap: big.NewInt(math.MaxInt),
		Gas:       math.MaxUint64,
		To:        &addr,
		Value:     big.NewInt(math.MaxInt),
		Data:      []byte{'r', 'a', 'n', 'd', 'o', 'm'},

		// Signature values
		V: big.NewInt(0),
		R: big.NewInt(math.MaxInt),
		S: big.NewInt(math.MaxInt),
	}
	tx := types.NewTx(innerData)
	execBlock := &pb.ExecutionBlock{
		Header: types.Header{
			ParentHash:  common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			Root:        common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			ReceiptHash: common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
			Bloom:       types.Bloom(logsBloom),
			Number:      big.NewInt(math.MaxInt),
			GasLimit:    math.MaxUint64,
			GasUsed:     math.MaxUint64,
			Time:        100,
			Extra:       nil,
			BaseFee:     big.NewInt(math.MaxInt),
			Difficulty:  big.NewInt(math.MaxInt),
		},
		Hash:            common.Hash([32]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01}),
		TotalDifficulty: "999999999999999999999999999999999999999",
		Transactions:    []*types.Transaction{tx, tx},
	}
	output, err := json.Marshal(execBlock)
	assert.NoError(f, err)

	f.Add(output)

	f.Fuzz(func(t *testing.T, jsonBlob []byte) {
		gethResp := make(map[string]interface{})
		prysmResp := &pb.ExecutionBlock{}
		gethErr := json.Unmarshal(jsonBlob, &gethResp)
		prysmErr := json.Unmarshal(jsonBlob, prysmResp)
		// Nothing to marshal if we have an error.
		if gethErr != nil || prysmErr != nil {
			return
		}
		// Exit early if fuzzer is inserting bogus hashes in.
		if isBogusTransactionHash(prysmResp, gethResp) {
			return
		}
		// Exit early if fuzzer provides bogus fields.
		valid, err := jsonFieldsAreValid(prysmResp, gethResp)
		assert.NoError(t, err)
		if !valid {
			return
		}
		assert.NoError(t, validateBlockConsistency(prysmResp, gethResp))

		gethBlob, gethErr := json.Marshal(gethResp)
		prysmBlob, prysmErr := json.Marshal(prysmResp)
		assert.Equal(t, gethErr != nil, prysmErr != nil, "geth and prysm unmarshaller return inconsistent errors")
		newGethResp := make(map[string]interface{})
		newGethErr := json.Unmarshal(prysmBlob, &newGethResp)
		assert.NoError(t, newGethErr)
		newGethResp2 := make(map[string]interface{})
		newGethErr = json.Unmarshal(gethBlob, &newGethResp2)
		assert.NoError(t, newGethErr)

		assert.DeepEqual(t, newGethResp, newGethResp2)
		compareHeaders(t, jsonBlob)
	})
}

func isBogusTransactionHash(blk *pb.ExecutionBlock, jsonMap map[string]interface{}) bool {
	if blk.Transactions == nil {
		return false
	}

	for i, tx := range blk.Transactions {
		jsonTx, ok := jsonMap["transactions"].([]interface{})[i].(map[string]interface{})
		if !ok {
			return true
		}
		// Fuzzer removed hash field.
		if _, ok := jsonTx["hash"]; !ok {
			return true
		}
		if tx.Hash().String() != jsonTx["hash"].(string) {
			return true
		}
	}
	return false
}

func compareHeaders(t *testing.T, jsonBlob []byte) {
	gethResp := &types.Header{}
	prysmResp := &pb.ExecutionBlock{}
	gethErr := json.Unmarshal(jsonBlob, gethResp)
	prysmErr := json.Unmarshal(jsonBlob, prysmResp)
	assert.Equal(t, gethErr != nil, prysmErr != nil, fmt.Sprintf("geth and prysm unmarshaller return inconsistent errors. %v and %v", gethErr, prysmErr))
	// Nothing to marshal if we have an error.
	if gethErr != nil {
		return
	}

	gethBlob, gethErr := json.Marshal(gethResp)
	prysmBlob, prysmErr := json.Marshal(prysmResp.Header)
	assert.Equal(t, gethErr != nil, prysmErr != nil, "geth and prysm unmarshaller return inconsistent errors")
	newGethResp := &types.Header{}
	newGethErr := json.Unmarshal(prysmBlob, newGethResp)
	assert.NoError(t, newGethErr)
	newGethResp2 := &types.Header{}
	newGethErr = json.Unmarshal(gethBlob, newGethResp2)
	assert.NoError(t, newGethErr)

	assert.DeepEqual(t, newGethResp, newGethResp2)
}

func validateBlockConsistency(execBlock *pb.ExecutionBlock, jsonMap map[string]interface{}) error {
	blockVal := reflect.ValueOf(execBlock).Elem()
	bType := reflect.TypeOf(execBlock).Elem()

	fieldnum := bType.NumField()

	for i := 0; i < fieldnum; i++ {
		field := bType.Field(i)
		fName := field.Tag.Get("json")
		if field.Name == "Header" {
			continue
		}
		if fName == "" {
			return errors.Errorf("Field %s had no json tag", field.Name)
		}
		fVal, ok := jsonMap[fName]
		if !ok {
			return errors.Errorf("%s doesn't exist in json map for field %s", fName, field.Name)
		}
		jsonVal := fVal
		bVal := blockVal.Field(i).Interface()
		if field.Name == "Hash" {
			jsonVal = common.HexToHash(jsonVal.(string))
		}
		if field.Name == "Transactions" {
			continue
		}
		if !reflect.DeepEqual(jsonVal, bVal) {
			return errors.Errorf("fields don't match, %v and %v are not equal for field %s", jsonVal, bVal, field.Name)
		}
	}
	return nil
}

func jsonFieldsAreValid(execBlock *pb.ExecutionBlock, jsonMap map[string]interface{}) (bool, error) {
	bType := reflect.TypeOf(execBlock).Elem()

	fieldnum := bType.NumField()

	for i := 0; i < fieldnum; i++ {
		field := bType.Field(i)
		fName := field.Tag.Get("json")
		if field.Name == "Header" {
			continue
		}
		if fName == "" {
			return false, errors.Errorf("Field %s had no json tag", field.Name)
		}
		_, ok := jsonMap[fName]
		if !ok {
			return false, nil
		}
	}
	return true, nil
}
