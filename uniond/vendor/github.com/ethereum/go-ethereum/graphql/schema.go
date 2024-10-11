// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package graphql

const schema string = `
    # Bytes32 is a 32 byte binary string, represented as 0x-prefixed hexadecimal.
    scalar Bytes32
    # Address is a 20 byte Ethereum address, represented as 0x-prefixed hexadecimal.
    scalar Address
    # Bytes is an arbitrary length binary string, represented as 0x-prefixed hexadecimal.
    # An empty byte string is represented as '0x'. Byte strings must have an even number of hexadecimal nybbles.
    scalar Bytes
    # BigInt is a large integer. Input is accepted as either a JSON number or as a string.
    # Strings may be either decimal or 0x-prefixed hexadecimal. Output values are all
    # 0x-prefixed hexadecimal.
    scalar BigInt
    # Long is a 64 bit unsigned integer. Input is accepted as either a JSON number or as a string.
    # Strings may be either decimal or 0x-prefixed hexadecimal. Output values are all
    # 0x-prefixed hexadecimal.
    scalar Long

    schema {
        query: Query
        mutation: Mutation
    }

    # Account is an Ethereum account at a particular block.
    type Account {
        # Address is the address owning the account.
        address: Address!
        # Balance is the balance of the account, in wei.
        balance: BigInt!
        # TransactionCount is the number of transactions sent from this account,
        # or in the case of a contract, the number of contracts created. Otherwise
        # known as the nonce.
        transactionCount: Long!
        # Code contains the smart contract code for this account, if the account
        # is a (non-self-destructed) contract.
        code: Bytes!
        # Storage provides access to the storage of a contract account, indexed
        # by its 32 byte slot identifier.
        storage(slot: Bytes32!): Bytes32!
    }

    # Log is an Ethereum event log.
    type Log {
        # Index is the index of this log in the block.
        index: Long!
        # Account is the account which generated this log - this will always
        # be a contract account.
        account(block: Long): Account!
        # Topics is a list of 0-4 indexed topics for the log.
        topics: [Bytes32!]!
        # Data is unindexed data for this log.
        data: Bytes!
        # Transaction is the transaction that generated this log entry.
        transaction: Transaction!
    }

    # EIP-2718
    type AccessTuple {
        address: Address!
        storageKeys : [Bytes32!]!
    }

    # EIP-4895
    type Withdrawal {
        # Index is a monotonically increasing identifier issued by consensus layer.
        index: Long!
        # Validator is index of the validator associated with withdrawal.
        validator: Long!
        # Recipient address of the withdrawn amount.
        address: Address!
        # Amount is the withdrawal value in Gwei.
        amount: Long!
    }

    # Transaction is an Ethereum transaction.
    type Transaction {
        # Hash is the hash of this transaction.
        hash: Bytes32!
        # Nonce is the nonce of the account this transaction was generated with.
        nonce: Long!
        # Index is the index of this transaction in the parent block. This will
        # be null if the transaction has not yet been mined.
        index: Long
        # From is the account that sent this transaction - this will always be
        # an externally owned account.
        from(block: Long): Account!
        # To is the account the transaction was sent to. This is null for
        # contract-creating transactions.
        to(block: Long): Account
        # Value is the value, in wei, sent along with this transaction.
        value: BigInt!
        # GasPrice is the price offered to miners for gas, in wei per unit.
        gasPrice: BigInt!
        # MaxFeePerGas is the maximum fee per gas offered to include a transaction, in wei.
        maxFeePerGas: BigInt
        # MaxPriorityFeePerGas is the maximum miner tip per gas offered to include a transaction, in wei.
        maxPriorityFeePerGas: BigInt
        # MaxFeePerBlobGas is the maximum blob gas fee cap per blob the sender is willing to pay for blob transaction, in wei.
        maxFeePerBlobGas: BigInt
        # EffectiveTip is the actual amount of reward going to miner after considering the max fee cap.
        effectiveTip: BigInt
        # Gas is the maximum amount of gas this transaction can consume.
        gas: Long!
        # InputData is the data supplied to the target of the transaction.
        inputData: Bytes!
        # Block is the block this transaction was mined in. This will be null if
        # the transaction has not yet been mined.
        block: Block

        # Status is the return status of the transaction. This will be 1 if the
        # transaction succeeded, or 0 if it failed (due to a revert, or due to
        # running out of gas). If the transaction has not yet been mined, this
        # field will be null.
        status: Long
        # GasUsed is the amount of gas that was used processing this transaction.
        # If the transaction has not yet been mined, this field will be null.
        gasUsed: Long
        # CumulativeGasUsed is the total gas used in the block up to and including
        # this transaction. If the transaction has not yet been mined, this field
        # will be null.
        cumulativeGasUsed: Long
        # EffectiveGasPrice is actual value per gas deducted from the sender's
        # account. Before EIP-1559, this is equal to the transaction's gas price.
        # After EIP-1559, it is baseFeePerGas + min(maxFeePerGas - baseFeePerGas,
        # maxPriorityFeePerGas). Legacy transactions and EIP-2930 transactions are
        # coerced into the EIP-1559 format by setting both maxFeePerGas and
        # maxPriorityFeePerGas as the transaction's gas price.
        effectiveGasPrice: BigInt
        # BlobGasUsed is the amount of blob gas used by this transaction.
        blobGasUsed: Long
        # blobGasPrice is the actual value per blob gas deducted from the senders account.
        blobGasPrice: BigInt
        # CreatedContract is the account that was created by a contract creation
        # transaction. If the transaction was not a contract creation transaction,
        # or it has not yet been mined, this field will be null.
        createdContract(block: Long): Account
        # Logs is a list of log entries emitted by this transaction. If the
        # transaction has not yet been mined, this field will be null.
        logs: [Log!]
        r: BigInt!
        s: BigInt!
        v: BigInt!
        yParity: Long
        # Envelope transaction support
        type: Long
        accessList: [AccessTuple!]
        # Raw is the canonical encoding of the transaction.
        # For legacy transactions, it returns the RLP encoding.
        # For EIP-2718 typed transactions, it returns the type and payload.
        raw: Bytes!
        # RawReceipt is the canonical encoding of the receipt. For post EIP-2718 typed transactions
        # this is equivalent to TxType || ReceiptEncoding.
        rawReceipt: Bytes!
        # BlobVersionedHashes is a set of hash outputs from the blobs in the transaction.
        blobVersionedHashes: [Bytes32!]
    }

    # BlockFilterCriteria encapsulates log filter criteria for a filter applied
    # to a single block.
    input BlockFilterCriteria {
        # Addresses is list of addresses that are of interest. If this list is
        # empty, results will not be filtered by address.
        addresses: [Address!]
        # Topics list restricts matches to particular event topics. Each event has a list
        # of topics. Topics matches a prefix of that list. An empty element array matches any
        # topic. Non-empty elements represent an alternative that matches any of the
        # contained topics.
        #
        # Examples:
        #  - [] or nil          matches any topic list
        #  - [[A]]              matches topic A in first position
        #  - [[], [B]]          matches any topic in first position, B in second position
        #  - [[A], [B]]         matches topic A in first position, B in second position
        #  - [[A, B]], [C, D]]  matches topic (A OR B) in first position, (C OR D) in second position
        topics: [[Bytes32!]!]
    }

    # Block is an Ethereum block.
    type Block {
        # Number is the number of this block, starting at 0 for the genesis block.
        number: Long!
        # Hash is the block hash of this block.
        hash: Bytes32!
        # Parent is the parent block of this block.
        parent: Block
        # Nonce is the block nonce, an 8 byte sequence determined by the miner.
        nonce: Bytes!
        # TransactionsRoot is the keccak256 hash of the root of the trie of transactions in this block.
        transactionsRoot: Bytes32!
        # TransactionCount is the number of transactions in this block. if
        # transactions are not available for this block, this field will be null.
        transactionCount: Long
        # StateRoot is the keccak256 hash of the state trie after this block was processed.
        stateRoot: Bytes32!
        # ReceiptsRoot is the keccak256 hash of the trie of transaction receipts in this block.
        receiptsRoot: Bytes32!
        # Miner is the account that mined this block.
        miner(block: Long): Account!
        # ExtraData is an arbitrary data field supplied by the miner.
        extraData: Bytes!
        # GasLimit is the maximum amount of gas that was available to transactions in this block.
        gasLimit: Long!
        # GasUsed is the amount of gas that was used executing transactions in this block.
        gasUsed: Long!
        # BaseFeePerGas is the fee per unit of gas burned by the protocol in this block.
        baseFeePerGas: BigInt
        # NextBaseFeePerGas is the fee per unit of gas which needs to be burned in the next block.
        nextBaseFeePerGas: BigInt
        # Timestamp is the unix timestamp at which this block was mined.
        timestamp: Long!
        # LogsBloom is a bloom filter that can be used to check if a block may
        # contain log entries matching a filter.
        logsBloom: Bytes!
        # MixHash is the hash that was used as an input to the PoW process.
        mixHash: Bytes32!
        # Difficulty is a measure of the difficulty of mining this block.
        difficulty: BigInt!
        # TotalDifficulty is the sum of all difficulty values up to and including
        # this block.
        totalDifficulty: BigInt!
        # OmmerCount is the number of ommers (AKA uncles) associated with this
        # block. If ommers are unavailable, this field will be null.
        ommerCount: Long
        # Ommers is a list of ommer (AKA uncle) blocks associated with this block.
        # If ommers are unavailable, this field will be null. Depending on your
        # node, the transactions, transactionAt, transactionCount, ommers,
        # ommerCount and ommerAt fields may not be available on any ommer blocks.
        ommers: [Block]
        # OmmerAt returns the ommer (AKA uncle) at the specified index. If ommers
        # are unavailable, or the index is out of bounds, this field will be null.
        ommerAt(index: Long!): Block
        # OmmerHash is the keccak256 hash of all the ommers (AKA uncles)
        # associated with this block.
        ommerHash: Bytes32!
        # Transactions is a list of transactions associated with this block. If
        # transactions are unavailable for this block, this field will be null.
        transactions: [Transaction!]
        # TransactionAt returns the transaction at the specified index. If
        # transactions are unavailable for this block, or if the index is out of
        # bounds, this field will be null.
        transactionAt(index: Long!): Transaction
        # Logs returns a filtered set of logs from this block.
        logs(filter: BlockFilterCriteria!): [Log!]!
        # Account fetches an Ethereum account at the current block's state.
        account(address: Address!): Account!
        # Call executes a local call operation at the current block's state.
        call(data: CallData!): CallResult
        # EstimateGas estimates the amount of gas that will be required for
        # successful execution of a transaction at the current block's state.
        estimateGas(data: CallData!): Long!
        # RawHeader is the RLP encoding of the block's header.
        rawHeader: Bytes!
        # Raw is the RLP encoding of the block.
        raw: Bytes!
        # WithdrawalsRoot is the withdrawals trie root in this block.
        # If withdrawals are unavailable for this block, this field will be null.
        withdrawalsRoot: Bytes32
        # Withdrawals is a list of withdrawals associated with this block. If
        # withdrawals are unavailable for this block, this field will be null.
        withdrawals: [Withdrawal!]
        # BlobGasUsed is the total amount of gas used by the transactions.
        blobGasUsed: Long
        # ExcessBlobGas is a running total of blob gas consumed in excess of the target, prior to the block.
        excessBlobGas: Long
    }

    # CallData represents the data associated with a local contract call.
    # All fields are optional.
    input CallData {
        # From is the address making the call.
        from: Address
        # To is the address the call is sent to.
        to: Address
        # Gas is the amount of gas sent with the call.
        gas: Long
        # GasPrice is the price, in wei, offered for each unit of gas.
        gasPrice: BigInt
        # MaxFeePerGas is the maximum fee per gas offered, in wei.
        maxFeePerGas: BigInt
        # MaxPriorityFeePerGas is the maximum miner tip per gas offered, in wei.
        maxPriorityFeePerGas: BigInt
        # Value is the value, in wei, sent along with the call.
        value: BigInt
        # Data is the data sent to the callee.
        data: Bytes
    }

    # CallResult is the result of a local call operation.
    type CallResult {
        # Data is the return data of the called contract.
        data: Bytes!
        # GasUsed is the amount of gas used by the call, after any refunds.
        gasUsed: Long!
        # Status is the result of the call - 1 for success or 0 for failure.
        status: Long!
    }

    # FilterCriteria encapsulates log filter criteria for searching log entries.
    input FilterCriteria {
        # FromBlock is the block at which to start searching, inclusive. Defaults
        # to the latest block if not supplied.
        fromBlock: Long
        # ToBlock is the block at which to stop searching, inclusive. Defaults
        # to the latest block if not supplied.
        toBlock: Long
        # Addresses is a list of addresses that are of interest. If this list is
        # empty, results will not be filtered by address.
        addresses: [Address!]
        # Topics list restricts matches to particular event topics. Each event has a list
        # of topics. Topics matches a prefix of that list. An empty element array matches any
        # topic. Non-empty elements represent an alternative that matches any of the
        # contained topics.
        #
        # Examples:
        #  - [] or nil          matches any topic list
        #  - [[A]]              matches topic A in first position
        #  - [[], [B]]          matches any topic in first position, B in second position
        #  - [[A], [B]]         matches topic A in first position, B in second position
        #  - [[A, B]], [C, D]]  matches topic (A OR B) in first position, (C OR D) in second position
        topics: [[Bytes32!]!]
    }

    # SyncState contains the current synchronisation state of the client.
    type SyncState {
        # StartingBlock is the block number at which synchronisation started.
        startingBlock: Long!
        # CurrentBlock is the point at which synchronisation has presently reached.
        currentBlock: Long!
        # HighestBlock is the latest known block number.
        highestBlock: Long!
    }

    # Pending represents the current pending state.
    type Pending {
        # TransactionCount is the number of transactions in the pending state.
        transactionCount: Long!
        # Transactions is a list of transactions in the current pending state.
        transactions: [Transaction!]
        # Account fetches an Ethereum account for the pending state.
        account(address: Address!): Account!
        # Call executes a local call operation for the pending state.
        call(data: CallData!): CallResult
        # EstimateGas estimates the amount of gas that will be required for
        # successful execution of a transaction for the pending state.
        estimateGas(data: CallData!): Long!
    }

    type Query {
        # Block fetches an Ethereum block by number or by hash. If neither is
        # supplied, the most recent known block is returned.
        block(number: Long, hash: Bytes32): Block
        # Blocks returns all the blocks between two numbers, inclusive. If
        # to is not supplied, it defaults to the most recent known block.
        blocks(from: Long, to: Long): [Block!]!
        # Pending returns the current pending state.
        pending: Pending!
        # Transaction returns a transaction specified by its hash.
        transaction(hash: Bytes32!): Transaction
        # Logs returns log entries matching the provided filter.
        logs(filter: FilterCriteria!): [Log!]!
        # GasPrice returns the node's estimate of a gas price sufficient to
        # ensure a transaction is mined in a timely fashion.
        gasPrice: BigInt!
        # MaxPriorityFeePerGas returns the node's estimate of a gas tip sufficient
        # to ensure a transaction is mined in a timely fashion.
        maxPriorityFeePerGas: BigInt!
        # Syncing returns information on the current synchronisation state.
        syncing: SyncState
        # ChainID returns the current chain ID for transaction replay protection.
        chainID: BigInt!
    }

    type Mutation {
        # SendRawTransaction sends an RLP-encoded transaction to the network.
        sendRawTransaction(data: Bytes!): Bytes32!
    }
`
