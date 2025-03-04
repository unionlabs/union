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
    baseToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
    baseAmount: "1000",
    quoteToken: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    quoteAmount: "1000",
    receiver: "union10z7xxj2m8q3f7j58uxmff38ws9u8m0vmne2key",
    sourceChannelId: 1,
    ucs03address: "0x742d35cc6634c0532925a3b844bc454e4438f44e",
    wethToken: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
  },
  cosmos: {
    sourceRpcType: "cosmos",
    destinationRpcType: "evm",
    baseToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
    baseAmount: "1000",
    quoteToken: "0xabcdef1234567890abcdef1234567890abcdef12",
    quoteAmount: "1000",
    receiver: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    sourceChannelId: 2,
    ucs03address: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
  },
  aptos: {
    sourceRpcType: "aptos",
    destinationRpcType: "evm",
    baseToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238",
    baseAmount: "1000",
    quoteToken: "0x2abcdef1234567890abcdef1234567890abcdef12",
    quoteAmount: "1000",
    receiver: "0x1f9090aae28b8a3dceadf281b0f12828e676c326",
    sourceChannelId: 3,
    ucs03address: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
  }
}
