package state_test

import (
	"fmt"
	"testing"
	"time"

	gogo "github.com/cosmos/gogoproto/types"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	abciv1 "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	abciv1beta1 "github.com/cometbft/cometbft/api/cometbft/abci/v1beta1"
	abciv1beta2 "github.com/cometbft/cometbft/api/cometbft/abci/v1beta2"
	abciv1beta3 "github.com/cometbft/cometbft/api/cometbft/abci/v1beta3"
	cryptov1 "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	statev1 "github.com/cometbft/cometbft/api/cometbft/state/v1"
	statev1beta2 "github.com/cometbft/cometbft/api/cometbft/state/v1beta2"
	statev1beta3 "github.com/cometbft/cometbft/api/cometbft/state/v1beta3"
	typesv1 "github.com/cometbft/cometbft/api/cometbft/types/v1"
	typesv1beta1 "github.com/cometbft/cometbft/api/cometbft/types/v1beta1"
	typesv1beta2 "github.com/cometbft/cometbft/api/cometbft/types/v1beta2"
	"github.com/cometbft/cometbft/crypto/ed25519"
	sm "github.com/cometbft/cometbft/state"
)

// Compatibility test across different state proto versions

func calcABCIResponsesKey(height int64) []byte {
	return []byte(fmt.Sprintf("abciResponsesKey:%v", height))
}

var lastABCIResponseKey = []byte("lastABCIResponseKey")

var (
	_ sm.Store    = (*MultiStore)(nil)
	_ LegacyStore = (*MultiStore)(nil)
)

// MultiStore represents a state store that implements the Store interface
// and contains additional store and database options.
//
// Fields:
// - Store (sm.Store): The store instance used by the MultiStore.
// - db (dbm.DB): The database instance used by the MultiStore.
// - StoreOptions (sm.StoreOptions): The options for the MultiStore.
type MultiStore struct {
	sm.Store
	db dbm.DB
	sm.StoreOptions
}

// NewMultiStore initializes a new instance of MultiStore with the provided parameters.
// It sets the store, db, and StoreOptions fields of the MultiStore struct.
//
// Parameters:
// - db (dbm.DB): The database instance to be used by the MultiStore.
// - options (sm.StoreOptions): The store options to be used by the MultiStore.
// - store (sm.Store): The store instance to be used by the MultiStore.
//
// Returns:
// - *MultiStore: A pointer to the newly created MultiStore instance.
func NewMultiStore(db dbm.DB, options sm.StoreOptions, store sm.Store) *MultiStore {
	return &MultiStore{
		Store:        store,
		db:           db,
		StoreOptions: options,
	}
}

// LegacyStore represents a legacy data store.
// Example usage:
//
//	_ LegacyStore = (*MultiStore)(nil)
type LegacyStore interface {
	SaveABCIResponses(height int64, abciResponses *statev1beta2.ABCIResponses) error
}

// SaveABCIResponses saves the ABCIResponses for a given height in the MultiStore.
// It strips out any nil values from the DeliverTxs field, and saves the ABCIResponses to
// disk if the DiscardABCIResponses flag is set to false. It also saves the last ABCI response
// for crash recovery, overwriting the previously saved response.
//
// Parameters:
// - height (int64): The height at which the ABCIResponses are being saved.
// - abciResponses (ABCIResponses): The ABCIResponses to be saved.
//
// Returns:
// - error: An error if there was a problem saving the ABCIResponses.
//
// NOTE: The MultiStore must be properly configured with the StoreOptions and db before calling this method.
func (multi MultiStore) SaveABCIResponses(height int64, abciResponses *statev1beta2.ABCIResponses) error {
	var dtxs []*abciv1beta2.ResponseDeliverTx
	// strip nil values,
	for _, tx := range abciResponses.DeliverTxs {
		if tx != nil {
			dtxs = append(dtxs, tx)
		}
	}
	abciResponses.DeliverTxs = dtxs

	// If the flag is false then we save the ABCIResponse. This can be used for the /BlockResults
	// query or to reindex an event using the command line.
	if !multi.StoreOptions.DiscardABCIResponses {
		bz, err := abciResponses.Marshal()
		if err != nil {
			return err
		}
		if err := multi.db.Set(calcABCIResponsesKey(height), bz); err != nil {
			return err
		}
	}

	// We always save the last ABCI response for crash recovery.
	// This overwrites the previous saved ABCI Response.
	response := &statev1beta2.ABCIResponsesInfo{
		AbciResponses: abciResponses,
		Height:        height,
	}
	bz, err := response.Marshal()
	if err != nil {
		return err
	}

	return multi.db.SetSync(lastABCIResponseKey, bz)
}

