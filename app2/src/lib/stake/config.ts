import { Ucs05 } from "@unionlabs/sdk"
import { ChannelId, UniversalChainId } from "@unionlabs/sdk/schema"

// Chain configuration
export const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.11155111")
export const UNION_CHAIN_ID = UniversalChainId.make("union.union-testnet-10")

// Channel configuration
export const SOURCE_CHANNEL_ID = ChannelId.make(3)
export const DESTINATION_CHANNEL_ID = ChannelId.make(3)

// Contract addresses
// TODO: Move these to environment-based configuration

// UCS03 on EVM
// TESTNET (Sepolia): 0x5fbe74a283f7954f10aa04c2edf55578811aeb03
export const UCS03_EVM_ADDRESS = "0x5fbe74a283f7954f10aa04c2edf55578811aeb03"

// UCS03 Minter on Union
// MAINNET: union150u2vpdtau48c50lntaqgleu8rqfnnuh2u3pzfg7pfcvw4uzq6tqceagxy
// TESTNET: union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0
export const UCS03_MINTER_ON_UNION = Ucs05.CosmosDisplay.make({
  address: "union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0", // testnet
})

// ZKGM Contract on Union
export const UCS03_ZKGM = Ucs05.CosmosDisplay.make({
  address: "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
})
