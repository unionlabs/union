package shared

type SignedBeaconBlock struct {
	Message   *BeaconBlock `json:"message"`
	Signature string       `json:"signature"`
}

type BeaconBlock struct {
	Slot          string           `json:"slot"`
	ProposerIndex string           `json:"proposer_index"`
	ParentRoot    string           `json:"parent_root"`
	StateRoot     string           `json:"state_root"`
	Body          *BeaconBlockBody `json:"body"`
}

type BeaconBlockBody struct {
	RandaoReveal      string                 `json:"randao_reveal"`
	Eth1Data          *Eth1Data              `json:"eth1_data"`
	Graffiti          string                 `json:"graffiti"`
	ProposerSlashings []*ProposerSlashing    `json:"proposer_slashings"`
	AttesterSlashings []*AttesterSlashing    `json:"attester_slashings"`
	Attestations      []*Attestation         `json:"attestations"`
	Deposits          []*Deposit             `json:"deposits"`
	VoluntaryExits    []*SignedVoluntaryExit `json:"voluntary_exits"`
}

type SignedBeaconBlockAltair struct {
	Message   *BeaconBlockAltair `json:"message"`
	Signature string             `json:"signature"`
}

type BeaconBlockAltair struct {
	Slot          string                 `json:"slot"`
	ProposerIndex string                 `json:"proposer_index"`
	ParentRoot    string                 `json:"parent_root"`
	StateRoot     string                 `json:"state_root"`
	Body          *BeaconBlockBodyAltair `json:"body"`
}

type BeaconBlockBodyAltair struct {
	RandaoReveal      string                 `json:"randao_reveal"`
	Eth1Data          *Eth1Data              `json:"eth1_data"`
	Graffiti          string                 `json:"graffiti"`
	ProposerSlashings []*ProposerSlashing    `json:"proposer_slashings"`
	AttesterSlashings []*AttesterSlashing    `json:"attester_slashings"`
	Attestations      []*Attestation         `json:"attestations"`
	Deposits          []*Deposit             `json:"deposits"`
	VoluntaryExits    []*SignedVoluntaryExit `json:"voluntary_exits"`
	SyncAggregate     *SyncAggregate         `json:"sync_aggregate"`
}

type SignedBeaconBlockBellatrix struct {
	Message   *BeaconBlockBellatrix `json:"message"`
	Signature string                `json:"signature"`
}

type BeaconBlockBellatrix struct {
	Slot          string                    `json:"slot"`
	ProposerIndex string                    `json:"proposer_index"`
	ParentRoot    string                    `json:"parent_root"`
	StateRoot     string                    `json:"state_root"`
	Body          *BeaconBlockBodyBellatrix `json:"body"`
}

type BeaconBlockBodyBellatrix struct {
	RandaoReveal      string                 `json:"randao_reveal"`
	Eth1Data          *Eth1Data              `json:"eth1_data"`
	Graffiti          string                 `json:"graffiti"`
	ProposerSlashings []*ProposerSlashing    `json:"proposer_slashings"`
	AttesterSlashings []*AttesterSlashing    `json:"attester_slashings"`
	Attestations      []*Attestation         `json:"attestations"`
	Deposits          []*Deposit             `json:"deposits"`
	VoluntaryExits    []*SignedVoluntaryExit `json:"voluntary_exits"`
	SyncAggregate     *SyncAggregate         `json:"sync_aggregate"`
	ExecutionPayload  *ExecutionPayload      `json:"execution_payload"`
}

type SignedBlindedBeaconBlockBellatrix struct {
	Message   *BlindedBeaconBlockBellatrix `json:"message"`
	Signature string                       `json:"signature"`
}

type BlindedBeaconBlockBellatrix struct {
	Slot          string                           `json:"slot"`
	ProposerIndex string                           `json:"proposer_index"`
	ParentRoot    string                           `json:"parent_root"`
	StateRoot     string                           `json:"state_root"`
	Body          *BlindedBeaconBlockBodyBellatrix `json:"body"`
}