// TestSaveLegacyAndLoadFinalizeBlock tests saving and loading of ABCIResponses
// using the multiStore. It verifies that the loaded ABCIResponses match the
// original ones and that missing fields are correctly handled.
// This test is important for the LoadFinalizeBlockResponse method in the state store.
func TestSaveLegacyAndLoadFinalizeBlock(t *testing.T) {
	tearDown, stateDB, _, store := setupTestCaseWithStore(t)
	defer tearDown(t)
	options := sm.StoreOptions{
		DiscardABCIResponses: false,
	}

	height := int64(1)
	multiStore := NewMultiStore(stateDB, options, store)

	// try with a complete ABCI Response
	v1beta2ABCIResponses := newV1Beta2ABCIResponses()
	err := multiStore.SaveABCIResponses(height, &v1beta2ABCIResponses)
	require.NoError(t, err)
	require.Equal(t, 1, len(v1beta2ABCIResponses.DeliverTxs))
	require.Equal(t, 1, len(v1beta2ABCIResponses.BeginBlock.Events))
	require.Equal(t, 1, len(v1beta2ABCIResponses.EndBlock.Events))

	finalizeBlockResponse, err := multiStore.LoadFinalizeBlockResponse(height)
	require.NoError(t, err)

	// Test for not nil
	require.NotNil(t, finalizeBlockResponse.TxResults)
	require.NotNil(t, finalizeBlockResponse.Events)
	require.NotNil(t, finalizeBlockResponse.ValidatorUpdates)
	require.NotNil(t, finalizeBlockResponse.ConsensusParamUpdates)
	require.Nil(t, finalizeBlockResponse.AppHash)

	// Test for equality
	require.Equal(t, 1, len(finalizeBlockResponse.TxResults))
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs), len(finalizeBlockResponse.TxResults))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Code, finalizeBlockResponse.TxResults[0].Code)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Data, finalizeBlockResponse.TxResults[0].Data)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Log, finalizeBlockResponse.TxResults[0].Log)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].GasWanted, finalizeBlockResponse.TxResults[0].GasWanted)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].GasUsed, finalizeBlockResponse.TxResults[0].GasUsed)
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs[0].Events), len(finalizeBlockResponse.TxResults[0].Events))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Events[0].Type, finalizeBlockResponse.TxResults[0].Events[0].Type)
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes), len(finalizeBlockResponse.TxResults[0].Events[0].Attributes))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Key, finalizeBlockResponse.TxResults[0].Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Value, finalizeBlockResponse.TxResults[0].Events[0].Attributes[0].Value)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Codespace, finalizeBlockResponse.TxResults[0].Codespace)

	require.Equal(t, 2, len(finalizeBlockResponse.Events))
	require.Equal(t, len(v1beta2ABCIResponses.BeginBlock.Events)+len(v1beta2ABCIResponses.EndBlock.Events), len(finalizeBlockResponse.Events))

	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Type, finalizeBlockResponse.Events[0].Type)
	require.Equal(t, len(v1beta2ABCIResponses.BeginBlock.Events[0].Attributes)+1, len(finalizeBlockResponse.Events[0].Attributes)) // +1 for inject 'mode' attribute
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Attributes[0].Key, finalizeBlockResponse.Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Attributes[0].Value, finalizeBlockResponse.Events[0].Attributes[0].Value)

	require.Equal(t, v1beta2ABCIResponses.EndBlock.ConsensusParamUpdates.Block.MaxBytes, finalizeBlockResponse.ConsensusParamUpdates.Block.MaxBytes)
	require.Equal(t, v1beta2ABCIResponses.EndBlock.ConsensusParamUpdates.Block.MaxGas, finalizeBlockResponse.ConsensusParamUpdates.Block.MaxGas)
	require.Equal(t, v1beta2ABCIResponses.EndBlock.ConsensusParamUpdates.Evidence.MaxAgeNumBlocks, finalizeBlockResponse.ConsensusParamUpdates.Evidence.MaxAgeNumBlocks)
	require.Equal(t, v1beta2ABCIResponses.EndBlock.ConsensusParamUpdates.Evidence.MaxAgeDuration, finalizeBlockResponse.ConsensusParamUpdates.Evidence.MaxAgeDuration)
	require.Equal(t, v1beta2ABCIResponses.EndBlock.ConsensusParamUpdates.Evidence.MaxBytes, finalizeBlockResponse.ConsensusParamUpdates.Evidence.MaxBytes)
	require.Equal(t, v1beta2ABCIResponses.EndBlock.ConsensusParamUpdates.Validator.PubKeyTypes, finalizeBlockResponse.ConsensusParamUpdates.Validator.PubKeyTypes)
	require.Equal(t, v1beta2ABCIResponses.EndBlock.ConsensusParamUpdates.Version.App, finalizeBlockResponse.ConsensusParamUpdates.Version.App)

	require.Nil(t, finalizeBlockResponse.ConsensusParamUpdates.Abci)
	require.Nil(t, finalizeBlockResponse.ConsensusParamUpdates.Synchrony)
	require.Nil(t, finalizeBlockResponse.ConsensusParamUpdates.Feature)
	require.Nil(t, finalizeBlockResponse.AppHash)

	require.Equal(t, len(v1beta2ABCIResponses.EndBlock.ValidatorUpdates), len(finalizeBlockResponse.ValidatorUpdates))
	require.Equal(t, v1beta2ABCIResponses.EndBlock.ValidatorUpdates[0].Power, finalizeBlockResponse.ValidatorUpdates[0].Power)

	// skip until an equivalency test is possible
	// require.NotNil(t, finalizeBlockResponse.ValidatorUpdates[0].PubKeyBytes)
	// require.NotEmpty(t, finalizeBlockResponse.ValidatorUpdates[0].PubKeyType)
	// require.Equal(t, v1beta2ABCIResponses.ValidatorUpdates[0].PubKey.GetEd25519(), finalizeBlockResponse.ValidatorUpdates[0].PubKeyBytes)

	// try with an ABCI Response missing fields
	height = int64(2)
	v1beta2ABCIResponses = newV1Beta2ABCIResponsesWithNullFields()
	require.Equal(t, 1, len(v1beta2ABCIResponses.DeliverTxs))
	require.Equal(t, 1, len(v1beta2ABCIResponses.BeginBlock.Events))
	require.Nil(t, v1beta2ABCIResponses.EndBlock)
	err = multiStore.SaveABCIResponses(height, &v1beta2ABCIResponses)
	require.NoError(t, err)
	finalizeBlockResponse, err = multiStore.LoadFinalizeBlockResponse(height)
	require.NoError(t, err)

	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs), len(finalizeBlockResponse.TxResults))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].String(), finalizeBlockResponse.TxResults[0].String())
	require.Equal(t, len(v1beta2ABCIResponses.BeginBlock.Events), len(finalizeBlockResponse.Events))
}

