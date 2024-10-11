#!/bin/bash
ticks="\`\`\`"

function showjson(){
  echo "\`$1\`:"
  echo "${ticks}json"
  cat $1
  echo ""
  echo "$ticks"
}
function demo(){
  echo "$ticks"
  echo "$1"
  $1
  echo ""
  echo "$ticks"
  echo ""
}
function tick(){
  echo "$ticks"
}

function code(){
  echo "$ticks$1"
}

cat << "EOF"
# EVM tool

The EVM tool provides a few useful subcommands to facilitate testing at the EVM
layer.

* transition tool    (`t8n`) : a stateless state transition utility
* transaction tool   (`t9n`) : a transaction validation utility
* block builder tool (`b11r`): a block assembler utility

## State transition tool (`t8n`)


The `evm t8n` tool is a stateless state transition utility. It is a utility
which can

1. Take a prestate, including
  - Accounts,
  - Block context information,
  - Previous blockshashes (*optional)
2. Apply a set of transactions,
3. Apply a mining-reward (*optional),
4. And generate a post-state, including
  - State root, transaction root, receipt root,
  - Information about rejected transactions,
  - Optionally: a full or partial post-state dump

### Specification

The idea is to specify the behaviour of this binary very _strict_, so that other
node implementors can build replicas based on their own state-machines, and the
state generators can swap between a \`geth\`-based implementation and a \`parityvm\`-based
implementation.

#### Command line params

Command line params that need to be supported are

```
EOF
./evm t8n -h | grep "\-\-trace\.\|\-\-output\.\|\-\-state\.\|\-\-input"
cat << "EOF"
```
#### Objects

The transition tool uses JSON objects to read and write data related to the transition operation. The
following object definitions are required.

##### `alloc`

The `alloc` object defines the prestate that transition will begin with.

```go
// Map of address to account definition.
type Alloc map[common.Address]Account
// Genesis account. Each field is optional.
type Account struct {
    Code       []byte                           `json:"code"`
    Storage    map[common.Hash]common.Hash      `json:"storage"`
    Balance    *big.Int                         `json:"balance"`
    Nonce      uint64                           `json:"nonce"`
    SecretKey  []byte                            `json:"secretKey"`
}
```

##### `env`

The `env` object defines the environmental context in which the transition will
take place.

```go
type Env struct {
    // required
    CurrentCoinbase  common.Address      `json:"currentCoinbase"`
    CurrentGasLimit  uint64              `json:"currentGasLimit"`
    CurrentNumber    uint64              `json:"currentNumber"`
    CurrentTimestamp uint64              `json:"currentTimestamp"`
    Withdrawals      []*Withdrawal       `json:"withdrawals"`
    // optional
    CurrentDifficulty *big.Int           `json:"currentDifficuly"`
    CurrentRandom     *big.Int           `json:"currentRandom"`
    CurrentBaseFee    *big.Int           `json:"currentBaseFee"`
    ParentDifficulty  *big.Int           `json:"parentDifficulty"`
    ParentGasUsed     uint64             `json:"parentGasUsed"`
    ParentGasLimit    uint64             `json:"parentGasLimit"`
    ParentTimestamp   uint64             `json:"parentTimestamp"`
    BlockHashes       map[uint64]common.Hash `json:"blockHashes"`
    ParentUncleHash   common.Hash        `json:"parentUncleHash"`
    Ommers            []Ommer            `json:"ommers"`
}
type Ommer struct {
    Delta   uint64         `json:"delta"`
    Address common.Address `json:"address"`
}
type Withdrawal struct {
    Index          uint64         `json:"index"`
    ValidatorIndex uint64         `json:"validatorIndex"`
    Recipient      common.Address `json:"recipient"`
    Amount         *big.Int       `json:"amount"`
}
```

##### `txs`

The `txs` object is an array of any of the transaction types: `LegacyTx`,
`AccessListTx`, or `DynamicFeeTx`.

```go
type LegacyTx struct {
	Nonce     uint64          `json:"nonce"`
	GasPrice  *big.Int        `json:"gasPrice"`
	Gas       uint64          `json:"gas"`
	To        *common.Address `json:"to"`
	Value     *big.Int        `json:"value"`
	Data      []byte          `json:"data"`
	V         *big.Int        `json:"v"`
	R         *big.Int        `json:"r"`
	S         *big.Int        `json:"s"`
    SecretKey *common.Hash    `json:"secretKey"`
}
type AccessList []AccessTuple
type AccessTuple struct {
	Address     common.Address `json:"address"        gencodec:"required"`
	StorageKeys []common.Hash  `json:"storageKeys"    gencodec:"required"`
}
type AccessListTx struct {
	ChainID    *big.Int        `json:"chainId"`
	Nonce      uint64          `json:"nonce"`
	GasPrice   *big.Int        `json:"gasPrice"`
	Gas        uint64          `json:"gas"`
	To         *common.Address `json:"to"`
	Value      *big.Int        `json:"value"`
	Data       []byte          `json:"data"`
	AccessList AccessList      `json:"accessList"`
	V          *big.Int        `json:"v"`
	R          *big.Int        `json:"r"`
	S          *big.Int        `json:"s"`
    SecretKey  *common.Hash     `json:"secretKey"`
}
type DynamicFeeTx struct {
	ChainID    *big.Int        `json:"chainId"`
	Nonce      uint64          `json:"nonce"`
	GasTipCap  *big.Int        `json:"maxPriorityFeePerGas"`
	GasFeeCap  *big.Int        `json:"maxFeePerGas"`
	Gas        uint64          `json:"gas"`
	To         *common.Address `json:"to"`
	Value      *big.Int        `json:"value"`
	Data       []byte          `json:"data"`
	AccessList AccessList      `json:"accessList"`
	V          *big.Int        `json:"v"`
	R          *big.Int        `json:"r"`
	S          *big.Int        `json:"s"`
    SecretKey  *common.Hash     `json:"secretKey"`
}
```

##### `result`

The `result` object is output after a transition is executed. It includes
information about the post-transition environment.

```go
type ExecutionResult struct {
    StateRoot   common.Hash    `json:"stateRoot"`
    TxRoot      common.Hash    `json:"txRoot"`
    ReceiptRoot common.Hash    `json:"receiptsRoot"`
    LogsHash    common.Hash    `json:"logsHash"`
    Bloom       types.Bloom    `json:"logsBloom"`
    Receipts    types.Receipts `json:"receipts"`
    Rejected    []*rejectedTx  `json:"rejected,omitempty"`
    Difficulty  *big.Int       `json:"currentDifficulty"`
    GasUsed     uint64         `json:"gasUsed"`
    BaseFee     *big.Int       `json:"currentBaseFee,omitempty"`
}
```

#### Error codes and output

All logging should happen against the `stderr`.
There are a few (not many) errors that can occur, those are defined below.

##### EVM-based errors (`2` to `9`)

- Other EVM error. Exit code `2`
- Failed configuration: when a non-supported or invalid fork was specified. Exit code `3`.
- Block history is not supplied, but needed for a `BLOCKHASH` operation. If `BLOCKHASH`
  is invoked targeting a block which history has not been provided for, the program will
  exit with code `4`.

##### IO errors (`10`-`20`)

- Invalid input json: the supplied data could not be marshalled.
  The program will exit with code `10`
- IO problems: failure to load or save files, the program will exit with code `11`

```
# This should exit with 3
./evm t8n --input.alloc=./testdata/1/alloc.json --input.txs=./testdata/1/txs.json --input.env=./testdata/1/env.json --state.fork=Frontier+1346 2>/dev/null
EOF
./evm t8n --input.alloc=./testdata/1/alloc.json --input.txs=./testdata/1/txs.json --input.env=./testdata/1/env.json --state.fork=Frontier+1346 2>/dev/null
exitcode=$?
if [ $exitcode !=  3 ]; then
	echo "Failed, exitcode should be 3,was $exitcode"