type BlindedBeaconBlockBodyBellatrix struct {
	RandaoReveal           string                  `json:"randao_reveal"`
	Eth1Data               *Eth1Data               `json:"eth1_data"`
	Graffiti               string                  `json:"graffiti"`
	ProposerSlashings      []*ProposerSlashing     `json:"proposer_slashings"`
	AttesterSlashings      []*AttesterSlashing     `json:"attester_slashings"`
	Attestations           []*Attestation          `json:"attestations"`
	Deposits               []*Deposit              `json:"deposits"`
	VoluntaryExits         []*SignedVoluntaryExit  `json:"voluntary_exits"`
	SyncAggregate          *SyncAggregate          `json:"sync_aggregate"`
	ExecutionPayloadHeader *ExecutionPayloadHeader `json:"execution_payload_header"`
}

type SignedBeaconBlockCapella struct {
	Message   *BeaconBlockCapella `json:"message"`
	Signature string              `json:"signature"`
}

type BeaconBlockCapella struct {
	Slot          string                  `json:"slot"`
	ProposerIndex string                  `json:"proposer_index"`
	ParentRoot    string                  `json:"parent_root"`
	StateRoot     string                  `json:"state_root"`
	Body          *BeaconBlockBodyCapella `json:"body"`
}

type BeaconBlockBodyCapella struct {
	RandaoReveal          string                        `json:"randao_reveal"`
	Eth1Data              *Eth1Data                     `json:"eth1_data"`
	Graffiti              string                        `json:"graffiti"`
	ProposerSlashings     []*ProposerSlashing           `json:"proposer_slashings"`
	AttesterSlashings     []*AttesterSlashing           `json:"attester_slashings"`
	Attestations          []*Attestation                `json:"attestations"`
	Deposits              []*Deposit                    `json:"deposits"`
	VoluntaryExits        []*SignedVoluntaryExit        `json:"voluntary_exits"`
	SyncAggregate         *SyncAggregate                `json:"sync_aggregate"`
	ExecutionPayload      *ExecutionPayloadCapella      `json:"execution_payload"`
	BLSToExecutionChanges []*SignedBLSToExecutionChange `json:"bls_to_execution_changes"`
}

type SignedBlindedBeaconBlockCapella struct {
	Message   *BlindedBeaconBlockCapella `json:"message"`
	Signature string                     `json:"signature"`
}

type BlindedBeaconBlockCapella struct {
	Slot          string                         `json:"slot"`
	ProposerIndex string                         `json:"proposer_index"`
	ParentRoot    string                         `json:"parent_root"`
	StateRoot     string                         `json:"state_root"`
	Body          *BlindedBeaconBlockBodyCapella `json:"body"`
}

type BlindedBeaconBlockBodyCapella struct {
	RandaoReveal           string                         `json:"randao_reveal"`
	Eth1Data               *Eth1Data                      `json:"eth1_data"`
	Graffiti               string                         `json:"graffiti"`
	ProposerSlashings      []*ProposerSlashing            `json:"proposer_slashings"`
	AttesterSlashings      []*AttesterSlashing            `json:"attester_slashings"`
	Attestations           []*Attestation                 `json:"attestations"`
	Deposits               []*Deposit                     `json:"deposits"`
	VoluntaryExits         []*SignedVoluntaryExit         `json:"voluntary_exits"`
	SyncAggregate          *SyncAggregate                 `json:"sync_aggregate"`
	ExecutionPayloadHeader *ExecutionPayloadHeaderCapella `json:"execution_payload_header"`
	BLSToExecutionChanges  []*SignedBLSToExecutionChange  `json:"bls_to_execution_changes"`
}

type SignedBeaconBlockContentsDeneb struct {
	SignedBlock *SignedBeaconBlockDeneb `json:"signed_block"`
	KzgProofs   []string                `json:"kzg_proofs"`
	Blobs       []string                `json:"blobs"`
}

type BeaconBlockContentsDeneb struct {
	Block     *BeaconBlockDeneb `json:"block"`
	KzgProofs []string          `json:"kzg_proofs"`
	Blobs     []string          `json:"blobs"`
}

type SignedBeaconBlockDeneb struct {
	Message   *BeaconBlockDeneb `json:"message"`
	Signature string            `json:"signature"`
}

type BeaconBlockDeneb struct {
	Slot          string                `json:"slot"`
	ProposerIndex string                `json:"proposer_index"`
	ParentRoot    string                `json:"parent_root"`
	StateRoot     string                `json:"state_root"`
	Body          *BeaconBlockBodyDeneb `json:"body"`
}