// This test un-marshals a v1beta2.ABCIResponses as a statev1.LegacyABCIResponses
// This logic is important for the LoadFinalizeBlockResponse method in the state store
// The conversion should not fail because they are compatible.
//

func TestStateV1Beta2ABCIResponsesAsStateV1LegacyABCIResponse(t *testing.T) {
	v1beta2ABCIResponses := newV1Beta2ABCIResponses()

	v1b2Resp, err := v1beta2ABCIResponses.Marshal()
	require.NoError(t, err)
	require.NotNil(t, v1b2Resp)

	// un-marshall a v1beta2 ABCI Response as a LegacyABCIResponse
	legacyABCIResponses := new(statev1.LegacyABCIResponses)
	err = legacyABCIResponses.Unmarshal(v1b2Resp)
	require.NoError(t, err)

	// ensure not nil
	require.NotNil(t, legacyABCIResponses.DeliverTxs)
	require.NotNil(t, legacyABCIResponses.EndBlock)
	require.NotNil(t, legacyABCIResponses.BeginBlock)

	// ensure for equality
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs), len(legacyABCIResponses.DeliverTxs))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Code, legacyABCIResponses.DeliverTxs[0].Code)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Data, legacyABCIResponses.DeliverTxs[0].Data)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Log, legacyABCIResponses.DeliverTxs[0].Log)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].GasWanted, legacyABCIResponses.DeliverTxs[0].GasWanted)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].GasUsed, legacyABCIResponses.DeliverTxs[0].GasUsed)
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs[0].Events), len(legacyABCIResponses.DeliverTxs[0].Events))
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes), len(legacyABCIResponses.DeliverTxs[0].Events[0].Attributes))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Key, legacyABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Value, legacyABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Value)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Codespace, legacyABCIResponses.DeliverTxs[0].Codespace)

	require.Equal(t, len(v1beta2ABCIResponses.BeginBlock.Events), len(legacyABCIResponses.BeginBlock.Events))
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Type, legacyABCIResponses.BeginBlock.Events[0].Type)
	require.Equal(t, len(v1beta2ABCIResponses.BeginBlock.Events[0].Attributes), len(legacyABCIResponses.BeginBlock.Events[0].Attributes))
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Attributes[0].Key, legacyABCIResponses.BeginBlock.Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Attributes[0].Value, legacyABCIResponses.BeginBlock.Events[0].Attributes[0].Value)
}

