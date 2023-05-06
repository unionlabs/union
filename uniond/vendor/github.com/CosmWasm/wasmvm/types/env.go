package types

//---------- Env ---------

// Env defines the state of the blockchain environment this contract is
// running in. This must contain only trusted data - nothing from the Tx itself
// that has not been verfied (like Signer).
//
// Env are json encoded to a byte slice before passing to the wasm contract.
type Env struct {
	Block       BlockInfo        `json:"block"`
	Transaction *TransactionInfo `json:"transaction"`
	Contract    ContractInfo     `json:"contract"`
}

type BlockInfo struct {
	// block height this transaction is executed
	Height uint64 `json:"height"`
	// time in nanoseconds since unix epoch. Uses string to ensure JavaScript compatibility.
	Time    uint64 `json:"time,string"`
	ChainID string `json:"chain_id"`
}

type ContractInfo struct {
	// Bech32 encoded sdk.AccAddress of the contract, to be used when sending messages
	Address HumanAddress `json:"address"`
}

type TransactionInfo struct {
	// Position of this transaction in the block.
	// The first transaction has index 0
	//
	// Along with BlockInfo.Height, this allows you to get a unique
	// transaction identifier for the chain for future queries
	Index uint32 `json:"index"`
}

type MessageInfo struct {
	// Bech32 encoded sdk.AccAddress executing the contract
	Sender HumanAddress `json:"sender"`
	// Amount of funds send to the contract along with this message
	Funds Coins `json:"funds"`
}
