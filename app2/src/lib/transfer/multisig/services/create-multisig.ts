import { GenerateMultisigError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"
import { isValidBech32ContractAddress } from "$lib/utils/index.ts"
import { instructionAbi } from "@unionlabs/sdk/evm/abi"
import { AddressCosmosCanonical, Tx } from "@unionlabs/sdk/schema"
import { encodeAbi } from "@unionlabs/sdk/ucs03/instruction"
import { generateSalt } from "@unionlabs/sdk/utils"
import { getTimeoutInNanoseconds24HoursFromNow } from "@unionlabs/sdk/utils/timeout.ts"
import { Effect, Option, pipe } from "effect"
import * as S from "effect/Schema"
import { encodeAbiParameters, fromHex } from "viem"

export const createMultisigMessage = (context: TransferContext) =>
  Effect.gen(function*() {
    const txToJson = S.encodeUnknown(S.parseJson(Tx))
    const sender = yield* context.intents[0].sourceChain.getDisplayAddress(
      // XXX: discriminate higher
      context.intents[0].sender as AddressCosmosCanonical,
    )
    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow().toString()
    const salt = yield* generateSalt("cosmos")

    const allowanceMsgs = pipe(
      context.allowances,
      Option.map((allowances) =>
        allowances.flatMap((allowance) => {
          return context.intents.flatMap((intent) => {
            return [
              {
                "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
                sender,
                "contract": fromHex(allowance.token, "string"),
                "msg": {
                  increase_allowance: {
                    spender: intent.sourceChain.minter_address_display,
                    amount: allowance.requiredAmount,
                  },
                },
                "funds": [],
              },
            ]
          })
        })
      ),
      Option.getOrElse(() => []),
    )

    const instructionMsgs = pipe(
      context.instruction,
      Option.map((instruction) => {
        return context.intents.map((intent) => {
          const isNative = !isValidBech32ContractAddress(intent.baseToken)
          const encodedInstruction = encodeAbiParameters(instructionAbi, [
            instruction.version,
            instruction.opcode,
            encodeAbi(instruction),
          ])

          return {
            "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
            sender,
            "contract": intent.ucs03address,
            "msg": {
              send: {
                channel_id: intent.sourceChannelId,
                timeout_height: "0",
                timeout_timestamp: timeoutTimestamp,
                salt,
                instruction: encodedInstruction,
              },
            },
            "funds": isNative
              ? [
                {
                  denom: intent.baseToken,
                  amount: intent.baseAmount,
                },
              ]
              : [],
          }
        })
      }),
      Option.getOrElse(() => []),
    )

    const allMsgs = [...allowanceMsgs, ...instructionMsgs]

    const encoded = txToJson({
      body: {
        messages: allMsgs,
      },
    })

    return yield* encoded
  }).pipe(
    Effect.catchAll((cause) => {
      console.error("[generateMultisigTx] Fiber failure:", cause)

      return Effect.fail(
        new GenerateMultisigError({
          reason: "Failed to generate multisig tx",
          cause,
        }),
      )
    }),
  )