else
  echo "exitcode:$exitcode OK"
fi
cat << "EOF"
```
#### Forks
### Basic usage

The chain configuration to be used for a transition is specified via the
`--state.fork` CLI flag. A list of possible values and configurations can be
found in [`tests/init.go`](tests/init.go).

#### Examples
##### Basic usage

Invoking it with the provided example files
EOF
cmd="./evm t8n --input.alloc=./testdata/1/alloc.json --input.txs=./testdata/1/txs.json --input.env=./testdata/1/env.json --state.fork=Berlin"
tick;echo "$cmd"; tick
$cmd 2>/dev/null
echo "Two resulting files:"
echo ""
showjson alloc.json
showjson result.json
echo ""

echo "We can make them spit out the data to e.g. \`stdout\` like this:"
cmd="./evm t8n --input.alloc=./testdata/1/alloc.json --input.txs=./testdata/1/txs.json --input.env=./testdata/1/env.json --output.result=stdout --output.alloc=stdout --state.fork=Berlin"
tick;echo "$cmd"; tick
output=`$cmd 2>/dev/null`
echo "Output:"
echo "${ticks}json"
echo "$output"
echo "$ticks"

cat << "EOF"

#### About Ommers

Mining rewards and ommer rewards might need to be added. This is how those are applied:

- `block_reward` is the block mining reward for the miner (`0xaa`), of a block at height `N`.
- For each ommer (mined by `0xbb`), with blocknumber `N-delta`
   - (where `delta` is the difference between the current block and the ommer)
   - The account `0xbb` (ommer miner) is awarded `(8-delta)/ 8 * block_reward`
   - The account `0xaa` (block miner) is awarded `block_reward / 32`

To make `t8n` apply these, the following inputs are required:

- `--state.reward`
  - For ethash, it is `5000000000000000000` `wei`,
  - If this is not defined, mining rewards are not applied,
  - A value of `0` is valid, and causes accounts to be 'touched'.
- For each ommer, the tool needs to be given an `address\` and a `delta`. This
  is done via the `ommers` field in `env`.

Note: the tool does not verify that e.g. the normal uncle rules apply,
and allows e.g two uncles at the same height, or the uncle-distance. This means that
the tool allows for negative uncle reward (distance > 8)

Example:
EOF

showjson ./testdata/5/env.json

echo "When applying this, using a reward of \`0x08\`"
cmd="./evm t8n --input.alloc=./testdata/5/alloc.json -input.txs=./testdata/5/txs.json --input.env=./testdata/5/env.json  --output.alloc=stdout --state.reward=0x80 --state.fork=Berlin"
output=`$cmd 2>/dev/null`
echo "Output:"
echo "${ticks}json"
echo "$output"
echo "$ticks"

echo "#### Future EIPS"
echo ""
echo "It is also possible to experiment with future eips that are not yet defined in a hard fork."
echo "Example, putting EIP-1344 into Frontier: "
cmd="./evm t8n --state.fork=Frontier+1344 --input.pre=./testdata/1/pre.json --input.txs=./testdata/1/txs.json --input.env=/testdata/1/env.json"
tick;echo "$cmd"; tick
echo ""

echo "#### Block history"
echo ""
echo "The \`BLOCKHASH\` opcode requires blockhashes to be provided by the caller, inside the \`env\`."
echo "If a required blockhash is not provided, the exit code should be \`4\`:"
echo "Example where blockhashes are provided: "
demo "./evm t8n --input.alloc=./testdata/3/alloc.json --input.txs=./testdata/3/txs.json --input.env=./testdata/3/env.json  --trace --state.fork=Berlin"
cmd="cat trace-0-0x72fadbef39cd251a437eea619cfeda752271a5faaaa2147df012e112159ffb81.jsonl | grep BLOCKHASH -C2"
tick && echo $cmd && tick
echo "$ticks"
cat trace-0-0x72fadbef39cd251a437eea619cfeda752271a5faaaa2147df012e112159ffb81.jsonl | grep BLOCKHASH -C2
echo "$ticks"
echo ""

echo "In this example, the caller has not provided the required blockhash:"
cmd="./evm t8n --input.alloc=./testdata/4/alloc.json --input.txs=./testdata/4/txs.json --input.env=./testdata/4/env.json  --trace --state.fork=Berlin"
tick && echo $cmd && $cmd 2>&1
errc=$?
tick
echo "Error code: $errc"
echo ""

echo "#### Chaining"
echo ""
echo "Another thing that can be done, is to chain invocations:"
cmd1="./evm t8n --input.alloc=./testdata/1/alloc.json --input.txs=./testdata/1/txs.json --input.env=./testdata/1/env.json --state.fork=Berlin --output.alloc=stdout"
cmd2="./evm t8n --input.alloc=stdin --input.env=./testdata/1/env.json --input.txs=./testdata/1/txs.json --state.fork=Berlin"
echo "$ticks"
echo "$cmd1 | $cmd2"
output=$($cmd1 | $cmd2 )
echo $output
echo "$ticks"
echo "What happened here, is that we first applied two identical transactions, so the second one was rejected. "
echo "Then, taking the poststate alloc as the input for the next state, we tried again to include"
echo "the same two transactions: this time, both failed due to too low nonce."
echo ""
echo "In order to meaningfully chain invocations, one would need to provide meaningful new \`env\`, otherwise the"
echo "actual blocknumber (exposed to the EVM) would not increase."
echo ""

echo "#### Transactions in RLP form"
echo ""
echo "It is possible to provide already-signed transactions as input to, using an \`input.txs\` which ends with the \`rlp\` suffix."
echo "The input format for RLP-form transactions is _identical_ to the _output_ format for block bodies. Therefore, it's fully possible"
echo "to use the evm to go from \`json\` input to \`rlp\` input."
echo ""
echo "The following command takes **json** the transactions in \`./testdata/13/txs.json\` and signs them. After execution, they are output to \`signed_txs.rlp\`.:"
cmd="./evm t8n --state.fork=London --input.alloc=./testdata/13/alloc.json --input.txs=./testdata/13/txs.json --input.env=./testdata/13/env.json --output.result=alloc_jsontx.json --output.body=signed_txs.rlp"
echo "$ticks"
echo $cmd
$cmd 2>&1
echo "$ticks"
echo ""
echo "The \`output.body\` is the rlp-list of transactions, encoded in hex and placed in a string a'la \`json\` encoding rules:"
demo "cat signed_txs.rlp"
echo "We can use \`rlpdump\` to check what the contents are: "
echo "$ticks"
echo "rlpdump -hex \$(cat signed_txs.rlp | jq -r )"
rlpdump -hex $(cat signed_txs.rlp | jq -r )
echo "$ticks"
echo "Now, we can now use those (or any other already signed transactions), as input, like so: "
cmd="./evm t8n --state.fork=London --input.alloc=./testdata/13/alloc.json --input.txs=./signed_txs.rlp --input.env=./testdata/13/env.json --output.result=alloc_rlptx.json"
echo "$ticks"
echo $cmd
$cmd 2>&1
echo "$ticks"
echo "You might have noticed that the results from these two invocations were stored in two separate files. "
echo "And we can now finally check that they match."
echo "$ticks"
echo "cat alloc_jsontx.json | jq .stateRoot && cat alloc_rlptx.json | jq .stateRoot"
cat alloc_jsontx.json | jq .stateRoot && cat alloc_rlptx.json | jq .stateRoot
echo "$ticks"

cat << "EOF"

## Transaction tool

The transaction tool is used to perform static validity checks on transactions such as:
* intrinsic gas calculation
* max values on integers
* fee semantics, such as `maxFeePerGas < maxPriorityFeePerGas`
* newer tx types on old forks

### Examples

EOF

cmd="./evm t9n --state.fork Homestead --input.txs testdata/15/signed_txs.rlp"
tick;echo "$cmd";
$cmd 2>/dev/null
tick

cmd="./evm t9n --state.fork London --input.txs testdata/15/signed_txs.rlp"
tick;echo "$cmd";
$cmd 2>/dev/null
tick

cat << "EOF"
## Block builder tool (b11r)

The `evm b11r` tool is used to assemble and seal full block rlps.

### Specification

#### Command line params

Command line params that need to be supported are:

```
    --input.header value        `stdin` or file name of where to find the block header to use. (default: "header.json")
    --input.ommers value        `stdin` or file name of where to find the list of ommer header RLPs to use.
    --input.txs value           `stdin` or file name of where to find the transactions list in RLP form. (default: "txs.rlp")
    --output.basedir value      Specifies where output files are placed. Will be created if it does not exist.
    --output.block value        Determines where to put the alloc of the post-state. (default: "block.json")
                                <file> - into the file <file>
                                `stdout` - into the stdout output
                                `stderr` - into the stderr output
    --seal.clique value         Seal block with Clique. `stdin` or file name of where to find the Clique sealing data.
    --seal.ethash               Seal block with ethash. (default: false)
    --seal.ethash.dir value     Path to ethash DAG. If none exists, a new DAG will be generated.
    --seal.ethash.mode value    Defines the type and amount of PoW verification an ethash engine makes. (default: "normal")
    --verbosity value           Sets the verbosity level. (default: 3)