// This test unmarshals a v1beta2.ABCIResponses as a v1beta3.ResponseFinalizeBlock
// This logic is important for the LoadFinalizeBlockResponse method in the state store
// The conversion should fail because they are not compatible.
func TestStateV1Beta2ABCIResponsesAsV1Beta3ResponseFinalizeBlock(t *testing.T) {
	v1beta2ABCIResponses := newV1Beta2ABCIResponses()
	data, err := v1beta2ABCIResponses.Marshal()
	require.NoError(t, err)
	require.NotNil(t, data)

	// This cannot work since they have different schemas, a wrong wireType error is generated
	responseFinalizeBlock := new(abciv1beta3.ResponseFinalizeBlock)
	err = responseFinalizeBlock.Unmarshal(data)
	require.Error(t, err)
	require.ErrorContains(t, err, "unexpected EOF")
}

// This test unmarshal a v1beta2.ABCIResponses as a v1.FinalizeBlockResponse
// This logic is important for the LoadFinalizeBlockResponse method in the state store
// The conversion should fail because they are not compatible.
func TestStateV1Beta2ABCIResponsesAsV1FinalizeBlockResponse(t *testing.T) {
	v1beta2ABCIResponses := newV1Beta2ABCIResponses()
	data, err := v1beta2ABCIResponses.Marshal()
	require.NoError(t, err)
	require.NotNil(t, data)

	// This cannot work since they have different schemas, a wrong wireType error is generated
	finalizeBlockResponse := new(abciv1.FinalizeBlockResponse)
	err = finalizeBlockResponse.Unmarshal(data)
	require.Error(t, err)
	require.ErrorContains(t, err, "unexpected EOF")
}

// This test unmarshal a v1beta2.ABCIResponses as a v1beta3.ResponseFinalizeBlock
// This logic is important for the LoadFinalizeBlockResponse method in the state store
// The conversion doesn't fail because no error is return, but they are NOT compatible.
func TestStateV1Beta2ABCIResponsesWithNullAsV1Beta3ResponseFinalizeBlock(t *testing.T) {
	v1beta2ABCIResponsesWithNull := newV1Beta2ABCIResponsesWithNullFields()
	data, err := v1beta2ABCIResponsesWithNull.Marshal()
	require.NoError(t, err)
	require.NotNil(t, data)

	// This should not work since they have different schemas
	// but an error is not returned, so it deserializes an ABCIResponse
	// on top of a FinalizeBlockResponse giving the false impression they are the same
	// but because it doesn't error out, the fields in finalizeBlockResponse will have
	// their zero-value (e.g. nil, 0, "")
	finalizeBlockResponse := new(abciv1beta3.ResponseFinalizeBlock)
	err = finalizeBlockResponse.Unmarshal(data)
	require.NoError(t, err)
	require.Nil(t, finalizeBlockResponse.AppHash)
	require.Nil(t, finalizeBlockResponse.TxResults)
}

// This test unmarshal a v1beta2.ABCIResponses as a statev1beta3.LegacyABCIResponses
// This logic is important for the LoadFinalizeBlockResponse method in the state store
// The conversion should work because they are compatible.
//

