import { UniversalChainId } from "@unionlabs/sdk/schema"

export const TESTNET_CHAINS: Array<UniversalChainId> = [
  UniversalChainId.make("ethereum.11155111"),
  UniversalChainId.make("corn.21000001"),
  UniversalChainId.make("bob.808813"),
  UniversalChainId.make("babylon.bbn-test-5")
]

export const MAINNET_CHAINS: Array<UniversalChainId> = [
  UniversalChainId.make("bob.60808"),
  UniversalChainId.make("corn.21000000"),
  UniversalChainId.make("babylon.bbn-1"),
  UniversalChainId.make("ethereum.1")
]
