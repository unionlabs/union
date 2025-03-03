type EVMTransferInput = {
  type: "evm"
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
  type: "cosmos"
  baseToken: string
  baseAmount: string
  quoteToken: string
  quoteAmount: string
  sourceChannelId: number
  wethToken: null
  receiver: string
  ucs03address: string
}

type AptosTransferInput = {
  type: "aptos"
  baseToken: string
  baseAmount: string
  quoteToken: string
  quoteAmount: string
  sourceChannelId: number
  wethToken: null
  receiver: string
  ucs03address: string
}

export const examples: {
  evm: EVMTransferInput
  cosmos: CosmosTransferInput
  aptos: AptosTransferInput
} = {
  evm: {
    type: "evm",
    baseToken: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
    baseAmount: "1000000000000000000",
    quoteToken: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
    quoteAmount: "1000000",
    receiver: "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
    sourceChannelId: 1,
    ucs03address: "0x742d35cc6634c0532925a3b844bc454e4438f44e",
    wethToken: "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"
  },
  cosmos: {
    type: "cosmos",
    baseToken: "0x1234567890abcdef1234567890abcdef12345678",
    baseAmount: "10000000",
    quoteToken: "0xabcdef1234567890abcdef1234567890abcdef12",
    quoteAmount: "10000000",
    receiver: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    sourceChannelId: 2,
    ucs03address: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890", // Hex, 32 bytes
    wethToken: null
  },
  aptos: {
    type: "aptos",
    baseToken: "0x1abcdef1234567890abcdef1234567890abcdef12",
    baseAmount: "1000000000000000000",
    quoteToken: "0x2abcdef1234567890abcdef1234567890abcdef12",
    quoteAmount: "10000000",
    receiver: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
    sourceChannelId: 3,
    ucs03address: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    wethToken: null
  }
}
