import { Ucs05 } from "@unionlabs/sdk"
import { ChannelId, UniversalChainId } from "@unionlabs/sdk/schema"
import { Match } from "effect"

export const LST_CONFIGS = ["mainnet", "sepolia", "custom"] as const
export type LSTConfig = typeof LST_CONFIGS[number]

export const LST_CONFIG_LABELS: Record<LSTConfig, string> = {
  mainnet: "Mainnet ↔ Union",
  sepolia: "Sepolia ↔ Union Testnet",
  custom: "Custom",
}

class LSTConfigState {
  ethereumChainId = $state("ethereum.1")
  unionChainId = $state("union.union-1")

  sourceChannelId = $state(2)
  destinationChannelId = $state(1)

  // EVM RPC Endpoint
  evmRpcEndpoint = $state("https://rpc.1.ethereum.chain.kitchen")

  // Union/Cosmos RPC Endpoint
  // MAINNET: https://rpc.union.build
  // TESTNET: https://rpc.union-testnet-10.union.chain.kitchen
  unionRpcEndpoint = $state("https://rpc.union.build")

  ucs03EvmAddress = $state<`0x${string}`>("0x5fbe74a283f7954f10aa04c2edf55578811aeb03")

  // UCS03 Minter on Union
  // MAINNET: union150u2vpdtau48c50lntaqgleu8rqfnnuh2u3pzfg7pfcvw4uzq6tqceagxy
  // TESTNET: union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0
  ucs03MinterOnUnion = $state<`${string}1${string}`>("union150u2vpdtau48c50lntaqgleu8rqfnnuh2u3pzfg7pfcvw4uzq6tqceagxy")

  // ZKGM Contract on Union
  ucs03Zkgm = $state<`${string}1${string}`>("union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh")

  // Derived getters that return the proper typed values
  get ETHEREUM_CHAIN_ID() {
    return UniversalChainId.make(this.ethereumChainId)
  }

  get UNION_CHAIN_ID() {
    return UniversalChainId.make(this.unionChainId)
  }

  get SOURCE_CHANNEL_ID() {
    return ChannelId.make(this.sourceChannelId)
  }

  get DESTINATION_CHANNEL_ID() {
    return ChannelId.make(this.destinationChannelId)
  }

  get EVM_RPC_ENDPOINT() {
    return this.evmRpcEndpoint
  }

  get UNION_RPC_ENDPOINT() {
    return this.unionRpcEndpoint
  }

  get UCS03_EVM_ADDRESS() {
    return this.ucs03EvmAddress
  }

  get UCS03_MINTER_ON_UNION() {
    return Ucs05.CosmosDisplay.make({
      address: this.ucs03MinterOnUnion,
    })
  }

  get UCS03_ZKGM() {
    return Ucs05.CosmosDisplay.make({
      address: this.ucs03Zkgm,
    })
  }

  loadPredefined(val: LSTConfig) {
    Match.value(val).pipe(
      Match.when("mainnet", () => {
        this.ethereumChainId = "ethereum.1"
        this.unionChainId = "union.union-1"
        this.sourceChannelId = 2
        this.destinationChannelId = 1
        this.evmRpcEndpoint = "https://rpc.1.ethereum.chain.kitchen"
        this.unionRpcEndpoint = "https://rpc.union.build"
        this.ucs03EvmAddress = "0x5fbe74a283f7954f10aa04c2edf55578811aeb03"
        this.ucs03MinterOnUnion = "union150u2vpdtau48c50lntaqgleu8rqfnnuh2u3pzfg7pfcvw4uzq6tqceagxy"
        this.ucs03Zkgm = "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh"
      }),
      Match.when("sepolia", () => {
        this.ethereumChainId = "ethereum.11155111"
        this.unionChainId = "union.union-testnet-10"
        this.sourceChannelId = 3
        this.destinationChannelId = 3
        this.evmRpcEndpoint = "https://rpc.11155111.ethereum.chain.kitchen"
        this.unionRpcEndpoint = "https://rpc.union-testnet-10.union.chain.kitchen"
        this.ucs03EvmAddress = "0x5fbe74a283f7954f10aa04c2edf55578811aeb03"
        this.ucs03MinterOnUnion = "union1t5awl707x54k6yyx7qfkuqp890dss2pqgwxh07cu44x5lrlvt4rs8hqmk0"
        this.ucs03Zkgm = "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh"
      }),
      Match.when("custom", () => {}),
      Match.exhaustive,
    )
  }
}

export const lstConfig = new LSTConfigState()
