import * as S from "effect/Schema"
import { Bech32 } from "./bech32.js"
import { Uint128 } from "./uint128.js"
import { Uint64FromString } from "./uint64.js"

export const Coin = S.Struct({ denom: S.String, amount: Uint128 })

export const Msg = S.Struct({
  "@type": S.String
})

export const MsgExecuteContract = S.extend(
  Msg,
  S.Struct({
    sender: Bech32,
    contract: Bech32,
    msg: S.Any,
    funds: S.Array(Coin)
  })
)

// This message: https://github.com/cosmos/cosmos-sdk/blob/ccd37e1d993ba3f220ea61d5e8dffd43a894f68c/proto/cosmos/tx/v1beta1/tx.proto#L15-L28
//
// ...but hardcoded to only contain MsgExecuteContract. Intended for multisig signing in multisig.keplr.app.
export const Tx = S.Struct({
  body: S.Struct({
    messages: S.NonEmptyArray(MsgExecuteContract)
  }),
})

export type Coin = typeof Coin.Type
export type Msg = typeof Msg.Type
export type MsgExecuteContract = typeof MsgExecuteContract.Type
export type Tx = typeof Tx.Type
