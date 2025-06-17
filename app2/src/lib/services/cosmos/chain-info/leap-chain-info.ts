import * as S from "effect/Schema"

const Bech32Config = S.Struct({
  bech32PrefixAccAddr: S.String,
  bech32PrefixAccPub: S.String,
  bech32PrefixValAddr: S.String,
  bech32PrefixValPub: S.String,
  bech32PrefixConsAddr: S.String,
  bech32PrefixConsPub: S.String,
})
type Bech32Config = typeof Bech32Config.Type
const Currency = S.Struct({
  coinDenom: S.String,
  coinMinimalDenom: S.String,
  coinDecimals: S.Int.pipe(S.positive()),
  coinGeckoId: S.optionalWith(S.String, { exact: true }),
  coinImageUrl: S.optionalWith(S.String, { exact: true }),
})

export const ChainInfo = S.Struct({
  rpc: S.String,
  rest: S.String,
  chainId: S.String,
  chainName: S.String,
  stakeCurrency: Currency,
  walletUrl: S.optionalWith(S.String, { exact: true }),
  walletUrlForStaking: S.optionalWith(S.String, { exact: true }),
  bip44: S.Struct({
    coinType: S.Int.pipe(S.positive()),
  }),
  bech32Config: Bech32Config,
  currencies: S.Array(Currency).pipe(S.mutable),
  feeCurrencies: S.Array(Currency).pipe(S.mutable),
  coinType: S.optionalWith(S.Number, { exact: true }),
  gasPriceStep: S.optionalWith(
    S.Struct({
      low: S.Number,
      average: S.Number,
      high: S.Number,
    }),
    { exact: true },
  ),
  features: S.optionalWith(S.Array(S.String).pipe(S.mutable), { exact: true }),
  beta: S.optionalWith(S.Boolean, { exact: true }),
  theme: S.Struct({
    primaryColor: S.String,
    gradient: S.String,
  }),
  image: S.String,
})
export type ChainInfo = typeof ChainInfo.Type
