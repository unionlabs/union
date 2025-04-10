import { Effect, Option, Match, Data } from "effect"
import type { AddressCanonicalBytes, Chain } from "@unionlabs/sdk/schema"
import { isHex, fromHex, http } from "viem"
import { createViemPublicClient, readErc20Allowance, ViemPublicClient } from "@unionlabs/sdk/evm"
import { createCosmWasmClient, CosmWasmClientSource } from "@unionlabs/sdk/cosmos"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import type { TransferIntent } from "$lib/components/Transfer/transfer.svelte.ts"
import { cosmosSpenderAddresses } from "$lib/constants/spender-addresses.ts"
import {
  AllowanceCheckError,
  CosmosQueryError,
  type TransferFlowError
} from "$lib/components/Transfer/state/errors.ts"

export class ApprovalStep extends Data.TaggedClass("ApprovalStep")<{
  token: string
  requiredAmount: bigint
  currentAllowance: bigint
}> {}

function gatherNeededAmounts(intents: Array<TransferIntent>): Map<string, bigint> {
  const neededMap = new Map<string, bigint>()
  for (const { baseToken, baseAmount } of intents) {
    const current = neededMap.get(baseToken) ?? 0n
    neededMap.set(baseToken, current + baseAmount)
  }
  return neededMap
}

export function checkAllowances(
  chain: Chain,
  intents: Array<TransferIntent>,
  sender: AddressCanonicalBytes,
  spenderAddress: string
): Effect.Effect<Option.Option<Array<ApprovalStep>>, TransferFlowError> {
  return Effect.gen(function* () {
    if (intents.length === 0) {
      return Option.none()
    }

    const neededMap = gatherNeededAmounts(intents)
    const tokenAddresses = [...neededMap.keys()]

    const allowancesOpt = yield* Match.value(chain.rpc_type).pipe(
      Match.when("evm", () =>
        handleEvmAllowances(tokenAddresses, sender, spenderAddress, chain).pipe(
          Effect.mapError((err) => new AllowanceCheckError({ details: err }))
        )
      ),
      Match.when("cosmos", () =>
        handleCosmosAllowances(tokenAddresses, sender, chain).pipe(
          Effect.mapError((err) => new AllowanceCheckError({ details: err }))
        )
      ),
      Match.orElse(() => Effect.succeed(Option.none()))
    )

    const allowances = Option.getOrElse(allowancesOpt, () => [])

    const steps: Array<ApprovalStep> = []

    for (const { token, allowance } of allowances) {
      const requiredAmount = neededMap.get(token) ?? 0n
      if (allowance < requiredAmount) {
        steps.push(new ApprovalStep({ token, requiredAmount, currentAllowance: allowance }))
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
    if (Option.isNone(viemChainOpt)) {
      return Option.none()
    }

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

function handleCosmosAllowances(
  tokenAddresses: Array<string>,
  sender: AddressCanonicalBytes,
  sourceChain: Chain
): Effect.Effect<Option.Option<Array<{ token: string; allowance: bigint }>>, CosmosQueryError> {
  return Effect.gen(function* () {
    const rpcUrlOpt = sourceChain.getRpcUrl("rpc")
    if (Option.isNone(rpcUrlOpt) || !sourceChain.toCosmosDisplay) {
      return Option.none()
    }

    const rpcUrl = rpcUrlOpt.value
    const cosmwasmClient = yield* createCosmWasmClient(rpcUrl)

    const isNativeToken = (token: string): boolean => /^u[a-zA-Z]+$/.test(token)

    function isContractToken(token: string) {
      return Effect.gen(function* () {
        if (isValidBech32ContractAddress(token)) return true
        if (!isHex(token)) return false

        const decoded = yield* Effect.try(() => fromHex(token, "string"))
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
          const owner = yield* sourceChain.toCosmosDisplay(sender)
          const spender = cosmosSpenderAddresses[sourceChain.universal_chain_id]

          let bech32Address: string | null = null

          if (!isHex(tokenAddress) && isValidBech32ContractAddress(tokenAddress)) {
            bech32Address = tokenAddress
          } else if (isHex(tokenAddress)) {
            const decoded = fromHex(tokenAddress, "string")
            if (isValidBech32ContractAddress(decoded)) {
              bech32Address = decoded
            }
          }

          if (!bech32Address) {
            return { token: tokenAddress, allowance: 0n }
          }

          const result = yield* Effect.tryPromise(() =>
            cosmwasmClient.queryContractSmart(bech32Address!, {
              allowance: { owner, spender }
            })
          ).pipe(
            Effect.mapError((err) =>
              new CosmosQueryError({ token: tokenAddress, details: err })
            )
          )

          const allowance = result.allowance ? BigInt(result.allowance) : 0n
          return { token: tokenAddress, allowance }
        }).pipe(Effect.provideService(CosmWasmClientSource, { client: cosmwasmClient }))
      )
    )

    return Option.some(checks)
  })
}