type BeaconBlockBodyDeneb struct {
	RandaoReveal          string                        `json:"randao_reveal"`
	Eth1Data              *Eth1Data                     `json:"eth1_data"`
	Graffiti              string                        `json:"graffiti"`
	ProposerSlashings     []*ProposerSlashing           `json:"proposer_slashings"`
	AttesterSlashings     []*AttesterSlashing           `json:"attester_slashings"`
	Attestations          []*Attestation                `json:"attestations"`
	Deposits              []*Deposit                    `json:"deposits"`
	VoluntaryExits        []*SignedVoluntaryExit        `json:"voluntary_exits"`
	SyncAggregate         *SyncAggregate                `json:"sync_aggregate"`
	ExecutionPayload      *ExecutionPayloadDeneb        `json:"execution_payload"`
	BLSToExecutionChanges []*SignedBLSToExecutionChange `json:"bls_to_execution_changes"`
	BlobKzgCommitments    []string                      `json:"blob_kzg_commitments"`
}

type BlindedBeaconBlockDeneb struct {
	Slot          string                       `json:"slot"`
	ProposerIndex string                       `json:"proposer_index"`
	ParentRoot    string                       `json:"parent_root"`
	StateRoot     string                       `json:"state_root"`
	Body          *BlindedBeaconBlockBodyDeneb `json:"body"`
}

type SignedBlindedBeaconBlockDeneb struct {
	Message   *BlindedBeaconBlockDeneb `json:"message"`
	Signature string                   `json:"signature"`
}

type BlindedBeaconBlockBodyDeneb struct {
	RandaoReveal           string                        `json:"randao_reveal"`
	Eth1Data               *Eth1Data                     `json:"eth1_data"`
	Graffiti               string                        `json:"graffiti"`
	ProposerSlashings      []*ProposerSlashing           `json:"proposer_slashings"`
	AttesterSlashings      []*AttesterSlashing           `json:"attester_slashings"`
	Attestations           []*Attestation                `json:"attestations"`
	Deposits               []*Deposit                    `json:"deposits"`
	VoluntaryExits         []*SignedVoluntaryExit        `json:"voluntary_exits"`
	SyncAggregate          *SyncAggregate                `json:"sync_aggregate"`
	ExecutionPayloadHeader *ExecutionPayloadHeaderDeneb  `json:"execution_payload_header"`
	BLSToExecutionChanges  []*SignedBLSToExecutionChange `json:"bls_to_execution_changes"`
	BlobKzgCommitments     []string                      `json:"blob_kzg_commitments"`
}

type SignedBeaconBlockHeaderContainer struct {
	Header    *SignedBeaconBlockHeader `json:"header"`
	Root      string                   `json:"root"`
	Canonical bool                     `json:"canonical"`
}

type SignedBeaconBlockHeader struct {
	Message   *BeaconBlockHeader `json:"message"`
	Signature string             `json:"signature"`
}

type BeaconBlockHeader struct {
	Slot          string `json:"slot"`
	ProposerIndex string `json:"proposer_index"`
	ParentRoot    string `json:"parent_root"`
	StateRoot     string `json:"state_root"`
	BodyRoot      string `json:"body_root"`
}

type ExecutionPayload struct {
	ParentHash    string   `json:"parent_hash"`
	FeeRecipient  string   `json:"fee_recipient"`
	StateRoot     string   `json:"state_root"`
	ReceiptsRoot  string   `json:"receipts_root"`
	LogsBloom     string   `json:"logs_bloom"`
	PrevRandao    string   `json:"prev_randao"`
	BlockNumber   string   `json:"block_number"`
	GasLimit      string   `json:"gas_limit"`
	GasUsed       string   `json:"gas_used"`
	Timestamp     string   `json:"timestamp"`
	ExtraData     string   `json:"extra_data"`
	BaseFeePerGas string   `json:"base_fee_per_gas"`
	BlockHash     string   `json:"block_hash"`
	Transactions  []string `json:"transactions"`
}

