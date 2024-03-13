import type { ChainId } from "./constants/chain.ts"
import type { ExecuteResult } from "@cosmjs/cosmwasm-stargate"
import { type GetBalanceParameters, getBalance, getDenomAddress } from "./query.ts"
import type {
  Hash,
  Chain,
  Client,
  Account,
  Transport,
  PublicActions,
  WalletActions,
  Address
} from "viem"
import {
  sendAsset,
  approveAsset,
  type SendAssetParameters,
  type ApproveAssetParameters
} from "./send.ts"

export type UnionClient = Client & PublicActions & WalletActions

export type UnionActions<
  TTransport extends Transport = Transport,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account | undefined = Account | undefined
> = {
  getDenomAddress: () => Promise<Address>
  approveAsset: (args: ApproveAssetParameters) => Promise<Hash>
  getBalance: (args: GetBalanceParameters) => Promise<bigint>
  sendAsset: <TDenom extends string | undefined, TGas extends `${string}${TDenom}` | undefined>(
    args: SendAssetParameters<ChainId>
  ) => Promise<ExecuteResult | Hash>
}

export const unionActions = <
  TTransport extends Transport = Transport,
  TChain extends Chain | undefined = Chain | undefined,
  TAccount extends Account | undefined = Account | undefined
>(
  client: Client<TTransport, TChain, TAccount> & PublicActions & WalletActions
): UnionActions<TTransport, TChain, TAccount> => ({
  getDenomAddress: () => getDenomAddress(client),
  approveAsset: args => approveAsset(client, args),
  getBalance: args => getBalance(client, args),
  sendAsset: args => sendAsset(client, args)
})
