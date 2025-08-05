import { Ucs03, Utils } from "@unionlabs/sdk"
import { Hex } from "@unionlabs/sdk/schema/hex"
import { Context, Data } from "effect"
import {
  Account as ViemAccount,
  Chain as ViemChain,
  CreateWalletClientErrorType,
  WalletClient as ViemWalletClient,
} from "viem"
import * as Evm from "./Evm.js"
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

/**
 * @category utils
 * @since 2.0.0
 */
export const sendInstruction = (operand: Hex) =>
  Effect.gen(function*() {
    const walletClient = yield* Wallet
    const sourceConfig = yield* ChannelSource

    const timeoutTimestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()
    const salt = yield* Utils.generateSalt("evm")

    return yield* Evm.writeContract({
      account: walletClient.account,
      abi: Ucs03.Abi,
      chain: walletClient.chain,
      functionName: "send",
      address: sourceConfig.ucs03address,
      args: [
        sourceConfig.channelId,
        0n,
        timeoutTimestamp,
        salt,
        {
          opcode: instruction.opcode,
          version: instruction.version,
          operand,
        },
      ],
      value: 10n,
    })
  })
