import * as S from "effect/Schema"

const EVMInfo = S.Struct({
  chainId: S.Positive,
  rpc: S.String,
  websocket: S.optionalWith(S.String, { exact: true }),
})
type EVMInfo = typeof EVMInfo.Type

const Bech32Config = S.Struct({
  bech32PrefixAccAddr: S.String,
  bech32PrefixAccPub: S.String,
  bech32PrefixValAddr: S.String,
  bech32PrefixValPub: S.String,
  bech32PrefixConsAddr: S.String,
  bech32PrefixConsPub: S.String,
})
type Bech32Config = typeof Bech32Config.Type

const BIP44 = S.Struct({
  coinType: S.Int.pipe(S.positive()),
  purpose: S.optionalWith(S.Number, { exact: true }),
})
type BIP44 = typeof BIP44.Type

const Currency = S.Struct({
  coinDenom: S.String,
  coinMinimalDenom: S.String,
  coinDecimals: S.Positive,
  coinGeckoId: S.optionalWith(
    S.String.pipe(
      S.annotations({
        description: "This is used to fetch asset's fiat value from coingecko.",
        documentation: "You can get the id from https://api.coingecko.com/api/v3/coins/list.",
      }),
    ),
    { exact: true },
  ),
  coinImageUrl: S.optionalWith(S.String, { exact: true }),
}).pipe(
  S.annotations({
    description: "This indicates the type of coin that can be used for stake.",
    documentation: "You can get actual currency information from Currencies.",
  }),
)
type Currency = typeof Currency.Type

const ERC20Currency = Currency.pipe(
  S.extend(S.Struct({
    type: S.Literal("erc20"),
    contractAddress: S.String,
  })),
  S.annotations({
    description: "The currency that is supported on the EVM.",
  }),
)
type ERC20Currency = typeof ERC20Currency.Type

const CW20Currency = Currency.pipe(
  S.extend(S.Struct({
    type: S.Literal("cw20"),
    contractAddress: S.String,
  })),
  S.annotations({
    description: "The currency that is supported on the cosmwasm.",
    documentation:
      `This should be the CW-20 that confirms the standard. And, in this case, \`coinMinimalDenom\` must start with the type and contract address of currency such as "cw20:coral1vv6hruqu...3sfhwh:ukeplr".`,
  }),
)
type CW20Currency = typeof CW20Currency.Type

const Secret20Currency = Currency.pipe(
  S.extend(S.Struct({
    type: S.Literal("secret20"),
    contractAddress: S.String,
    viewingKey: S.String,
  })),
)
type Secret20Currency = typeof Secret20Currency.Type

const IBCCurrency = Currency.pipe(
  S.extend(S.Struct({
    paths: S.Array(S.Struct({
      portId: S.String,
      channelId: S.String,
      counterpartyChannelId: S.optionalWith(S.String, { exact: true }),
      counterpartyPortId: S.optionalWith(S.String, { exact: true }),
      clientChainId: S.optionalWith(S.String, { exact: true }),
    })),
    originChainId: S.UndefinedOr(S.String).pipe(
      S.annotations({
        description: "The chain id that the currency is from.",
        documentation: "If that chain is unknown, this will be undefined.",
      }),
    ),
    originCurrency: S.UndefinedOr(S.Union(Currency, CW20Currency, Secret20Currency)),
  })),
  S.annotations({
    description: "IBCCurrency is the currency that is sent from the other chain via IBC.",
    documentation:
      "This will be handled as similar to the native currency. But, this has more information abounr IBC channel and paths.",
  }),
)
type IBCCurrency = typeof IBCCurrency.Type

const AppCurrency = S.Union(
  Currency,
  ERC20Currency,
  CW20Currency,
  Secret20Currency,
  IBCCurrency,
)
type AppCurrency = typeof AppCurrency.Type

const FeeCurrency = AppCurrency.pipe(
  S.extend(S.Struct({
    gasPriceStep: S.optionalWith(
      S.Struct({
        low: S.Positive,
        average: S.Positive,
        high: S.Positive,
      }).pipe(
        S.annotations({
          description: "This is used to set the fee of the transaction.",
          documentation:
            "If this field is empty, it just use the default gas price step (low: 0.01, average: 0.025, high: 0.04).",
        }),
      ),
      { exact: true },
    ),
  })),
)
type FeeCurrency = typeof FeeCurrency.Type

export const InternalChainInfo = S.Struct({
  rpc: S.String,
  rest: S.String,
  nodeProvider: S.optionalWith(
    S.Struct({
      name: S.String,
      email: S.optionalWith(S.String, { exact: true }),
      discord: S.optionalWith(S.String, { exact: true }),
      website: S.optionalWith(S.String, { exact: true }),
    }),
    { exact: true },
  ),
  chainId: S.String,
  chainName: S.String,
  stakeCurrency: Currency,
  walletUrl: S.optionalWith(S.String, { exact: true }),
  walletUrlForStaking: S.optionalWith(S.String, { exact: true }),
  bip44: BIP44,
  alternativeBIP44s: S.optionalWith(S.Array(BIP44).pipe(S.mutable), { exact: true }),
  bech32Config: Bech32Config,
  currencies: S.Array(AppCurrency).pipe(S.mutable),
  feeCurrencies: S.Array(FeeCurrency).pipe(
    S.mutable,
    S.annotations({
      description: "This indicates which coin or token can be used for fee to send transaction.",
      documentation: "You can get actual currency information from Currencies.",
    }),
  ),
  features: S.optionalWith(
    S.Array(S.String).pipe(
      S.mutable,
      S.annotations({
        description: "Indicate the features supported by this chain. Ex) cosmwasm, secretwasm ...",
      }),
    ),
    { exact: true },
  ),
  beta: S.optionalWith(
    S.Boolean.pipe(
      S.annotations({
        description: "Shows whether the blockchain is in production phase or beta phase.",
        documentation:
          "Major features such as staking and sending are supported on staging blockchains, but without guarantee. If the blockchain is in an early stage, please set it as beta.",
      }),
    ),
    { exact: true },
  ),
  theme: S.Struct({
    primaryColor: S.String,
    gradient: S.String,
  }),
  image: S.String,
  chainSymbolImageUrl: S.optionalWith(S.String, { exact: true }),
  hideInUI: S.optionalWith(S.Boolean, { exact: true }),
  evm: S.optionalWith(EVMInfo, { exact: true }),
})
export type InternalChainInfo = typeof InternalChainInfo.Type
