import {
  AllowanceCheckError,
  type ContextFlowError,
  CosmosQueryError,
} from "$lib/transfer/shared/errors"
import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import { Token, type Ucs05, ZkgmClientRequest } from "@unionlabs/sdk"
import { Utils } from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { Evm } from "@unionlabs/sdk-evm"
import { GAS_DENOMS } from "@unionlabs/sdk/constants/gas-denoms"
import { CosmWasmClientSource, createCosmWasmClient } from "@unionlabs/sdk/cosmos"
import {
  createViemPublicClient,
  CreateViemPublicClientError,
  ReadContractError,
  readErc20Allowance,
  ViemPublicClient,
} from "@unionlabs/sdk/evm"
import type { AddressCanonicalBytes, AddressCosmosCanonical, Chain } from "@unionlabs/sdk/schema"
import { Data, Effect, identity, Match, Option, pipe, Tuple } from "effect"
import * as A from "effect/Array"
import * as S from "effect/Schema"
import { type Address, fromHex, http, isHex } from "viem"

export class ApprovalStep extends Data.TaggedClass("ApprovalStep")<{
  token: Token.Any
  requiredAmount: bigint
  currentAllowance: bigint
}> {}

function gatherNeededAmounts(
  contexts: Array<{ baseToken: string; baseAmount: bigint }>,
) {
  const map = new Map<string, bigint>()
  for (const { baseToken, baseAmount } of contexts) {
    const current = map.get(baseToken) ?? 0n
    map.set(baseToken, current + baseAmount)
  }
  return map
}

export const checkAllowances = Effect.fn((
  context: TransferContext,
): Effect.Effect<Option.Option<Array<ApprovalStep>>, ContextFlowError> =>
  Effect.option(Effect.gen(function*() {
    if (A.isEmptyArray(context.intents)) {
      return Option.none()
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
    )

    // const steps: Array<ApprovalStep> = []

    const steps = pipe(
      A.groupWith(
        requiredTokens,
        allowances,
        (reqired, allowed) => {
          return required
        },
      ),
    )
    // for (const { token, allowance } of allowances) {
    //   const requiredAmount = neededMap.get(token) ?? 0n
    //   if (allowance < requiredAmount) {
    //     steps.push(
    //       new ApprovalStep({
    //         token: ensureHex(token),
    //         requiredAmount,
    //         currentAllowance: allowance,
    //       }),
    //     )
    //   }
    // }

    // return steps.length > 0 ? Option.some(steps) : Option.none()
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
      Effect.mapError(() => new AllowanceCheckError({ message: "could not" })),
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
            token.address as unknown as any,
            sender.address as unknown as any,
            sourceChain.minter_address_display as unknown as any,
          ),
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
