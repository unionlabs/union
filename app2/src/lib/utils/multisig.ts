import { Array as A, Record as R, Data, Effect, Option, pipe, Struct } from "effect"
import * as S from "effect/Schema"
import { Tx } from "@unionlabs/sdk/schema"
import { encodeAbiParameters, fromHex } from "viem"
import { instructionAbi } from "@unionlabs/sdk/evm/abi"
import { encodeAbi } from "@unionlabs/sdk/ucs03/instruction"
import { cosmosSpenderAddresses } from "$lib/constants/spender-addresses.ts"
import type { TransferIntents } from "$lib/components/Transfer/state/filling/create-intents.ts"
import { generateSalt } from "@unionlabs/sdk/utils"

export class GenerateMultisigError extends Data.TaggedError("GenerateMultisigError")<{
  reason: string
  cause?: unknown
}> {}

export const generateMultisigTx = (intents: TransferIntents) =>
  Effect.gen(function* () {
    const txToJson = S.encodeUnknown(S.parseJson(Tx))
    const timeoutTimestamp = "0" // TODO
    const isNative = true //TODO

    const salt = yield* generateSalt("cosmos")

    const intentMessages = pipe(
      intents,
      A.map(intent =>
        pipe(
          intent,
          Struct.evolve({
            allowance: maybeAllowance =>
              pipe(
                Option.Do.pipe(
                  Option.bind("allowance", () => maybeAllowance),
                  Option.bind("spender", () =>
                    R.get(cosmosSpenderAddresses, intent.context.sourceChainId)
                  ),
                  Option.map(({ allowance, spender }) => ({
                    "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
                    sender: intent.context.sender,
                    // @ts-expect-error
                    contract: fromHex(allowance.token, "string"),
                    msg: {
                      increase_allowance: {
                        spender,
                        amount: allowance.requiredAmount.toString()
                      }
                    },
                    funds: []
                  }))
                )
              ),
            instruction: instruction =>
              pipe(
                instruction,
                Option.map(i =>
                  encodeAbiParameters(instructionAbi, [i.version, i.opcode, encodeAbi(i)])
                ),
                encoded => ({
                  "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
                  sender: intent.context.sender,
                  contract: fromHex(intent.context.ucs03address, "string"),
                  msg: {
                    send: {
                      channel_id: intent.context.sourceChannelId,
                      timeout_height: "0",
                      timeout_timestamp: timeoutTimestamp,
                      salt,
                      instruction: encoded
                    }
                  },
                  funds: isNative
                    ? [
                        {
                          denom: fromHex(intent.context.baseToken, "string"),
                          amount: intent.context.baseAmount.toString()
                        }
                      ]
                    : []
                })
              )
          })
        )
      )
    )

    const allowanceMessages = pipe(
      intentMessages,
      A.map(x => x.allowance),
      A.getSomes
    )

    const orderMessages = pipe(
      intentMessages,
      A.map(x => x.instruction)
    )

    const encodedMessages = txToJson({
      body: {
        messages: [...allowanceMessages, ...orderMessages]
      }
    })

    return yield* encodedMessages
  })