func TestStateV1Beta2ABCIResponsesAsStateV1Beta3LegacyABCIResponse(t *testing.T) {
	v1beta2ABCIResponses := newV1Beta2ABCIResponses()

	data, err := v1beta2ABCIResponses.Marshal()
	require.NoError(t, err)
	require.NotNil(t, data)

	// This works because they are equivalent protos and the fields are populated
	legacyABCIResponses := new(statev1beta3.LegacyABCIResponses)
	err = legacyABCIResponses.Unmarshal(data)
	require.NoError(t, err)

	// ensure not nil
	require.NotNil(t, legacyABCIResponses.DeliverTxs)
	require.NotNil(t, legacyABCIResponses.EndBlock)
	require.NotNil(t, legacyABCIResponses.BeginBlock)

	// ensure for equality
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs), len(legacyABCIResponses.DeliverTxs))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Code, legacyABCIResponses.DeliverTxs[0].Code)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Data, legacyABCIResponses.DeliverTxs[0].Data)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Log, legacyABCIResponses.DeliverTxs[0].Log)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].GasWanted, legacyABCIResponses.DeliverTxs[0].GasWanted)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].GasUsed, legacyABCIResponses.DeliverTxs[0].GasUsed)
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs[0].Events), len(legacyABCIResponses.DeliverTxs[0].Events))
	require.Equal(t, len(v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes), len(legacyABCIResponses.DeliverTxs[0].Events[0].Attributes))
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Key, legacyABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Value, legacyABCIResponses.DeliverTxs[0].Events[0].Attributes[0].Value)
	require.Equal(t, v1beta2ABCIResponses.DeliverTxs[0].Codespace, legacyABCIResponses.DeliverTxs[0].Codespace)

	require.Equal(t, len(v1beta2ABCIResponses.BeginBlock.Events), len(legacyABCIResponses.BeginBlock.Events))
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events, legacyABCIResponses.BeginBlock.Events)
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Type, legacyABCIResponses.BeginBlock.Events[0].Type)
	require.Equal(t, len(v1beta2ABCIResponses.BeginBlock.Events[0].Attributes), len(legacyABCIResponses.BeginBlock.Events[0].Attributes))
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Attributes[0].Key, legacyABCIResponses.BeginBlock.Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponses.BeginBlock.Events[0].Attributes[0].Value, legacyABCIResponses.BeginBlock.Events[0].Attributes[0].Value)
}

// This test unmarshal a v1beta2.ABCIResponsesWithNullFields as a statev1beta3.LegacyABCIResponses
// This logic is important for the LoadFinalizeBlockResponse method in the state store
// The conversion should work because they are compatible even if fields to be converted are null.
func TestStateV1Beta2ABCIResponsesWithNullAsStateV1Beta3LegacyABCIResponse(t *testing.T) {
	v1beta2ABCIResponsesWithNull := newV1Beta2ABCIResponsesWithNullFields()
	data, err := v1beta2ABCIResponsesWithNull.Marshal()
	require.NoError(t, err)
	require.NotNil(t, data)

	// This works because they are equivalent protos and the fields are populated
	// even if a field is null, it will be converted properly
	legacyResponseWithNull := new(statev1beta3.LegacyABCIResponses)
	err = legacyResponseWithNull.Unmarshal(data)
	require.NoError(t, err)
	require.NotNil(t, legacyResponseWithNull.DeliverTxs)
	require.Nil(t, legacyResponseWithNull.EndBlock)
	require.NotNil(t, legacyResponseWithNull.BeginBlock)

	require.Equal(t, len(v1beta2ABCIResponsesWithNull.BeginBlock.Events), len(legacyResponseWithNull.BeginBlock.Events))
	require.Equal(t, v1beta2ABCIResponsesWithNull.BeginBlock.Events, legacyResponseWithNull.BeginBlock.Events)
	require.Equal(t, v1beta2ABCIResponsesWithNull.BeginBlock.Events[0].Type, legacyResponseWithNull.BeginBlock.Events[0].Type)
	require.Equal(t, len(v1beta2ABCIResponsesWithNull.BeginBlock.Events[0].Attributes), len(legacyResponseWithNull.BeginBlock.Events[0].Attributes))
	require.Equal(t, v1beta2ABCIResponsesWithNull.BeginBlock.Events[0].Attributes[0].Key, legacyResponseWithNull.BeginBlock.Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponsesWithNull.BeginBlock.Events[0].Attributes[0].Value, legacyResponseWithNull.BeginBlock.Events[0].Attributes[0].Value)

	require.Equal(t, len(v1beta2ABCIResponsesWithNull.DeliverTxs), len(legacyResponseWithNull.DeliverTxs))
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].Code, legacyResponseWithNull.DeliverTxs[0].Code)
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].Data, legacyResponseWithNull.DeliverTxs[0].Data)
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].Log, legacyResponseWithNull.DeliverTxs[0].Log)
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].GasWanted, legacyResponseWithNull.DeliverTxs[0].GasWanted)
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].GasUsed, legacyResponseWithNull.DeliverTxs[0].GasUsed)
	require.Equal(t, len(v1beta2ABCIResponsesWithNull.DeliverTxs[0].Events), len(legacyResponseWithNull.DeliverTxs[0].Events))
	require.Equal(t, len(v1beta2ABCIResponsesWithNull.DeliverTxs[0].Events[0].Attributes), len(legacyResponseWithNull.DeliverTxs[0].Events[0].Attributes))
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].Events[0].Attributes[0].Key, legacyResponseWithNull.DeliverTxs[0].Events[0].Attributes[0].Key)
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].Events[0].Attributes[0].Value, legacyResponseWithNull.DeliverTxs[0].Events[0].Attributes[0].Value)
	require.Equal(t, v1beta2ABCIResponsesWithNull.DeliverTxs[0].Codespace, legacyResponseWithNull.DeliverTxs[0].Codespace)
}

