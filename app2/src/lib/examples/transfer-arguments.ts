import type { RpcType } from "$lib/schema/chain"

type EVMTransferInput = {
  sourceRpcType: "evm"
  destinationRpcType: typeof RpcType.Type
  baseToken: string
  baseAmount: string
  quoteToken: string
  quoteAmount: string
  sourceChannelId: number
  wethToken: string
  receiver: string
  ucs03address: string
}

type CosmosTransferInput = {
  sourceRpcType: "cosmos"
  destinationRpcType: typeof RpcType.Type
  baseToken: string
  baseAmount: string
  quoteToken: string
  quoteAmount: string
  sourceChannelId: number
  receiver: string
  ucs03address: string
}

type AptosTransferInput = {
  sourceRpcType: "aptos"
  destinationRpcType: typeof RpcType.Type
  baseToken: string
  baseAmount: string
  quoteToken: string
  quoteAmount: string
  sourceChannelId: number
  receiver: string
  ucs03address: string
}

export const examples: {
  evm: EVMTransferInput
  cosmos: CosmosTransferInput
  aptos: AptosTransferInput
} = {
  evm: {
    sourceRpcType: "evm",
    destinationRpcType: "cosmos",
    baseToken: "enter-test-data",
    baseAmount: "1000",
    quoteToken: "enter-test-data",
    quoteAmount: "100",
    receiver: "enter-test-data",
    sourceChannelId: 1,
    ucs03address: "enter-test-data",
    wethToken: "enter-test-data"
  },
  cosmos: {
    sourceRpcType: "cosmos",
    destinationRpcType: "evm",
    baseToken: "enter-test-data",
    baseAmount: "100",
    quoteToken: "enter-test-data",
    quoteAmount: "100",
    receiver: "enter-test-data",
    sourceChannelId: 2,
    ucs03address: "enter-test-data"
  },
  aptos: {
    sourceRpcType: "aptos",
    destinationRpcType: "evm",
    baseToken: "enter-test-data",
    baseAmount: "100",
    quoteToken: "enter-test-data",
    quoteAmount: "100",
    receiver: "enter-test-data",
    sourceChannelId: 3,
    ucs03address: "enter-test-data"
  }
}
