package enginev1

import (
	"encoding/json"
	"math/big"
	"reflect"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

var errExecutionUnmarshal = errors.New("unable to unmarshal execution engine data")

// PayloadIDBytes defines a custom type for Payload IDs used by the engine API
// client with proper JSON Marshal and Unmarshal methods to hex.
type PayloadIDBytes [8]byte

// MarshalJSON --
func (b PayloadIDBytes) MarshalJSON() ([]byte, error) {
	return json.Marshal(hexutil.Bytes(b[:]))
}

// ExecutionBlock is the response kind received by the eth_getBlockByHash and
// eth_getBlockByNumber endpoints via JSON-RPC.
type ExecutionBlock struct {
	Version int
	gethtypes.Header
	Hash            common.Hash              `json:"hash"`
	Transactions    []*gethtypes.Transaction `json:"transactions"`
	TotalDifficulty string                   `json:"totalDifficulty"`
	Withdrawals     []*Withdrawal            `json:"withdrawals"`
}

func (e *ExecutionBlock) MarshalJSON() ([]byte, error) {
	decoded := make(map[string]interface{})
	encodedHeader, err := e.Header.MarshalJSON()
	if err != nil {
		return nil, err
	}
	if err := json.Unmarshal(encodedHeader, &decoded); err != nil {
		return nil, err
	}
	decoded["hash"] = e.Hash.String()
	decoded["transactions"] = e.Transactions
	decoded["totalDifficulty"] = e.TotalDifficulty

	if e.Version == version.Capella {
		decoded["withdrawals"] = e.Withdrawals
	}

	return json.Marshal(decoded)
}

func (e *ExecutionBlock) UnmarshalJSON(enc []byte) error {
	type transactionsJson struct {
		Transactions []*gethtypes.Transaction `json:"transactions"`
	}
	type withdrawalsJson struct {
		Withdrawals []*withdrawalJSON `json:"withdrawals"`
	}

	if err := e.Header.UnmarshalJSON(enc); err != nil {
		return err
	}
	decoded := make(map[string]interface{})
	if err := json.Unmarshal(enc, &decoded); err != nil {
		return err
	}
	blockHashStr, ok := decoded["hash"].(string)
	if !ok {
		return errors.New("expected `hash` field in JSON response")
	}
	decodedHash, err := hexutil.Decode(blockHashStr)
	if err != nil {
		return err
	}
	e.Hash = common.BytesToHash(decodedHash)
	e.TotalDifficulty, ok = decoded["totalDifficulty"].(string)
	if !ok {
		return errors.New("expected `totalDifficulty` field in JSON response")
	}

	rawWithdrawals, ok := decoded["withdrawals"]
	if !ok || rawWithdrawals == nil {
		e.Version = version.Bellatrix
	} else {
		e.Version = version.Capella
		j := &withdrawalsJson{}
		if err := json.Unmarshal(enc, j); err != nil {
			return err
		}
		ws := make([]*Withdrawal, len(j.Withdrawals))
		for i, wj := range j.Withdrawals {
			ws[i], err = wj.ToWithdrawal()
			if err != nil {
				return err
			}
		}
		e.Withdrawals = ws

		edg, has := decoded["excessBlobGas"]
		if has && edg != nil {
			e.Version = version.Deneb
		}

		dgu, has := decoded["blobGasUsed"]
		if has && dgu != nil {
			e.Version = version.Deneb
		}
	}

	rawTxList, ok := decoded["transactions"]
	if !ok || rawTxList == nil {
		// Exit early if there are no transactions stored in the json payload.
		return nil
	}
	txsList, ok := rawTxList.([]interface{})
	if !ok {
		return errors.Errorf("expected transaction list to be of a slice interface type.")
	}
	for _, tx := range txsList {
		// If the transaction is just a hex string, do not attempt to
		// unmarshal into a full transaction object.
		if txItem, ok := tx.(string); ok && strings.HasPrefix(txItem, "0x") {
			return nil
		}
	}
	// If the block contains a list of transactions, we JSON unmarshal
	// them into a list of geth transaction objects.
	txJson := &transactionsJson{}
	if err := json.Unmarshal(enc, txJson); err != nil {
		return err
	}
	e.Transactions = txJson.Transactions
	return nil
}

// UnmarshalJSON --
func (b *PayloadIDBytes) UnmarshalJSON(enc []byte) error {
	var res [8]byte
	if err := hexutil.UnmarshalFixedJSON(reflect.TypeOf(b), enc, res[:]); err != nil {
		return err
	}
	*b = res
	return nil
}

type withdrawalJSON struct {
	Index     *hexutil.Uint64 `json:"index"`
	Validator *hexutil.Uint64 `json:"validatorIndex"`
	Address   *common.Address `json:"address"`
	Amount    *hexutil.Uint64 `json:"amount"`
}

func (j *withdrawalJSON) ToWithdrawal() (*Withdrawal, error) {
	w := &Withdrawal{}
	b, err := json.Marshal(j)
	if err != nil {
		return nil, err
	}
	if err := w.UnmarshalJSON(b); err != nil {
		return nil, err
	}
	return w, nil
}

func (w *Withdrawal) MarshalJSON() ([]byte, error) {
	index := hexutil.Uint64(w.Index)
	validatorIndex := hexutil.Uint64(w.ValidatorIndex)
	gwei := hexutil.Uint64(w.Amount)
	address := common.BytesToAddress(w.Address)
	return json.Marshal(withdrawalJSON{
		Index:     &index,
		Validator: &validatorIndex,
		Address:   &address,
		Amount:    &gwei,
	})
}

func (w *Withdrawal) UnmarshalJSON(enc []byte) error {
	dec := withdrawalJSON{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}
	if dec.Index == nil {
		return errors.New("missing withdrawal index")
	}
	if dec.Validator == nil {
		return errors.New("missing validator index")
	}
	if dec.Amount == nil {
		return errors.New("missing withdrawal amount")
	}
	if dec.Address == nil {
		return errors.New("missing execution address")
	}
	*w = Withdrawal{}
	w.Index = uint64(*dec.Index)
	w.ValidatorIndex = primitives.ValidatorIndex(*dec.Validator)
	w.Amount = uint64(*dec.Amount)
	w.Address = dec.Address.Bytes()
	return nil
}

type executionPayloadJSON struct {
	ParentHash    *common.Hash    `json:"parentHash"`
	FeeRecipient  *common.Address `json:"feeRecipient"`
	StateRoot     *common.Hash    `json:"stateRoot"`
	ReceiptsRoot  *common.Hash    `json:"receiptsRoot"`
	LogsBloom     *hexutil.Bytes  `json:"logsBloom"`
	PrevRandao    *common.Hash    `json:"prevRandao"`
	BlockNumber   *hexutil.Uint64 `json:"blockNumber"`
	GasLimit      *hexutil.Uint64 `json:"gasLimit"`
	GasUsed       *hexutil.Uint64 `json:"gasUsed"`
	Timestamp     *hexutil.Uint64 `json:"timestamp"`
	ExtraData     hexutil.Bytes   `json:"extraData"`
	BaseFeePerGas string          `json:"baseFeePerGas"`
	BlockHash     *common.Hash    `json:"blockHash"`
	Transactions  []hexutil.Bytes `json:"transactions"`
}

type GetPayloadV2ResponseJson struct {
	ExecutionPayload *ExecutionPayloadCapellaJSON `json:"executionPayload"`
	BlockValue       string                       `json:"blockValue"`
}

type ExecutionPayloadCapellaJSON struct {
	ParentHash    *common.Hash    `json:"parentHash"`
	FeeRecipient  *common.Address `json:"feeRecipient"`
	StateRoot     *common.Hash    `json:"stateRoot"`
	ReceiptsRoot  *common.Hash    `json:"receiptsRoot"`
	LogsBloom     *hexutil.Bytes  `json:"logsBloom"`
	PrevRandao    *common.Hash    `json:"prevRandao"`
	BlockNumber   *hexutil.Uint64 `json:"blockNumber"`
	GasLimit      *hexutil.Uint64 `json:"gasLimit"`
	GasUsed       *hexutil.Uint64 `json:"gasUsed"`
	Timestamp     *hexutil.Uint64 `json:"timestamp"`
	ExtraData     hexutil.Bytes   `json:"extraData"`
	BaseFeePerGas string          `json:"baseFeePerGas"`
	BlockHash     *common.Hash    `json:"blockHash"`
	Transactions  []hexutil.Bytes `json:"transactions"`
	Withdrawals   []*Withdrawal   `json:"withdrawals"`
}

type GetPayloadV3ResponseJson struct {
	ExecutionPayload      *ExecutionPayloadDenebJSON `json:"executionPayload"`
	BlockValue            string                     `json:"blockValue"`
	BlobsBundle           *BlobBundleJSON            `json:"blobsBundle"`
	ShouldOverrideBuilder bool                       `json:"shouldOverrideBuilder"`
}

type ExecutionPayloadDenebJSON struct {
	ParentHash    *common.Hash    `json:"parentHash"`
	FeeRecipient  *common.Address `json:"feeRecipient"`
	StateRoot     *common.Hash    `json:"stateRoot"`
	ReceiptsRoot  *common.Hash    `json:"receiptsRoot"`
	LogsBloom     *hexutil.Bytes  `json:"logsBloom"`
	PrevRandao    *common.Hash    `json:"prevRandao"`
	BlockNumber   *hexutil.Uint64 `json:"blockNumber"`
	GasLimit      *hexutil.Uint64 `json:"gasLimit"`
	GasUsed       *hexutil.Uint64 `json:"gasUsed"`
	Timestamp     *hexutil.Uint64 `json:"timestamp"`
	ExtraData     hexutil.Bytes   `json:"extraData"`
	BaseFeePerGas string          `json:"baseFeePerGas"`
	BlobGasUsed   *hexutil.Uint64 `json:"blobGasUsed"`
	ExcessBlobGas *hexutil.Uint64 `json:"excessBlobGas"`
	BlockHash     *common.Hash    `json:"blockHash"`
	Transactions  []hexutil.Bytes `json:"transactions"`
	Withdrawals   []*Withdrawal   `json:"withdrawals"`
}

// MarshalJSON --
func (e *ExecutionPayload) MarshalJSON() ([]byte, error) {
	transactions := make([]hexutil.Bytes, len(e.Transactions))
	for i, tx := range e.Transactions {
		transactions[i] = tx
	}
	baseFee := bytesutil.LittleEndianBytesToBigInt(e.BaseFeePerGas)
	baseFeeHex := hexutil.EncodeBig(baseFee)
	pHash := common.BytesToHash(e.ParentHash)
	sRoot := common.BytesToHash(e.StateRoot)
	recRoot := common.BytesToHash(e.ReceiptsRoot)
	prevRan := common.BytesToHash(e.PrevRandao)
	bHash := common.BytesToHash(e.BlockHash)
	blockNum := hexutil.Uint64(e.BlockNumber)
	gasLimit := hexutil.Uint64(e.GasLimit)
	gasUsed := hexutil.Uint64(e.GasUsed)
	timeStamp := hexutil.Uint64(e.Timestamp)
	recipient := common.BytesToAddress(e.FeeRecipient)
	logsBloom := hexutil.Bytes(e.LogsBloom)
	return json.Marshal(executionPayloadJSON{
		ParentHash:    &pHash,
		FeeRecipient:  &recipient,
		StateRoot:     &sRoot,
		ReceiptsRoot:  &recRoot,
		LogsBloom:     &logsBloom,
		PrevRandao:    &prevRan,
		BlockNumber:   &blockNum,
		GasLimit:      &gasLimit,
		GasUsed:       &gasUsed,
		Timestamp:     &timeStamp,
		ExtraData:     e.ExtraData,
		BaseFeePerGas: baseFeeHex,
		BlockHash:     &bHash,
		Transactions:  transactions,
	})
}

// MarshalJSON --
func (e *ExecutionPayloadCapella) MarshalJSON() ([]byte, error) {
	transactions := make([]hexutil.Bytes, len(e.Transactions))
	for i, tx := range e.Transactions {
		transactions[i] = tx
	}
	baseFee := new(big.Int).SetBytes(bytesutil.ReverseByteOrder(e.BaseFeePerGas))
	baseFeeHex := hexutil.EncodeBig(baseFee)
	pHash := common.BytesToHash(e.ParentHash)
	sRoot := common.BytesToHash(e.StateRoot)
	recRoot := common.BytesToHash(e.ReceiptsRoot)
	prevRan := common.BytesToHash(e.PrevRandao)
	bHash := common.BytesToHash(e.BlockHash)
	blockNum := hexutil.Uint64(e.BlockNumber)
	gasLimit := hexutil.Uint64(e.GasLimit)
	gasUsed := hexutil.Uint64(e.GasUsed)
	timeStamp := hexutil.Uint64(e.Timestamp)
	recipient := common.BytesToAddress(e.FeeRecipient)
	logsBloom := hexutil.Bytes(e.LogsBloom)
	withdrawals := e.Withdrawals
	if withdrawals == nil {
		withdrawals = make([]*Withdrawal, 0)
	}
	return json.Marshal(ExecutionPayloadCapellaJSON{
		ParentHash:    &pHash,
		FeeRecipient:  &recipient,
		StateRoot:     &sRoot,
		ReceiptsRoot:  &recRoot,
		LogsBloom:     &logsBloom,
		PrevRandao:    &prevRan,
		BlockNumber:   &blockNum,
		GasLimit:      &gasLimit,
		GasUsed:       &gasUsed,
		Timestamp:     &timeStamp,
		ExtraData:     e.ExtraData,
		BaseFeePerGas: baseFeeHex,
		BlockHash:     &bHash,
		Transactions:  transactions,
		Withdrawals:   withdrawals,
	})
}

// UnmarshalJSON --
func (e *ExecutionPayload) UnmarshalJSON(enc []byte) error {
	dec := executionPayloadJSON{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}

	if dec.ParentHash == nil {
		return errors.New("missing required field 'parentHash' for ExecutionPayload")
	}
	if dec.FeeRecipient == nil {
		return errors.New("missing required field 'feeRecipient' for ExecutionPayload")
	}
	if dec.StateRoot == nil {
		return errors.New("missing required field 'stateRoot' for ExecutionPayload")
	}
	if dec.ReceiptsRoot == nil {
		return errors.New("missing required field 'receiptsRoot' for ExecutableDataV1")
	}
	if dec.LogsBloom == nil {
		return errors.New("missing required field 'logsBloom' for ExecutionPayload")
	}
	if dec.PrevRandao == nil {
		return errors.New("missing required field 'prevRandao' for ExecutionPayload")
	}
	if dec.ExtraData == nil {
		return errors.New("missing required field 'extraData' for ExecutionPayload")
	}
	if dec.BlockHash == nil {
		return errors.New("missing required field 'blockHash' for ExecutionPayload")
	}
	if dec.Transactions == nil {
		return errors.New("missing required field 'transactions' for ExecutionPayload")
	}
	if dec.BlockNumber == nil {
		return errors.New("missing required field 'blockNumber' for ExecutionPayload")
	}
	if dec.Timestamp == nil {
		return errors.New("missing required field 'timestamp' for ExecutionPayload")
	}
	if dec.GasUsed == nil {
		return errors.New("missing required field 'gasUsed' for ExecutionPayload")
	}
	if dec.GasLimit == nil {
		return errors.New("missing required field 'gasLimit' for ExecutionPayload")
	}
	*e = ExecutionPayload{}
	e.ParentHash = dec.ParentHash.Bytes()
	e.FeeRecipient = dec.FeeRecipient.Bytes()
	e.StateRoot = dec.StateRoot.Bytes()
	e.ReceiptsRoot = dec.ReceiptsRoot.Bytes()
	e.LogsBloom = *dec.LogsBloom
	e.PrevRandao = dec.PrevRandao.Bytes()
	e.BlockNumber = uint64(*dec.BlockNumber)
	e.GasLimit = uint64(*dec.GasLimit)
	e.GasUsed = uint64(*dec.GasUsed)
	e.Timestamp = uint64(*dec.Timestamp)
	e.ExtraData = dec.ExtraData
	baseFee, err := hexutil.DecodeBig(dec.BaseFeePerGas)
	if err != nil {
		return err
	}
	e.BaseFeePerGas = bytesutil.PadTo(bytesutil.ReverseByteOrder(baseFee.Bytes()), fieldparams.RootLength)
	e.BlockHash = dec.BlockHash.Bytes()
	transactions := make([][]byte, len(dec.Transactions))
	for i, tx := range dec.Transactions {
		transactions[i] = tx
	}
	e.Transactions = transactions
	return nil
}

// UnmarshalJSON --
func (e *ExecutionPayloadCapellaWithValue) UnmarshalJSON(enc []byte) error {
	dec := GetPayloadV2ResponseJson{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}
	if dec.ExecutionPayload == nil {
		return errors.New("missing required field 'executionPayload' for ExecutionPayloadWithValue")
	}

	if dec.ExecutionPayload.ParentHash == nil {
		return errors.New("missing required field 'parentHash' for ExecutionPayload")
	}
	if dec.ExecutionPayload.FeeRecipient == nil {
		return errors.New("missing required field 'feeRecipient' for ExecutionPayload")
	}
	if dec.ExecutionPayload.StateRoot == nil {
		return errors.New("missing required field 'stateRoot' for ExecutionPayload")
	}
	if dec.ExecutionPayload.ReceiptsRoot == nil {
		return errors.New("missing required field 'receiptsRoot' for ExecutableDataV1")
	}
	if dec.ExecutionPayload.LogsBloom == nil {
		return errors.New("missing required field 'logsBloom' for ExecutionPayload")
	}
	if dec.ExecutionPayload.PrevRandao == nil {
		return errors.New("missing required field 'prevRandao' for ExecutionPayload")
	}
	if dec.ExecutionPayload.ExtraData == nil {
		return errors.New("missing required field 'extraData' for ExecutionPayload")
	}
	if dec.ExecutionPayload.BlockHash == nil {
		return errors.New("missing required field 'blockHash' for ExecutionPayload")
	}
	if dec.ExecutionPayload.Transactions == nil {
		return errors.New("missing required field 'transactions' for ExecutionPayload")
	}
	if dec.ExecutionPayload.BlockNumber == nil {
		return errors.New("missing required field 'blockNumber' for ExecutionPayload")
	}
	if dec.ExecutionPayload.Timestamp == nil {
		return errors.New("missing required field 'timestamp' for ExecutionPayload")
	}
	if dec.ExecutionPayload.GasUsed == nil {
		return errors.New("missing required field 'gasUsed' for ExecutionPayload")
	}
	if dec.ExecutionPayload.GasLimit == nil {
		return errors.New("missing required field 'gasLimit' for ExecutionPayload")
	}

	*e = ExecutionPayloadCapellaWithValue{Payload: &ExecutionPayloadCapella{}}
	e.Payload.ParentHash = dec.ExecutionPayload.ParentHash.Bytes()
	e.Payload.FeeRecipient = dec.ExecutionPayload.FeeRecipient.Bytes()
	e.Payload.StateRoot = dec.ExecutionPayload.StateRoot.Bytes()
	e.Payload.ReceiptsRoot = dec.ExecutionPayload.ReceiptsRoot.Bytes()
	e.Payload.LogsBloom = *dec.ExecutionPayload.LogsBloom
	e.Payload.PrevRandao = dec.ExecutionPayload.PrevRandao.Bytes()
	e.Payload.BlockNumber = uint64(*dec.ExecutionPayload.BlockNumber)
	e.Payload.GasLimit = uint64(*dec.ExecutionPayload.GasLimit)
	e.Payload.GasUsed = uint64(*dec.ExecutionPayload.GasUsed)
	e.Payload.Timestamp = uint64(*dec.ExecutionPayload.Timestamp)
	e.Payload.ExtraData = dec.ExecutionPayload.ExtraData
	baseFee, err := hexutil.DecodeBig(dec.ExecutionPayload.BaseFeePerGas)
	if err != nil {
		return err
	}
	e.Payload.BaseFeePerGas = bytesutil.PadTo(bytesutil.ReverseByteOrder(baseFee.Bytes()), fieldparams.RootLength)
	e.Payload.BlockHash = dec.ExecutionPayload.BlockHash.Bytes()
	transactions := make([][]byte, len(dec.ExecutionPayload.Transactions))
	for i, tx := range dec.ExecutionPayload.Transactions {
		transactions[i] = tx
	}
	e.Payload.Transactions = transactions
	if dec.ExecutionPayload.Withdrawals == nil {
		dec.ExecutionPayload.Withdrawals = make([]*Withdrawal, 0)
	}
	e.Payload.Withdrawals = dec.ExecutionPayload.Withdrawals

	v, err := hexutil.DecodeBig(dec.BlockValue)
	if err != nil {
		return err
	}
	e.Value = bytesutil.PadTo(bytesutil.ReverseByteOrder(v.Bytes()), fieldparams.RootLength)

	return nil
}

type payloadAttributesJSON struct {
	Timestamp             hexutil.Uint64 `json:"timestamp"`
	PrevRandao            hexutil.Bytes  `json:"prevRandao"`
	SuggestedFeeRecipient hexutil.Bytes  `json:"suggestedFeeRecipient"`
}

type payloadAttributesV2JSON struct {
	Timestamp             hexutil.Uint64 `json:"timestamp"`
	PrevRandao            hexutil.Bytes  `json:"prevRandao"`
	SuggestedFeeRecipient hexutil.Bytes  `json:"suggestedFeeRecipient"`
	Withdrawals           []*Withdrawal  `json:"withdrawals"`
}

type payloadAttributesV3JSON struct {
	Timestamp             hexutil.Uint64 `json:"timestamp"`
	PrevRandao            hexutil.Bytes  `json:"prevRandao"`
	SuggestedFeeRecipient hexutil.Bytes  `json:"suggestedFeeRecipient"`
	Withdrawals           []*Withdrawal  `json:"withdrawals"`
	ParentBeaconBlockRoot hexutil.Bytes  `json:"parentBeaconBlockRoot"`
}

// MarshalJSON --
func (p *PayloadAttributes) MarshalJSON() ([]byte, error) {
	return json.Marshal(payloadAttributesJSON{
		Timestamp:             hexutil.Uint64(p.Timestamp),
		PrevRandao:            p.PrevRandao,
		SuggestedFeeRecipient: p.SuggestedFeeRecipient,
	})
}

// MarshalJSON --
func (p *PayloadAttributesV2) MarshalJSON() ([]byte, error) {
	withdrawals := p.Withdrawals
	if withdrawals == nil {
		withdrawals = make([]*Withdrawal, 0)
	}

	return json.Marshal(payloadAttributesV2JSON{
		Timestamp:             hexutil.Uint64(p.Timestamp),
		PrevRandao:            p.PrevRandao,
		SuggestedFeeRecipient: p.SuggestedFeeRecipient,
		Withdrawals:           withdrawals,
	})
}

func (p *PayloadAttributesV3) MarshalJSON() ([]byte, error) {
	withdrawals := p.Withdrawals
	if withdrawals == nil {
		withdrawals = make([]*Withdrawal, 0)
	}

	return json.Marshal(payloadAttributesV3JSON{
		Timestamp:             hexutil.Uint64(p.Timestamp),
		PrevRandao:            p.PrevRandao,
		SuggestedFeeRecipient: p.SuggestedFeeRecipient,
		Withdrawals:           withdrawals,
		ParentBeaconBlockRoot: p.ParentBeaconBlockRoot,
	})
}

// UnmarshalJSON --
func (p *PayloadAttributes) UnmarshalJSON(enc []byte) error {
	dec := payloadAttributesJSON{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}
	*p = PayloadAttributes{}
	p.Timestamp = uint64(dec.Timestamp)
	p.PrevRandao = dec.PrevRandao
	p.SuggestedFeeRecipient = dec.SuggestedFeeRecipient
	return nil
}

func (p *PayloadAttributesV2) UnmarshalJSON(enc []byte) error {
	dec := payloadAttributesV2JSON{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}
	*p = PayloadAttributesV2{}
	p.Timestamp = uint64(dec.Timestamp)
	p.PrevRandao = dec.PrevRandao
	p.SuggestedFeeRecipient = dec.SuggestedFeeRecipient
	withdrawals := dec.Withdrawals
	if withdrawals == nil {
		withdrawals = make([]*Withdrawal, 0)
	}
	p.Withdrawals = withdrawals
	return nil
}

func (p *PayloadAttributesV3) UnmarshalJSON(enc []byte) error {
	dec := payloadAttributesV3JSON{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}
	*p = PayloadAttributesV3{}
	p.Timestamp = uint64(dec.Timestamp)
	p.PrevRandao = dec.PrevRandao
	p.SuggestedFeeRecipient = dec.SuggestedFeeRecipient
	withdrawals := dec.Withdrawals
	if withdrawals == nil {
		withdrawals = make([]*Withdrawal, 0)
	}
	p.Withdrawals = withdrawals
	p.ParentBeaconBlockRoot = dec.ParentBeaconBlockRoot
	return nil
}

type payloadStatusJSON struct {
	LatestValidHash *common.Hash `json:"latestValidHash"`
	Status          string       `json:"status"`
	ValidationError *string      `json:"validationError"`
}

// MarshalJSON --
func (p *PayloadStatus) MarshalJSON() ([]byte, error) {
	var latestHash *common.Hash
	if p.LatestValidHash != nil {
		hash := common.Hash(bytesutil.ToBytes32(p.LatestValidHash))
		latestHash = &hash
	}
	return json.Marshal(payloadStatusJSON{
		LatestValidHash: latestHash,
		Status:          p.Status.String(),
		ValidationError: &p.ValidationError,
	})
}

// UnmarshalJSON --
func (p *PayloadStatus) UnmarshalJSON(enc []byte) error {
	dec := payloadStatusJSON{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}
	*p = PayloadStatus{}
	if dec.LatestValidHash != nil {
		p.LatestValidHash = dec.LatestValidHash[:]
	}
	p.Status = PayloadStatus_Status(PayloadStatus_Status_value[dec.Status])
	if dec.ValidationError != nil {
		p.ValidationError = *dec.ValidationError
	}
	return nil
}

type forkchoiceStateJSON struct {
	HeadBlockHash      hexutil.Bytes `json:"headBlockHash"`
	SafeBlockHash      hexutil.Bytes `json:"safeBlockHash"`
	FinalizedBlockHash hexutil.Bytes `json:"finalizedBlockHash"`
}

// MarshalJSON --
func (f *ForkchoiceState) MarshalJSON() ([]byte, error) {
	return json.Marshal(forkchoiceStateJSON{
		HeadBlockHash:      f.HeadBlockHash,
		SafeBlockHash:      f.SafeBlockHash,
		FinalizedBlockHash: f.FinalizedBlockHash,
	})
}

// UnmarshalJSON --
func (f *ForkchoiceState) UnmarshalJSON(enc []byte) error {
	dec := forkchoiceStateJSON{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}
	*f = ForkchoiceState{}
	f.HeadBlockHash = dec.HeadBlockHash
	f.SafeBlockHash = dec.SafeBlockHash
	f.FinalizedBlockHash = dec.FinalizedBlockHash
	return nil
}

type BlobBundleJSON struct {
	Commitments []hexutil.Bytes `json:"commitments"`
	Proofs      []hexutil.Bytes `json:"proofs"`
	Blobs       []hexutil.Bytes `json:"blobs"`
}

func (b BlobBundleJSON) ToProto() *BlobsBundle {
	return &BlobsBundle{
		KzgCommitments: bytesutil.SafeCopy2dHexUtilBytes(b.Commitments),
		Proofs:         bytesutil.SafeCopy2dHexUtilBytes(b.Proofs),
		Blobs:          bytesutil.SafeCopy2dHexUtilBytes(b.Blobs),
	}
}

// MarshalJSON --
func (e *ExecutionPayloadDeneb) MarshalJSON() ([]byte, error) {
	transactions := make([]hexutil.Bytes, len(e.Transactions))
	for i, tx := range e.Transactions {
		transactions[i] = tx
	}
	baseFee := new(big.Int).SetBytes(bytesutil.ReverseByteOrder(e.BaseFeePerGas))
	baseFeeHex := hexutil.EncodeBig(baseFee)
	pHash := common.BytesToHash(e.ParentHash)
	sRoot := common.BytesToHash(e.StateRoot)
	recRoot := common.BytesToHash(e.ReceiptsRoot)
	prevRan := common.BytesToHash(e.PrevRandao)
	bHash := common.BytesToHash(e.BlockHash)
	blockNum := hexutil.Uint64(e.BlockNumber)
	gasLimit := hexutil.Uint64(e.GasLimit)
	gasUsed := hexutil.Uint64(e.GasUsed)
	timeStamp := hexutil.Uint64(e.Timestamp)
	recipient := common.BytesToAddress(e.FeeRecipient)
	logsBloom := hexutil.Bytes(e.LogsBloom)
	withdrawals := e.Withdrawals
	if withdrawals == nil {
		withdrawals = make([]*Withdrawal, 0)
	}
	blobGasUsed := hexutil.Uint64(e.BlobGasUsed)
	excessBlobGas := hexutil.Uint64(e.ExcessBlobGas)

	return json.Marshal(ExecutionPayloadDenebJSON{
		ParentHash:    &pHash,
		FeeRecipient:  &recipient,
		StateRoot:     &sRoot,
		ReceiptsRoot:  &recRoot,
		LogsBloom:     &logsBloom,
		PrevRandao:    &prevRan,
		BlockNumber:   &blockNum,
		GasLimit:      &gasLimit,
		GasUsed:       &gasUsed,
		Timestamp:     &timeStamp,
		ExtraData:     e.ExtraData,
		BaseFeePerGas: baseFeeHex,
		BlockHash:     &bHash,
		Transactions:  transactions,
		Withdrawals:   withdrawals,
		BlobGasUsed:   &blobGasUsed,
		ExcessBlobGas: &excessBlobGas,
	})
}

func (e *ExecutionPayloadDenebWithValueAndBlobsBundle) UnmarshalJSON(enc []byte) error {
	dec := GetPayloadV3ResponseJson{}
	if err := json.Unmarshal(enc, &dec); err != nil {
		return err
	}

	if dec.ExecutionPayload.ParentHash == nil {
		return errors.New("missing required field 'parentHash' for ExecutionPayload")
	}
	if dec.ExecutionPayload.FeeRecipient == nil {
		return errors.New("missing required field 'feeRecipient' for ExecutionPayload")
	}
	if dec.ExecutionPayload.StateRoot == nil {
		return errors.New("missing required field 'stateRoot' for ExecutionPayload")
	}
	if dec.ExecutionPayload.ReceiptsRoot == nil {
		return errors.New("missing required field 'receiptsRoot' for ExecutableDataV1")
	}
	if dec.ExecutionPayload.LogsBloom == nil {
		return errors.New("missing required field 'logsBloom' for ExecutionPayload")
	}
	if dec.ExecutionPayload.PrevRandao == nil {
		return errors.New("missing required field 'prevRandao' for ExecutionPayload")
	}
	if dec.ExecutionPayload.ExtraData == nil {
		return errors.New("missing required field 'extraData' for ExecutionPayload")
	}
	if dec.ExecutionPayload.BlockHash == nil {
		return errors.New("missing required field 'blockHash' for ExecutionPayload")
	}
	if dec.ExecutionPayload.Transactions == nil {
		return errors.New("missing required field 'transactions' for ExecutionPayload")
	}
	if dec.ExecutionPayload.BlockNumber == nil {
		return errors.New("missing required field 'blockNumber' for ExecutionPayload")
	}
	if dec.ExecutionPayload.Timestamp == nil {
		return errors.New("missing required field 'timestamp' for ExecutionPayload")
	}
	if dec.ExecutionPayload.GasUsed == nil {
		return errors.New("missing required field 'gasUsed' for ExecutionPayload")
	}
	if dec.ExecutionPayload.GasLimit == nil {
		return errors.New("missing required field 'gasLimit' for ExecutionPayload")
	}
	if dec.ExecutionPayload.BlobGasUsed == nil {
		return errors.New("missing required field 'blobGasUsed' for ExecutionPayload")
	}
	if dec.ExecutionPayload.ExcessBlobGas == nil {
		return errors.New("missing required field 'excessBlobGas' for ExecutionPayload")
	}

	*e = ExecutionPayloadDenebWithValueAndBlobsBundle{Payload: &ExecutionPayloadDeneb{}}
	e.Payload.ParentHash = dec.ExecutionPayload.ParentHash.Bytes()
	e.Payload.FeeRecipient = dec.ExecutionPayload.FeeRecipient.Bytes()
	e.Payload.StateRoot = dec.ExecutionPayload.StateRoot.Bytes()
	e.Payload.ReceiptsRoot = dec.ExecutionPayload.ReceiptsRoot.Bytes()
	e.Payload.LogsBloom = *dec.ExecutionPayload.LogsBloom
	e.Payload.PrevRandao = dec.ExecutionPayload.PrevRandao.Bytes()
	e.Payload.BlockNumber = uint64(*dec.ExecutionPayload.BlockNumber)
	e.Payload.GasLimit = uint64(*dec.ExecutionPayload.GasLimit)
	e.Payload.GasUsed = uint64(*dec.ExecutionPayload.GasUsed)
	e.Payload.Timestamp = uint64(*dec.ExecutionPayload.Timestamp)
	e.Payload.ExtraData = dec.ExecutionPayload.ExtraData
	baseFee, err := hexutil.DecodeBig(dec.ExecutionPayload.BaseFeePerGas)
	if err != nil {
		return err
	}
	e.Payload.BaseFeePerGas = bytesutil.PadTo(bytesutil.ReverseByteOrder(baseFee.Bytes()), fieldparams.RootLength)

	e.Payload.ExcessBlobGas = uint64(*dec.ExecutionPayload.ExcessBlobGas)
	e.Payload.BlobGasUsed = uint64(*dec.ExecutionPayload.BlobGasUsed)

	e.Payload.BlockHash = dec.ExecutionPayload.BlockHash.Bytes()
	transactions := make([][]byte, len(dec.ExecutionPayload.Transactions))
	for i, tx := range dec.ExecutionPayload.Transactions {
		transactions[i] = tx
	}
	e.Payload.Transactions = transactions
	if dec.ExecutionPayload.Withdrawals == nil {
		dec.ExecutionPayload.Withdrawals = make([]*Withdrawal, 0)
	}
	e.Payload.Withdrawals = dec.ExecutionPayload.Withdrawals

	v, err := hexutil.DecodeBig(dec.BlockValue)
	if err != nil {
		return err
	}
	e.Value = bytesutil.PadTo(bytesutil.ReverseByteOrder(v.Bytes()), fieldparams.RootLength)

	if dec.BlobsBundle == nil {
		return nil
	}
	e.BlobsBundle = &BlobsBundle{}

	commitments := make([][]byte, len(dec.BlobsBundle.Commitments))
	for i, kzg := range dec.BlobsBundle.Commitments {
		k := kzg
		commitments[i] = bytesutil.PadTo(k[:], fieldparams.BLSPubkeyLength)
	}
	e.BlobsBundle.KzgCommitments = commitments

	proofs := make([][]byte, len(dec.BlobsBundle.Proofs))
	for i, proof := range dec.BlobsBundle.Proofs {
		p := proof
		proofs[i] = bytesutil.PadTo(p[:], fieldparams.BLSPubkeyLength)
	}
	e.BlobsBundle.Proofs = proofs

	blobs := make([][]byte, len(dec.BlobsBundle.Blobs))
	for i, blob := range dec.BlobsBundle.Blobs {
		b := make([]byte, fieldparams.BlobLength)
		copy(b, blob)
		blobs[i] = b
	}
	e.BlobsBundle.Blobs = blobs

	e.ShouldOverrideBuilder = dec.ShouldOverrideBuilder

	return nil
}

type executionPayloadBodyV1JSON struct {
	Transactions []hexutil.Bytes `json:"transactions"`
	Withdrawals  []*Withdrawal   `json:"withdrawals"`
}

func (b *ExecutionPayloadBodyV1) MarshalJSON() ([]byte, error) {
	transactions := make([]hexutil.Bytes, len(b.Transactions))
	for i, tx := range b.Transactions {
		transactions[i] = tx
	}
	if len(b.Withdrawals) == 0 {
		b.Withdrawals = make([]*Withdrawal, 0)
	}
	return json.Marshal(executionPayloadBodyV1JSON{
		Transactions: transactions,
		Withdrawals:  b.Withdrawals,
	})
}

func (b *ExecutionPayloadBodyV1) UnmarshalJSON(enc []byte) error {
	var decoded *executionPayloadBodyV1JSON
	err := json.Unmarshal(enc, &decoded)
	if err != nil {
		return err
	}
	if len(decoded.Transactions) == 0 {
		b.Transactions = make([][]byte, 0)
	}
	if len(decoded.Withdrawals) == 0 {
		b.Withdrawals = make([]*Withdrawal, 0)
	}
	transactions := make([][]byte, len(decoded.Transactions))
	for i, tx := range decoded.Transactions {
		transactions[i] = tx
	}
	b.Transactions = transactions
	b.Withdrawals = decoded.Withdrawals
	return nil
}
