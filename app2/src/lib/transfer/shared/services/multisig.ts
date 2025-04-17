import { Record as R, Data, Effect, Option, pipe } from "effect"
import * as S from "effect/Schema"
import { Tx } from "@unionlabs/sdk/schema"
import { encodeAbiParameters } from "viem"
import { instructionAbi } from "@unionlabs/sdk/evm/abi"
import { encodeAbi } from "@unionlabs/sdk/ucs03/instruction"
import { cosmosSpenderAddresses } from "$lib/constants/spender-addresses.ts"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"
import { generateSalt } from "@unionlabs/sdk/utils"
import { isValidBech32ContractAddress } from "$lib/utils/index.ts"
import { getTimeoutInNanoseconds24HoursFromNow } from "@unionlabs/sdk/utils/timeout.ts"

export class GenerateMultisigError extends Data.TaggedError("GenerateMultisigError")<{
  reason: string
  cause?: unknown
}> {}

export const generateMultisigTx = (context: TransferContext) =>
  Effect.gen(function* () {
    console.log("[generateMultisigTx] intent:", JSON.parse(JSON.stringify(context)))

    const txToJson = S.encodeUnknown(S.parseJson(Tx))
    const sender = yield* context.intents[0].sourceChain.getDisplayAddress(
      context.intents[0].sender
    )
    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow().toString()
    const salt = yield* generateSalt("cosmos")
    console.log("[generateMultisigTx] generated salt:", salt)

    const allowanceMsgs = pipe(
      context.allowances,
      Option.map(allowances =>
        allowances.flatMap(allowance => {
          console.log("[allowance] token:", allowance.token)
          return context.intents.flatMap(intent => {
            console.log("[context] sourceChainId:", intent.sourceChainId)
            console.log("[context] sender:", intent.sender)
            const maybeSpender = R.get(cosmosSpenderAddresses, intent.sourceChainId)
            if (Option.isNone(maybeSpender)) {
              console.warn("[warning] no spender for chain:", intent.sourceChainId)
              return []
            }

            const spender = maybeSpender.value
            console.log("[spender] resolved:", spender)

            return [
              {
                "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
                sender,
                contract: allowance.token,
                msg: {
                  increase_allowance: {
                    spender,
                    amount: allowance.requiredAmount
                  }
                },
                funds: []
              }
            ]
          })
        })
      ),
      Option.getOrElse(() => [])
    )

    const instructionMsgs = pipe(
      context.instruction,
      Option.map(instruction => {
        console.log("[instruction] opcode:", instruction.opcode)
        return context.intents.map(intent => {
          console.log("[context] ucs03address:", intent.ucs03address)
          console.log("[context] baseToken:", intent.baseToken)
          console.log("[context] baseAmount:", intent.baseAmount)
          const isNative = !isValidBech32ContractAddress(intent.baseToken)
          const encodedInstruction = encodeAbiParameters(instructionAbi, [
            instruction.version,
            instruction.opcode,
            encodeAbi(instruction)
          ])

          console.log("[isNative] ", isNative)
          console.log("[instruction] encodedAbi:", encodedInstruction)

          return {
            "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
            sender,
            contract: intent.ucs03address,
            msg: {
              send: {
                channel_id: intent.sourceChannelId,
                timeout_height: "0",
                timeout_timestamp: timeoutTimestamp,
                salt,
                instruction: encodedInstruction
              }
            },
            funds: isNative
              ? [
                  {
                    denom: intent.baseToken,
                    amount: intent.baseAmount
                  }
                ]
              : []
          }
        })
      }),
      Option.getOrElse(() => [])
    )

    const allMsgs = [...allowanceMsgs, ...instructionMsgs]
    console.log("[generateMultisigTx] allMsgs count:", allMsgs.length)
    console.dir(allMsgs)

    const encoded = txToJson({
      body: {
        messages: allMsgs
      }
    })

    return yield* encoded
  }).pipe(
    Effect.catchAll(cause => {
      console.error("[generateMultisigTx] Fiber failure:", cause)

      return Effect.fail(
        new GenerateMultisigError({
          reason: "Failed to generate multisig tx",
          cause
        })
      )
    })
  )
