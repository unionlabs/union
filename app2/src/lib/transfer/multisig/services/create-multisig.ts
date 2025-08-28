import { GenerateMultisigError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context"
import { isValidBech32ContractAddress } from "$lib/utils/index"
import { Call, Token, TokenOrder, Ucs03, ZkgmInstruction } from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { tokenMetaOverride } from "@unionlabs/sdk/Constants"
import { Tx } from "@unionlabs/sdk/schema/tx"
import { generateSalt } from "@unionlabs/sdk/utils/index"
import { getTimeoutInNanoseconds24HoursFromNow } from "@unionlabs/sdk/utils/timeout"
import { Cause, Effect, Match, Option, ParseResult, pipe } from "effect"
import * as A from "effect/Array"
import * as E from "effect/Either"
import * as S from "effect/Schema"

export const createMultisigMessage = (context: TransferContext) =>
  Effect.gen(function*() {
    const txToJson = S.encodeUnknown(S.parseJson(Tx))
    // XXX: discriminate higher
    const sender = context.intents[0].sender.address
    const timeoutTimestamp = getTimeoutInNanoseconds24HoursFromNow().toString()
    const salt = yield* generateSalt("cosmos")
    const chain = context.intents[0].sourceChain
    const rpcUrl = yield* chain.getRpcUrl("rpc")
    const publicClient = Cosmos.Client.Live(rpcUrl)

    const allowanceMsgs = pipe(
      context.allowances,
      Option.map((allowances) =>
        allowances.flatMap((allowance) => {
          return context.intents.flatMap((intent) => {
            return [
              {
                "@type": "/cosmwasm.wasm.v1.MsgExecuteContract",
                sender,
                "contract": allowance.token.address,
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

    const instructionMsgs = yield* pipe(
      context.request,
      Option.map((request) => {
        return context.intents.map((intent) =>
          Effect.gen(function*() {
            const isNative = Token.isNative(intent.baseToken)

            const encodeInstruction: (
              u: ZkgmInstruction.ZkgmInstruction,
            ) => Effect.Effect<
              Ucs03.Ucs03,
              | ParseResult.ParseError
              | Cause.TimeoutException
              | Cosmos.QueryContractError
              | Cosmos.ClientError
            > = pipe(
              Match.type<ZkgmInstruction.ZkgmInstruction>(),
              Match.tagsExhaustive({
                Batch: (batch) =>
                  pipe(
                    batch.instructions,
                    A.map(encodeInstruction),
                    Effect.allWith({ concurrency: "unbounded" }),
                    Effect.map((operand) =>
                      new Ucs03.Batch({
                        opcode: batch.opcode,
                        version: batch.version,
                        operand,
                      })
                    ),
                  ),
                TokenOrder: (self) =>
                  pipe(
                    Match.value(self),
                    Match.when(
                      { version: 1 },
                      (v1) =>
                        Effect.gen(function*() {
                          const meta = yield* pipe(
                            Cosmos.readCw20TokenInfo(v1.baseToken.address as unknown as any),
                            Effect.either,
                            Effect.map(
                              E.getOrElse(() => tokenMetaOverride(v1.baseToken.address)),
                            ),
                            Effect.provide(publicClient),
                          )

                          return yield* TokenOrder.encodeV1(v1)({
                            ...meta,
                            sourceChannelId: request.channelId,
                          })
                        }),
                    ),
                    Match.when(
                      { version: 2 },
                      (v2) => TokenOrder.encodeV2(v2),
                    ),
                    Match.exhaustive,
                  ),
                Call: Call.encode,
              }),
            )

            const encodedInstruction = yield* pipe(
              encodeInstruction(request.instruction),
              Effect.flatMap(S.encode(Ucs03.Ucs03WithInstructionFromHex)),
            )

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
                    denom: intent.baseToken.address,
                    amount: intent.baseAmount,
                  },
                ]
                : [],
            }
          })
        )
      }),
      Option.flatMap(Option.liftPredicate(A.isNonEmptyArray)),
      Option.map(Effect.allWith({ concurrency: "unbounded" })),
      Effect.transposeOption,
    )

    const _instructionMsgs = yield* pipe(
      instructionMsgs,
      Effect.mapError((cause) =>
        new GenerateMultisigError({
          reason: "instructionmsgs is none",
          cause,
        })
      ),
    )

    const allMsgs = [...allowanceMsgs, ..._instructionMsgs]

    return yield* pipe(
      txToJson({
        body: {
          messages: allMsgs,
        },
      }),
      Effect.mapError((cause) =>
        new GenerateMultisigError({
          reason: "Could not generate transaction JSON.",
          cause,
        })
      ),
    )
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