```

#### Objects

##### `header`

The `header` object is a consensus header.

```go=
type Header struct {
        ParentHash  common.Hash       `json:"parentHash"`
        OmmerHash   *common.Hash      `json:"sha3Uncles"`
        Coinbase    *common.Address   `json:"miner"`
        Root        common.Hash       `json:"stateRoot"         gencodec:"required"`
        TxHash      *common.Hash      `json:"transactionsRoot"`
        ReceiptHash *common.Hash      `json:"receiptsRoot"`
        Bloom       types.Bloom       `json:"logsBloom"`
        Difficulty  *big.Int          `json:"difficulty"`
        Number      *big.Int          `json:"number"            gencodec:"required"`
        GasLimit    uint64            `json:"gasLimit"          gencodec:"required"`
        GasUsed     uint64            `json:"gasUsed"`
        Time        uint64            `json:"timestamp"         gencodec:"required"`
        Extra       []byte            `json:"extraData"`
        MixDigest   common.Hash       `json:"mixHash"`
        Nonce       *types.BlockNonce `json:"nonce"`
        BaseFee     *big.Int          `json:"baseFeePerGas"`
}
```
#### `ommers`

The `ommers` object is a list of RLP-encoded ommer blocks in hex
representation.

```go=
type Ommers []string
```

#### `txs`

The `txs` object is a list of RLP-encoded transactions in hex representation.

```go=
type Txs []string
```

#### `clique`

The `clique` object provides the necessary information to complete a clique
seal of the block.

```go=
var CliqueInfo struct {
        Key       *common.Hash    `json:"secretKey"`
        Voted     *common.Address `json:"voted"`
        Authorize *bool           `json:"authorize"`
        Vanity    common.Hash     `json:"vanity"`
}
```

#### `output`

The `output` object contains two values, the block RLP and the block hash.

```go=
type BlockInfo struct {
    Rlp  []byte      `json:"rlp"`
    Hash common.Hash `json:"hash"`
}
```

## A Note on Encoding

The encoding of values for `evm` utility attempts to be relatively flexible. It
generally supports hex-encoded or decimal-encoded numeric values, and
hex-encoded byte values (like `common.Address`, `common.Hash`, etc). When in
doubt, the [`execution-apis`](https://github.com/ethereum/execution-apis) way
of encoding should always be accepted.

## Testing

There are many test cases in the [`cmd/evm/testdata`](./testdata) directory.
These fixtures are used to power the `t8n` tests in
[`t8n_test.go`](./t8n_test.go). The best way to verify correctness of new `evm`
implementations is to execute these and verify the output and error codes match
the expected values.

EOF
