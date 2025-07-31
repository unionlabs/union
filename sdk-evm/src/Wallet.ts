import { Context, Data } from "effect"
import {
  Account as ViemAccount,
  Chain as ViemChain,
  CreateWalletClientErrorType,
  WalletClient as ViemWalletClient,
} from "viem"
import * as internal from "./internal/wallet.js"

export interface Wallet {
  readonly client: ViemWalletClient
  readonly account: ViemAccount
  readonly chain: ViemChain
}

/**
 * @category errors
 * @since 2.0.0
 */
export class CreateWalletError
  extends Data.TaggedError("@unionlabs/sdk-evm/Wallet/CreateWalletError")<{
    cause: CreateWalletClientErrorType
  }>
{}

/**
 * A wallet client that can be used for signing transactions
 *
 * @category context
 * @since 2.0.0
 */
export class Wallet extends Context.Tag("@unionlabs/sdk-evm/Wallet/Wallet")<
  Wallet,
  Wallet
>() {
  static Live = internal.walletClientLayer(this)
}