// This test unmarshal a v1beta3.ResponseFinalizeBlock as a abciv1.FinalizeBlockResponse
// This logic is important for the LoadFinalizeBlockResponse method in the state store
// The conversion should work because they are compatible.
func TestStateV1Beta3ResponsesFinalizeBlockAsV1FinalizeBlockResponse(t *testing.T) {
	v1beta3ResponseFinalizeBlock := newV1Beta3ResponsesFinalizeBlock()

	data, err := v1beta3ResponseFinalizeBlock.Marshal()
	require.NoError(t, err)
	require.NotNil(t, data)

	// This works because they are equivalent protos and the fields are populated
	finalizeBlockResponse := new(abciv1.FinalizeBlockResponse)
	err = finalizeBlockResponse.Unmarshal(data)
	require.NoError(t, err)

	// Test for not nil
	require.NotNil(t, finalizeBlockResponse.TxResults)
	require.NotNil(t, finalizeBlockResponse.Events)
	require.NotNil(t, finalizeBlockResponse.ValidatorUpdates)
	require.NotNil(t, finalizeBlockResponse.ConsensusParamUpdates)
	require.NotNil(t, finalizeBlockResponse.AppHash)

	// Test for equality
	require.Equal(t, len(v1beta3ResponseFinalizeBlock.TxResults), len(finalizeBlockResponse.TxResults))
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].Code, finalizeBlockResponse.TxResults[0].Code)
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].Data, finalizeBlockResponse.TxResults[0].Data)
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].Log, finalizeBlockResponse.TxResults[0].Log)
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].GasWanted, finalizeBlockResponse.TxResults[0].GasWanted)
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].GasUsed, finalizeBlockResponse.TxResults[0].GasUsed)
	require.Equal(t, len(v1beta3ResponseFinalizeBlock.TxResults[0].Events), len(finalizeBlockResponse.TxResults[0].Events))
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].Events[0].Type, finalizeBlockResponse.TxResults[0].Events[0].Type)
	require.Equal(t, len(v1beta3ResponseFinalizeBlock.TxResults[0].Events[0].Attributes), len(finalizeBlockResponse.TxResults[0].Events[0].Attributes))
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].Events[0].Attributes[0].Key, finalizeBlockResponse.TxResults[0].Events[0].Attributes[0].Key)
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].Events[0].Attributes[0].Value, finalizeBlockResponse.TxResults[0].Events[0].Attributes[0].Value)
	require.Equal(t, v1beta3ResponseFinalizeBlock.TxResults[0].Codespace, finalizeBlockResponse.TxResults[0].Codespace)

	require.Equal(t, len(v1beta3ResponseFinalizeBlock.Events), len(finalizeBlockResponse.Events))
	require.Equal(t, v1beta3ResponseFinalizeBlock.Events[0].Type, finalizeBlockResponse.Events[0].Type)
	require.Equal(t, len(v1beta3ResponseFinalizeBlock.Events[0].Attributes), len(finalizeBlockResponse.Events[0].Attributes))
	require.Equal(t, v1beta3ResponseFinalizeBlock.Events[0].Attributes[0].Key, finalizeBlockResponse.Events[0].Attributes[0].Key)
	require.Equal(t, v1beta3ResponseFinalizeBlock.Events[0].Attributes[0].Value, finalizeBlockResponse.Events[0].Attributes[0].Value)

	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates, finalizeBlockResponse.ConsensusParamUpdates)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Block.MaxBytes, finalizeBlockResponse.ConsensusParamUpdates.Block.MaxBytes)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Block.MaxGas, finalizeBlockResponse.ConsensusParamUpdates.Block.MaxGas)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Evidence.MaxAgeNumBlocks, finalizeBlockResponse.ConsensusParamUpdates.Evidence.MaxAgeNumBlocks)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Evidence.MaxAgeDuration, finalizeBlockResponse.ConsensusParamUpdates.Evidence.MaxAgeDuration)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Evidence.MaxBytes, finalizeBlockResponse.ConsensusParamUpdates.Evidence.MaxBytes)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Validator.PubKeyTypes, finalizeBlockResponse.ConsensusParamUpdates.Validator.PubKeyTypes)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Version.App, finalizeBlockResponse.ConsensusParamUpdates.Version.App)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Synchrony.Precision, finalizeBlockResponse.ConsensusParamUpdates.Synchrony.Precision)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Synchrony.MessageDelay, finalizeBlockResponse.ConsensusParamUpdates.Synchrony.MessageDelay)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Feature.VoteExtensionsEnableHeight.Value, finalizeBlockResponse.ConsensusParamUpdates.Feature.VoteExtensionsEnableHeight.Value)
	require.Equal(t, v1beta3ResponseFinalizeBlock.ConsensusParamUpdates.Feature.PbtsEnableHeight.Value, finalizeBlockResponse.ConsensusParamUpdates.Feature.PbtsEnableHeight.Value)

	require.Equal(t, v1beta3ResponseFinalizeBlock.AppHash, finalizeBlockResponse.AppHash)

	require.Equal(t, len(v1beta3ResponseFinalizeBlock.ValidatorUpdates), len(finalizeBlockResponse.ValidatorUpdates))
	require.Equal(t, v1beta3ResponseFinalizeBlock.ValidatorUpdates[0].Power, finalizeBlockResponse.ValidatorUpdates[0].Power)

	// skip until an equivalency test is possible
	// require.NotNil(t, finalizeBlockResponse.ValidatorUpdates[0].PubKeyBytes)
	// require.NotEmpty(t, finalizeBlockResponse.ValidatorUpdates[0].PubKeyType)
	// require.Equal(t, v1beta3ResponseFinalizeBlock.ValidatorUpdates[0].PubKey.GetEd25519(), finalizeBlockResponse.ValidatorUpdates[0].PubKeyBytes)
}

