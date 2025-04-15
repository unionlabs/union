import { Data, Effect, Option } from "effect"
import * as S from "effect/Schema"
import { Tx } from "@unionlabs/sdk/schema"
import { encodeAbiParameters, fromHex } from "viem"
import { instructionAbi } from "@unionlabs/sdk/evm/abi"
import { encodeAbi } from "@unionlabs/sdk/ucs03/instruction"
import { cosmosSpenderAddresses } from "$lib/constants/spender-addresses.ts"
import type {TransferIntents} from "$lib/components/Transfer/state/filling/create-intents.ts";

export class GenerateMultisigError extends Data.TaggedError("GenerateMultisigError")<{
  reason: string
  cause?: unknown
}> {}

export const generateMultisigTx = (intents: TransferIntents) =>
  Effect.gen(function* () {
    const txToJson = S.encodeUnknown(S.parseJson(Tx))

    const salt = "0x1234" // TODO
    const timeoutTimestamp = "0" // TODO
    const isNative = true //TODO

    const approvalMsgs = intents.flatMap((intent) =>
      Option.match(intent.allowances, {
        onNone: () => [],
        onSome: ({ token, requiredAmount }) => {
          const spender = cosmosSpenderAddresses[intent.context.sourceChainId]
          if (!spender) {
            throw new GenerateMultisigError({
              reason: `Missing spender address for chain ${intent.context.sourceChainId}`,
            })
          }

          return [
            {
              "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
              sender: intent.context.sender,
              contract: fromHex(token, "string"),
              msg: {
                increase_allowance: {
                  spender,
                  amount: requiredAmount.toString(),
                },
              },
              funds: [],
            },
          ]
        },
      })
    )

    const batchInstruction = intents.find((i) => Option.isSome(i.instructions))
    const instruction = batchInstruction?.instructions
    const encoded = Option.flatMap(instruction, (inst) =>
      Effect.succeed(
        encodeAbiParameters(instructionAbi, [
          inst.version,
          inst.opcode,
          encodeAbi(inst),
        ])
      )
    )

    if (Option.isNone(instruction)) {
      return yield* Effect.fail(
        new GenerateMultisigError({ reason: "No instruction found in intents" })
      )
    }

    const anyIntent = intents[0] // any is fine since grouped intents share channel/sender
    const orderMsg = {
      "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
      sender: anyIntent.context.sender,
      contract: fromHex(anyIntent.context.ucs03address, "string"),
      msg: {
        send: {
          channel_id: anyIntent.context.sourceChannelId,
          timeout_height: "0",
          timeout_timestamp: timeoutTimestamp,
          salt,
          instruction: yield* encoded,
        },
      },
      funds: isNative
        ? [
          {
            denom: fromHex(anyIntent.context.baseToken, "string"),
            amount: anyIntent.context.baseAmount.toString(),
          },
        ]
        : [],
    }

    return yield* Effect.try({
      try: () =>
        txToJson({
          body: {
            messages: [...approvalMsgs, orderMsg],
          },
        }),
      catch: (cause) =>
        new GenerateMultisigError({
          reason: "Failed to encode multisig transaction",
          cause,
        }),
    })
  })
