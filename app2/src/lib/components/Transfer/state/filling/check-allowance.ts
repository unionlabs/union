import { Data, Effect, Match, Option } from "effect"
import type { AddressCanonicalBytes, Chain } from "@unionlabs/sdk/schema"
import { fromHex, http, isHex } from "viem"
import { createViemPublicClient, readErc20Allowance, ViemPublicClient } from "@unionlabs/sdk/evm"
import { CosmWasmClientSource, createCosmWasmClient } from "@unionlabs/sdk/cosmos"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import { cosmosSpenderAddresses } from "$lib/constants/spender-addresses.ts"
import {
  AllowanceCheckError,
  CosmosQueryError,
  type TransferFlowError
} from "$lib/components/Transfer/state/errors.ts"
import type { TransferIntent } from "$lib/components/Transfer/state/filling/create-intents.ts"

export class ApprovalStep extends Data.TaggedClass("ApprovalStep")<{
  token: string
  requiredAmount: bigint
  currentAllowance: bigint
}> {}

function gatherNeededAmounts(contexts: Array<{ baseToken: string; baseAmount: bigint }>) {
  const map = new Map<string, bigint>()
  for (const { baseToken, baseAmount } of contexts) {
    const current = map.get(baseToken) ?? 0n
    map.set(baseToken, current + baseAmount)
  }
  return map
}

export function checkAllowances(
  intent: TransferIntent
): Effect.Effect<Option.Option<Array<ApprovalStep>>, TransferFlowError> {
  return Effect.gen(function* () {
    if (intent.contexts.length === 0) return Option.none()

    const [firstContext] = intent.contexts
    const chain = firstContext.sourceChain
    const sender = firstContext.sender
    const spender = firstContext.ucs03address

    const neededMap = gatherNeededAmounts(
      intent.contexts.map(({ baseToken, baseAmount }) => ({ baseToken, baseAmount }))
    )
    const tokenAddresses = [...neededMap.keys()]

    const allowancesOpt = yield* Match.value(chain.rpc_type).pipe(
      Match.when("evm", () =>
        handleEvmAllowances(tokenAddresses, sender, spender, chain).pipe(
          Effect.mapError(err => new AllowanceCheckError({ cause: err }))
        )
      ),
      Match.when("cosmos", () =>
        handleCosmosAllowances(tokenAddresses, sender, chain).pipe(
          Effect.mapError(err => new AllowanceCheckError({ cause: err }))
        )
      ),
      Match.orElse(() => Effect.succeed(Option.none()))
    )

    const allowances = Option.getOrElse(allowancesOpt, () => [])
    const steps: Array<ApprovalStep> = []

    for (const { token, allowance } of allowances) {
      const requiredAmount = neededMap.get(token) ?? 0n
      if (allowance < requiredAmount) {
        steps.push(
          new ApprovalStep({
            token,
            requiredAmount,
            currentAllowance: allowance
          })
        )
      }
    }

    return steps.length > 0 ? Option.some(steps) : Option.none()
  })
}

function handleEvmAllowances(
  tokenAddresses: Array<string>,
  sender: AddressCanonicalBytes,
  spender: string,
  sourceChain: Chain
): Effect.Effect<Option.Option<Array<{ token: string; allowance: bigint }>>, unknown> {
  return Effect.gen(function* () {
    const viemChainOpt = sourceChain.toViemChain()
    if (Option.isNone(viemChainOpt)) return Option.none()

    const publicClientSource = yield* createViemPublicClient({
      chain: viemChainOpt.value,
      transport: http()
    })

    const results = yield* Effect.all(
      tokenAddresses.map(tokenAddress =>
        Effect.gen(function* () {
          const allowance = yield* readErc20Allowance(tokenAddress, sender, spender)
          return { token: tokenAddress, allowance }
        }).pipe(Effect.provideService(ViemPublicClient, { client: publicClientSource }))
      )
    )

    return Option.some(results)
  })
}

export function handleCosmosAllowances(
  tokenAddresses: Array<string>,
  sender: AddressCanonicalBytes,
  sourceChain: Chain
): Effect.Effect<Option.Option<Array<{ token: string; allowance: bigint }>>, CosmosQueryError> {
  return Effect.gen(function* () {
    const rpcUrlOpt = sourceChain.getRpcUrl("rpc")
    if (Option.isNone(rpcUrlOpt) || !sourceChain.toCosmosDisplay) {
      return yield* Effect.fail(
        new CosmosQueryError({
          token: "N/A",
          cause: "Missing RPC URL or missing display converter"
        })
      )
    }

    const rpcUrl = rpcUrlOpt.value
    const cosmwasmClient = yield* createCosmWasmClient(rpcUrl).pipe(
      Effect.mapError(err => new CosmosQueryError({ token: "N/A", cause: err }))
    )

    const isNativeToken = (token: string): boolean => /^u[a-zA-Z]+$/.test(token)

    function isContractToken(token: string) {
      return Effect.gen(function* () {
        if (isValidBech32ContractAddress(token)) return true
        if (!isHex(token)) return false

        const decoded = yield* Effect.try(() => fromHex(token, "string")).pipe(
          Effect.mapError(
            err => new CosmosQueryError({ token, cause: `Hex decoding failed: ${err}` })
          )
        )
        return isValidBech32ContractAddress(decoded)
      })
    }

    const contractTokenCandidates = tokenAddresses.filter(token => !isNativeToken(token))
    const contractTokens = yield* Effect.filter(contractTokenCandidates, isContractToken)

    if (contractTokens.length === 0) {
      return Option.some([])
    }

    const checks = yield* Effect.all(
      contractTokens.map(tokenAddress =>
        Effect.gen(function* () {
          const owner = yield* sourceChain
            .toCosmosDisplay(sender)
            .pipe(Effect.mapError(err => new CosmosQueryError({ token: tokenAddress, cause: err })))

          const spender = cosmosSpenderAddresses[sourceChain.universal_chain_id]
          let bech32Address: string | null = null

          if (!isHex(tokenAddress) && isValidBech32ContractAddress(tokenAddress)) {
            bech32Address = tokenAddress
          } else if (isHex(tokenAddress)) {
            const decoded = yield* Effect.try(() => fromHex(tokenAddress, "string")).pipe(
              Effect.mapError(err => new CosmosQueryError({ token: tokenAddress, cause: err }))
            )
            if (isValidBech32ContractAddress(decoded)) {
              bech32Address = decoded
            }
          }

          if (!bech32Address) return { token: tokenAddress, allowance: 0n }

          const result = yield* Effect.tryPromise(() =>
            cosmwasmClient.queryContractSmart(bech32Address, {
              allowance: { owner, spender }
            })
          ).pipe(Effect.mapError(err => new CosmosQueryError({ token: tokenAddress, cause: err })))

          const allowance = result.allowance ? BigInt(result.allowance) : 0n
          return { token: tokenAddress, allowance }
        }).pipe(Effect.provideService(CosmWasmClientSource, { client: cosmwasmClient }))
      )
    )

    return Option.some(checks)
  })
}
