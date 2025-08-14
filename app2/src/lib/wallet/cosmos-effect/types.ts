import * as S from "effect/Schema"

// Transaction message types
export const CosmosMessage = S.Struct({
  "@type": S.String,
  sender: S.String,
  contract: S.optional(S.String),
  msg: S.optional(S.Record({ key: S.String, value: S.Unknown })),
  funds: S.optional(S.Array(S.Struct({
    denom: S.String,
    amount: S.String,
  }))),
  spender: S.optional(S.String),
  amount: S.optional(S.String),
})

// Transaction body
export const TransactionBody = S.Struct({
  messages: S.Array(CosmosMessage),
  memo: S.String,
  timeout_height: S.String,
  unordered: S.Boolean,
  timeout_timestamp: S.NullOr(S.String),
  extension_options: S.Array(S.Unknown),
  non_critical_extension_options: S.Array(S.Unknown),
})

// Signer info
export const SignerInfo = S.Struct({
  public_key: S.Struct({
    "@type": S.String,
    key: S.String,
  }),
  mode_info: S.Struct({
    single: S.Struct({
      mode: S.String,
    }),
  }),
  sequence: S.String,
})

// Fee structure
export const FeeInfo = S.Struct({
  amount: S.Array(S.Struct({
    denom: S.String,
    amount: S.String,
  })),
  gas_limit: S.String,
  payer: S.String,
  granter: S.String,
})

// Auth info
export const AuthInfo = S.Struct({
  signer_infos: S.Array(SignerInfo),
  fee: FeeInfo,
  tip: S.NullOr(S.Unknown),
})

// Complete transaction
export const Transaction = S.Struct({
  body: TransactionBody,
  auth_info: AuthInfo,
  signatures: S.Array(S.String),
})

// Event attribute
export const EventAttribute = S.Struct({
  key: S.String,
  value: S.String,
  index: S.Boolean,
})

// Event
export const TransactionEvent = S.Struct({
  type: S.String,
  attributes: S.Array(EventAttribute),
})

// Transaction response
export const TransactionResponse = S.Struct({
  height: S.String,
  txhash: S.String,
  codespace: S.String,
  code: S.Number,
  data: S.String,
  raw_log: S.String,
  logs: S.Array(S.Unknown),
  info: S.String,
  gas_wanted: S.String,
  gas_used: S.String,
  tx: Transaction,
  timestamp: S.String,
  events: S.Array(TransactionEvent),
})

// Bridge instruction data (decoded from hex)
export const BridgeInstruction = S.Struct({
  channel_id: S.Number,
  timeout_height: S.String,
  timeout_timestamp: S.String,
  salt: S.String,
  instruction: S.String,
})

// Token info
export const TokenInfo = S.Struct({
  symbol: S.String,
  name: S.String,
  contract: S.optional(S.String),
  denom: S.optional(S.String),
  decimals: S.optional(S.Number),
})

// Parsed transaction data
export const ParsedTransaction = S.Struct({
  hash: S.String,
  height: S.Number,
  timestamp: S.Date,
  sender: S.String,
  type: S.Literal("bridge", "transfer", "allowance", "contract_execution"),
  status: S.Literal("success", "failed"),
  gas_used: S.Number,
  gas_wanted: S.Number,
  fee: S.Struct({
    amount: S.String,
    denom: S.String,
  }),
  tokens: S.Array(TokenInfo),
  bridge_info: S.optional(S.Struct({
    source_chain: S.String,
    destination_chain: S.String,
    channel_id: S.Number,
    amount: S.String,
    token: TokenInfo,
  })),
  contract_executions: S.Array(S.String),
})

// Transaction list
export const TransactionList = S.Struct({
  txs: S.Array(Transaction),
  tx_responses: S.Array(TransactionResponse),
  pagination: S.NullOr(S.Struct({
    next_key: S.optional(S.String),
    total: S.optional(S.String),
  })),
  total: S.String,
})

// Type exports
export type CosmosMessage = typeof CosmosMessage.Type
export type TransactionBody = typeof TransactionBody.Type
export type Transaction = typeof Transaction.Type
export type TransactionResponse = typeof TransactionResponse.Type
export type TransactionEvent = typeof TransactionEvent.Type
export type BridgeInstruction = typeof BridgeInstruction.Type
export type TokenInfo = typeof TokenInfo.Type
export type ParsedTransaction = S.Schema.Type<typeof ParsedTransaction>
export type TransactionList = typeof TransactionList.Type

// Contract addresses (from the provided data)
export const KNOWN_CONTRACTS = {
  BRIDGE: "xion1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qlzhdk9",
  LINK_TOKEN: "xion1a8fpr9850azpxhdjfxzzxnkk5dsl68t6dmtry7ureva8rq0rs2ds3r3rwp",
  SEI_TOKEN: "xion1tms92cm34lxln4kvxw2xdsgncumzepr5e2eug90vmtyw55z8djuqvwnee7",
  ROUTER: "xion1ak8muzgf2nv5ukzeg2wpf2vls74247et33cfhmg87uvpey73xdeqexkzgc",
  IBC_HANDLER: "xion1hnuj8f6d3wy3fcprt55vddv7v2650t6uudnvd2hukqrteeam8wjqjrkp8m",
} as const

// Channel mappings
export const BRIDGE_CHANNELS = {
  2: "BBN", // Babylon
  6: "ETH", // Ethereum (via IBC)
  19: "OSMO", // Osmosis
} as const

// Token mappings
export const TOKEN_INFO: Record<string, TokenInfo> = {
  [KNOWN_CONTRACTS.LINK_TOKEN]: {
    symbol: "LINK",
    name: "ChainLink Token",
    contract: KNOWN_CONTRACTS.LINK_TOKEN,
    decimals: 18,
  },
  [KNOWN_CONTRACTS.SEI_TOKEN]: {
    symbol: "SEI",
    name: "Sei",
    contract: KNOWN_CONTRACTS.SEI_TOKEN,
    decimals: 6,
  },
  "uxion": {
    symbol: "XION",
    name: "Xion",
    denom: "uxion",
    decimals: 6,
  },
} 