type ExecutionPayloadHeader struct {
	ParentHash       string `json:"parent_hash"`
	FeeRecipient     string `json:"fee_recipient"`
	StateRoot        string `json:"state_root"`
	ReceiptsRoot     string `json:"receipts_root"`
	LogsBloom        string `json:"logs_bloom"`
	PrevRandao       string `json:"prev_randao"`
	BlockNumber      string `json:"block_number"`
	GasLimit         string `json:"gas_limit"`
	GasUsed          string `json:"gas_used"`
	Timestamp        string `json:"timestamp"`
	ExtraData        string `json:"extra_data"`
	BaseFeePerGas    string `json:"base_fee_per_gas"`
	BlockHash        string `json:"block_hash"`
	TransactionsRoot string `json:"transactions_root"`
}

type ExecutionPayloadCapella struct {
	ParentHash    string        `json:"parent_hash"`
	FeeRecipient  string        `json:"fee_recipient"`
	StateRoot     string        `json:"state_root"`
	ReceiptsRoot  string        `json:"receipts_root"`
	LogsBloom     string        `json:"logs_bloom"`
	PrevRandao    string        `json:"prev_randao"`
	BlockNumber   string        `json:"block_number"`
	GasLimit      string        `json:"gas_limit"`
	GasUsed       string        `json:"gas_used"`
	Timestamp     string        `json:"timestamp"`
	ExtraData     string        `json:"extra_data"`
	BaseFeePerGas string        `json:"base_fee_per_gas"`
	BlockHash     string        `json:"block_hash"`
	Transactions  []string      `json:"transactions"`
	Withdrawals   []*Withdrawal `json:"withdrawals"`
}

type ExecutionPayloadHeaderCapella struct {
	ParentHash       string `json:"parent_hash"`
	FeeRecipient     string `json:"fee_recipient"`
	StateRoot        string `json:"state_root"`
	ReceiptsRoot     string `json:"receipts_root"`
	LogsBloom        string `json:"logs_bloom"`
	PrevRandao       string `json:"prev_randao"`
	BlockNumber      string `json:"block_number"`
	GasLimit         string `json:"gas_limit"`
	GasUsed          string `json:"gas_used"`
	Timestamp        string `json:"timestamp"`
	ExtraData        string `json:"extra_data"`
	BaseFeePerGas    string `json:"base_fee_per_gas"`
	BlockHash        string `json:"block_hash"`
	TransactionsRoot string `json:"transactions_root"`
	WithdrawalsRoot  string `json:"withdrawals_root"`
}

type ExecutionPayloadDeneb struct {
	ParentHash    string        `json:"parent_hash"`
	FeeRecipient  string        `json:"fee_recipient"`
	StateRoot     string        `json:"state_root"`
	ReceiptsRoot  string        `json:"receipts_root"`
	LogsBloom     string        `json:"logs_bloom"`
	PrevRandao    string        `json:"prev_randao"`
	BlockNumber   string        `json:"block_number"`
	GasLimit      string        `json:"gas_limit"`
	GasUsed       string        `json:"gas_used"`
	Timestamp     string        `json:"timestamp"`
	ExtraData     string        `json:"extra_data"`
	BaseFeePerGas string        `json:"base_fee_per_gas"`
	BlockHash     string        `json:"block_hash"`
	Transactions  []string      `json:"transactions"`
	Withdrawals   []*Withdrawal `json:"withdrawals"`
	BlobGasUsed   string        `json:"blob_gas_used"`
	ExcessBlobGas string        `json:"excess_blob_gas"`
}

type ExecutionPayloadHeaderDeneb struct {
	ParentHash       string `json:"parent_hash"`
	FeeRecipient     string `json:"fee_recipient"`
	StateRoot        string `json:"state_root"`
	ReceiptsRoot     string `json:"receipts_root"`
	LogsBloom        string `json:"logs_bloom"`
	PrevRandao       string `json:"prev_randao"`
	BlockNumber      string `json:"block_number"`
	GasLimit         string `json:"gas_limit"`
	GasUsed          string `json:"gas_used"`
	Timestamp        string `json:"timestamp"`
	ExtraData        string `json:"extra_data"`
	BaseFeePerGas    string `json:"base_fee_per_gas"`
	BlockHash        string `json:"block_hash"`
	TransactionsRoot string `json:"transactions_root"`
	WithdrawalsRoot  string `json:"withdrawals_root"`
	BlobGasUsed      string `json:"blob_gas_used"`
	ExcessBlobGas    string `json:"excess_blob_gas"`
}