// Generate a v1beta2 ABCIResponses with data for all fields.
func newV1Beta2ABCIResponses() statev1beta2.ABCIResponses {
	eventAttr := abciv1beta2.EventAttribute{
		Key:   "key",
		Value: "value",
	}

	deliverTxEvent := abciv1beta2.Event{
		Type:       "deliver_tx_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	responseDeliverTx := abciv1beta2.ResponseDeliverTx{
		Code:   abciv1beta1.CodeTypeOK,
		Data:   []byte("result tx data"),
		Log:    "tx committed successfully",
		Info:   "tx processing info",
		Events: []abciv1beta2.Event{deliverTxEvent},
	}

	validatorUpdates := []abciv1beta1.ValidatorUpdate{{
		PubKey: cryptov1.PublicKey{Sum: &cryptov1.PublicKey_Ed25519{Ed25519: make([]byte, 1)}},
		Power:  int64(10),
	}}

	consensusParams := &typesv1beta2.ConsensusParams{
		Block: &typesv1beta2.BlockParams{
			MaxBytes: int64(100000),
			MaxGas:   int64(10000),
		},
		Evidence: &typesv1beta1.EvidenceParams{
			MaxAgeNumBlocks: int64(10),
			MaxAgeDuration:  time.Duration(1000),
			MaxBytes:        int64(10000),
		},
		Validator: &typesv1beta1.ValidatorParams{
			PubKeyTypes: []string{"ed25519"},
		},
		Version: &typesv1beta1.VersionParams{
			App: uint64(10),
		},
	}

	endBlockEvent := abciv1beta2.Event{
		Type:       "end_block_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	beginBlockEvent := abciv1beta2.Event{
		Type:       "begin_block_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	// v1beta2 ABCI Responses
	v1beta2ABCIResponses := statev1beta2.ABCIResponses{
		BeginBlock: &abciv1beta2.ResponseBeginBlock{
			Events: []abciv1beta2.Event{beginBlockEvent},
		},
		DeliverTxs: []*abciv1beta2.ResponseDeliverTx{
			&responseDeliverTx,
		},
		EndBlock: &abciv1beta2.ResponseEndBlock{
			ValidatorUpdates:      validatorUpdates,
			ConsensusParamUpdates: consensusParams,
			Events:                []abciv1beta2.Event{endBlockEvent},
		},
	}
	return v1beta2ABCIResponses
}

// Generate a v1beta2 ABCIResponses with fields missing data (nil).
func newV1Beta2ABCIResponsesWithNullFields() statev1beta2.ABCIResponses {
	eventAttr := abciv1beta2.EventAttribute{
		Key:   "key",
		Value: "value",
	}

	deliverTxEvent := abciv1beta2.Event{
		Type:       "deliver_tx_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	responseDeliverTx := abciv1beta2.ResponseDeliverTx{
		Code:   abciv1beta1.CodeTypeOK,
		Events: []abciv1beta2.Event{deliverTxEvent},
	}

	beginBlockEvent := abciv1beta2.Event{
		Type:       "begin_block_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	// v1beta2 ABCI Responses
	v1beta2ABCIResponses := statev1beta2.ABCIResponses{
		BeginBlock: &abciv1beta2.ResponseBeginBlock{
			Events: []abciv1beta2.Event{beginBlockEvent},
		},
		DeliverTxs: []*abciv1beta2.ResponseDeliverTx{
			&responseDeliverTx,
		},
	}
	return v1beta2ABCIResponses
}

// Generate a v1beta3 Response Finalize Block with data for all fields.
func newV1Beta3ResponsesFinalizeBlock() abciv1beta3.ResponseFinalizeBlock {
	eventAttr := abciv1beta2.EventAttribute{
		Key:   "key",
		Value: "value",
	}

	txEvent := abciv1beta2.Event{
		Type:       "tx_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	oneEvent := abciv1beta2.Event{
		Type:       "one_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	twoEvent := abciv1beta2.Event{
		Type:       "two_event",
		Attributes: []abciv1beta2.EventAttribute{eventAttr},
	}

	events := make([]abciv1beta2.Event, 0)
	events = append(events, txEvent)
	events = append(events, oneEvent)
	events = append(events, twoEvent)

	txResults := []*abciv1beta3.ExecTxResult{{
		Code:      0,
		Data:      []byte("result tx data"),
		Log:       "tx committed successfully",
		Info:      "tx processing info",
		GasWanted: 15,
		GasUsed:   10,
		Events:    []abciv1beta2.Event{txEvent},
		Codespace: "01",
	}}

	validatorUpdates := []abciv1beta1.ValidatorUpdate{{
		PubKey: cryptov1.PublicKey{Sum: &cryptov1.PublicKey_Ed25519{Ed25519: make([]byte, ed25519.PubKeySize)}},
		Power:  int64(10),
	}}

	consensusParams := &typesv1.ConsensusParams{
		Block: &typesv1.BlockParams{
			MaxBytes: int64(100000),
			MaxGas:   int64(10000),
		},
		Evidence: &typesv1.EvidenceParams{
			MaxAgeNumBlocks: int64(10),
			MaxAgeDuration:  time.Duration(1000),
			MaxBytes:        int64(10000),
		},
		Validator: &typesv1.ValidatorParams{
			PubKeyTypes: []string{ed25519.KeyType},
		},
		Version: &typesv1.VersionParams{
			App: uint64(10),
		},
		Synchrony: &typesv1.SynchronyParams{
			Precision:    durationPtr(time.Second * 2),
			MessageDelay: durationPtr(time.Second * 4),
		},
		Feature: &typesv1.FeatureParams{
			VoteExtensionsEnableHeight: &gogo.Int64Value{
				Value: 10,
			},
			PbtsEnableHeight: &gogo.Int64Value{
				Value: 10,
			},
		},
	}

	// v1beta3 FinalizeBlock Response
	v1beta3FinalizeBlock := abciv1beta3.ResponseFinalizeBlock{
		Events:                events,
		TxResults:             txResults,
		ValidatorUpdates:      validatorUpdates,
		ConsensusParamUpdates: consensusParams,
		AppHash:               make([]byte, 32),
	}
	return v1beta3FinalizeBlock
}

func durationPtr(t time.Duration) *time.Duration {
	return &t
}
