import { AllowanceCheckError } from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context"
import { Token, Ucs05, Utils, ZkgmClientRequest } from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { Evm } from "@unionlabs/sdk-evm"
import type { Chain } from "@unionlabs/sdk/schema"
import { Data, Effect, HashMap, Match, Option, pipe, Tuple } from "effect"
import * as A from "effect/Array"
import * as S from "effect/Schema"
import { type Address, http } from "viem"

export class ApprovalStep extends Data.TaggedClass("ApprovalStep")<{
  token: Token.Any
  requiredAmount: bigint
  currentAllowance: bigint
}> {}

export const checkAllowances = Effect.fn((
  context: TransferContext,
): Effect.Effect<Option.Option<A.NonEmptyReadonlyArray<ApprovalStep>>> =>
  Effect.option(Effect.gen(function*() {
    if (A.isEmptyArray(context.intents)) {
      return yield* Option.none()
    }

    const request = yield* context.request

    const firstIntent = yield* A.head(context.intents)
    const chain = request.source
    const sender = firstIntent.sender
    const spender = firstIntent.ucs03address

    const requiredTokens = pipe(
      context.request,
      Option.flatMap(ZkgmClientRequest.requiredFunds),
      Option.getOrElse(() => []),
    )

    const allowances = yield* Match.value(sender).pipe(
      Match.tagsExhaustive({
        EvmDisplay: (sender) =>
          handleEvmAllowances(
            A.map(requiredTokens, Tuple.getFirst),
            sender,
            Utils.ensureHex(spender),
            chain,
          ),
        CosmosDisplay: (sender) =>
          handleCosmosAllowances(
            A.map(requiredTokens, Tuple.getFirst),
            sender,
            chain,
          ),
      }),
      Effect.map(A.map(({ token, allowance }) => [token, allowance] as const)),
      Effect.map(HashMap.fromIterable),
    )

    const result = yield* pipe(
      HashMap.fromIterable(requiredTokens),
      HashMap.reduce(
        A.empty<ApprovalStep>(),
        (acc, v, k) => {
          const allowance = HashMap.get(allowances, k)
          if (Option.isSome(allowance)) {
            if (allowance.value < v) {
              return [
                ...acc,
                new ApprovalStep({
                  currentAllowance: allowance.value,
                  requiredAmount: v,
                  token: k,
                }),
              ]
            }
          }
          return acc
        },
      ),
      Option.liftPredicate(A.isNonEmptyArray),
    )

    return result
  }))
)

const handleEvmAllowances = (
  // XXX: make Address type more specific
  tokens: ReadonlyArray<Token.Any>,
  sender: Ucs05.EvmDisplay,
  spender: Address,
  sourceChain: Chain,
): Effect.Effect<
  ReadonlyArray<{ readonly token: Token.Erc20; readonly allowance: bigint }>,
  AllowanceCheckError | Evm.CreatePublicClientError | Evm.ReadContractError
> =>
  Effect.gen(function*() {
    const chain = yield* sourceChain.toViemChain().pipe(
      Effect.mapError(() => new AllowanceCheckError({ message: "could not" })),
    )

    const client = Evm.PublicClient.Live({
      chain,
      transport: http(),
    })

    return yield* pipe(
      tokens,
      A.filter(S.is(Token.Erc20)),
      A.map((token) =>
        pipe(
          Evm.readErc20Allowance(
            token.address,
            sender.address,
            spender,
          ),
          Effect.map((allowance) => ({
            token,
            allowance,
          })),
        )
      ),
      Effect.allWith({ concurrency: 2 }),
      Effect.provide(client),
    )
  })

export const handleCosmosAllowances = (
  tokenAddresses: Array<Token.Any>,
  sender: Ucs05.CosmosDisplay,
  sourceChain: Chain,
): Effect.Effect<
  ReadonlyArray<{ readonly token: Token.Any; readonly allowance: bigint }>,
  AllowanceCheckError
> =>
  Effect.gen(function*() {
    const rpc = yield* sourceChain.getRpcUrl("rpc").pipe(
      Effect.mapError(() => new AllowanceCheckError({ message: "could not derive rpc" })),
    )
    const client = Cosmos.Client.Live(rpc)

    return yield* pipe(
      tokenAddresses,
      // TODO: check token filtering
      A.filter(S.is(Token.Cw20)),
      A.map((token) =>
        pipe(
          Cosmos.readCw20Allowance(
            // XXX: type assertions
            Ucs05.CosmosDisplay.make({ address: token.address as unknown as any }),
            sender,
            Ucs05.CosmosDisplay.make({
              address: sourceChain.minter_address_display as unknown as any,
            }),
          ),
          Effect.tap((a) => Effect.log("allowance result", a)),
          Effect.map((allowance) => ({
            token,
            allowance: BigInt(allowance),
          })),
        )
      ),
      Effect.allWith({ concurrency: 2 }),
      Effect.provide(client),
      Effect.catchTags({
        "@unionlabs/sdk/Cosmos/QueryContractError": (cause) =>
          new AllowanceCheckError({
            message: "Could not check allowance.",
            cause,
          }),
        "@unionlabs/sdk/Cosmos/ClientError": (cause) =>
          new AllowanceCheckError({
            message: "Could not create Cosmos client.",
            cause,
          }),
        "TimeoutException": (cause) =>
          new AllowanceCheckError({
            message: "Allowance check timed out.",
            cause,
          }),
      }),
    )
  })
