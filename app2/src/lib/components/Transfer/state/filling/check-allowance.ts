import { Effect, Option, Match } from "effect"
import type { AddressCanonicalBytes, Chain } from "@unionlabs/sdk/schema"
import { isHex, fromHex, http } from "viem"
import { createViemPublicClient, readErc20Allowance, ViemPublicClient } from "@unionlabs/sdk/evm"
import { createCosmWasmClient, CosmWasmClientSource } from "@unionlabs/sdk/cosmos"
import { isValidBech32ContractAddress } from "@unionlabs/client"
import type { TransferIntent } from "$lib/components/Transfer/transfer.svelte.ts"

/**
 * Represents a single approval step indicating that a specific token
 * needs an approval for the requiredAmount if currentAllowance < requiredAmount.
 */
type ApprovalStep = {
  token: string
  requiredAmount: bigint
  currentAllowance: bigint
}

/**
 * Collect total needed amounts for each token from an array of TransferIntents.
 */
function gatherNeededAmounts(intents: Array<TransferIntent>): Map<string, bigint> {
  console.log("lukas: gatherNeededAmounts called with", intents)
  const neededMap = new Map<string, bigint>()
  for (const { baseToken, baseAmount } of intents) {
    const current = neededMap.get(baseToken) ?? 0n
    neededMap.set(baseToken, current + baseAmount)
  }
  console.log("lukas: gatherNeededAmounts result:", neededMap)
  return neededMap
}

/**
 * checkAllowances:
 *
 * 1) Aggregates the user's needed amounts per token (from TransferIntent).
 * 2) Dispatches to EVM or Cosmos logic to read actual allowances.
 * 3) Builds a list of ApprovalStep objects for any token where allowance < needed.
 * 4) Returns Option.none() if no approvals are needed, or Option.some([...]) otherwise.
 */
export function checkAllowances(
  chain: Chain,
  intents: Array<TransferIntent>,
  sender: AddressCanonicalBytes,
  spenderAddress: string
) {
  return Effect.gen(function* () {
    console.log("lukas: checkAllowances called with", {
      chain: chain.display_name,
      rpc_type: chain.rpc_type,
      sender,
      spenderAddress,
      intents
    })

    // 1) Summarize amounts needed for each token
    const neededMap = gatherNeededAmounts(intents)
    const tokenAddresses = [...neededMap.keys()]

    // 2) Based on chain type, do EVM or Cosmos
    const allowancesOpt = yield* Match.value(chain.rpc_type).pipe(
      Match.when("evm", () => handleEvmAllowances(tokenAddresses, sender, spenderAddress, chain)),
      Match.when("cosmos", () =>
        handleCosmosAllowances(tokenAddresses, sender, spenderAddress, chain)
      ),
      Match.orElse(() => {
        console.log("lukas: Unsupported chain type for allowances:", chain.rpc_type)
        return Effect.succeed(Option.none<Array<{ token: string; allowance: bigint }>>())
      })
    )

    if (Option.isNone(allowancesOpt)) {
      console.log("lukas: No allowances returned (Option.none)")
      return Option.none<Array<ApprovalStep>>()
    }

    const allowances = allowancesOpt.value
    console.log("lukas: Fetched allowances:", allowances)

    // 3) Compare each token’s needed amount with fetched allowance
    const steps: Array<ApprovalStep> = []

    for (const { token, allowance } of allowances) {
      const requiredAmount = neededMap.get(token) ?? 0n
      if (allowance < requiredAmount) {
        console.log("lukas: Token requires approval:", {
          token,
          requiredAmount,
          currentAllowance: allowance
        })
        steps.push({
          token,
          requiredAmount,
          currentAllowance: allowance
        })
      }
    }

    if (steps.length === 0) {
      console.log("lukas: No approval steps needed – returning Option.none")
      return Option.none<Array<ApprovalStep>>()
    }

    console.log("lukas: Returning approval steps:", steps)
    return Option.some(steps)
  })
}

/**
 * handleEvmAllowances:
 *  For each ERC20 token, read allowance in parallel.
 *  Returns Option.some([{ token, allowance }, ...]) or Option.none.
 */
function handleEvmAllowances(
  tokenAddresses: Array<string>,
  sender: AddressCanonicalBytes,
  spender: string,
  sourceChain: Chain
) {
  return Effect.gen(function* () {
    console.log("lukas: handleEvmAllowances called with", {
      tokenAddresses,
      sender,
      spender,
      chain: sourceChain.display_name
    })

    const viemChainOpt = sourceChain.toViemChain()
    if (Option.isNone(viemChainOpt)) {
      console.log("lukas: Could not get viemChain for EVM chain:", sourceChain.display_name)
      return Option.none<Array<{ token: string; allowance: bigint }>>()
    }

    const publicClientSource = yield* createViemPublicClient({
      chain: viemChainOpt.value,
      transport: http()
    })

    // Parallel readErc20Allowance calls
    const results = yield* Effect.all(
      tokenAddresses.map(tokenAddress =>
        Effect.gen(function* () {
          const allowance = yield* readErc20Allowance(tokenAddress, sender, spender)
          return { token: tokenAddress, allowance }
        }).pipe(
          Effect.provideService(ViemPublicClient, {
            client: publicClientSource
          })
        )
      )
    )

    console.log("lukas: handleEvmAllowances final results:", results)
    return Option.some(results)
  })
}

/**
 * handleCosmosAllowances:
 *  If token is not hex => native => no approval needed => 0n allowance
 *  If token is hex => treat as CW20 => query allowance
 */
function handleCosmosAllowances(
  tokenAddresses: Array<string>,
  sender: AddressCanonicalBytes,
  spender: string,
  sourceChain: Chain
) {
  return Effect.gen(function* () {
    console.log("lukas: handleCosmosAllowances called with", {
      tokenAddresses,
      sender,
      spender,
      chain: sourceChain.display_name
    })

    const rpcUrlOpt = sourceChain.getRpcUrl("rpc")
    if (Option.isNone(rpcUrlOpt) || !sourceChain.toCosmosDisplay) {
      console.log("lukas: Missing rpcUrl or toCosmosDisplay on chain:", sourceChain.display_name)
      return Option.none<Array<{ token: string; allowance: bigint }>>()
    }

    const rpcUrl = rpcUrlOpt.value
    const cosmwasmClient = yield* createCosmWasmClient(rpcUrl)

    // Function to identify native tokens (tokens that start with 'u' followed by letters)
    const isNativeToken = (token: string): boolean => {
      return /^u[a-zA-Z]+$/.test(token)
    }

    // Function to check if a token is a contract (either Bech32 or hex-encoded)
    const isContractToken = (token: string): boolean => {
      // Direct Bech32 contract check
      if (isValidBech32ContractAddress(token)) {
        return true
      }

      // Hex-encoded contract check
      if (isHex(token)) {
        try {
          const decoded = fromHex(token, "string")
          return isValidBech32ContractAddress(decoded)
        } catch {
          return false
        }
      }

      return false
    }

    // Filter to separate native tokens and contract tokens
    const nativeTokens = tokenAddresses.filter(isNativeToken)
    const contractTokenCandidates = tokenAddresses.filter(token => !isNativeToken(token))

    // Further check which of the non-native tokens are valid contracts
    const contractTokens = contractTokenCandidates.filter(isContractToken)

    console.log("lukas: Identified native tokens:", nativeTokens)
    console.log("lukas: Identified contract tokens:", contractTokens)

    // If all tokens are native, return empty array (no approvals needed)
    if (contractTokens.length === 0) {
      console.log("lukas: No contract tokens to check allowances for")
      return Option.some([])
    }

    // Process contract tokens
    const checks = yield* Effect.all(
      contractTokens.map(tokenAddress =>
        Effect.gen(function* () {
          console.log("lukas: Checking contract token:", tokenAddress)

          // For direct Bech32 addresses
          if (!isHex(tokenAddress) && isValidBech32ContractAddress(tokenAddress)) {
            console.log("lukas: Processing direct Bech32 address:", tokenAddress)
            const owner = yield* sourceChain.toCosmosDisplay(sender)
            const result = yield* Effect.tryPromise({
              try: () => {
                console.log("zkgm", { allowance: { owner, spender } })
                return cosmwasmClient.queryContractSmart(tokenAddress, {
                  allowance: {
                    owner,
                    spender: "bbn1dy20pwy30hfqyxdzrmp33h47h4xdxht6phqecfp2jdnes6su9pysqq2kpw"
                  }
                })
              },
              catch: e => {
                console.log("lukas: Error in queryContractSmart for direct address:", e)
                return e
              }
            })

            const allowance = result?.allowance ? BigInt(result.allowance) : 0n
            console.log("lukas: Query result allowance for direct address:", allowance)
            return { token: tokenAddress, allowance }
          }

          // For hex-encoded addresses
          if (isHex(tokenAddress)) {
            const decoded = fromHex(tokenAddress, "string")

            if (!isValidBech32ContractAddress(decoded)) {
              console.log("lukas: Not valid bech32 contract address => skipping:", decoded)
              return { token: tokenAddress, allowance: 0n }
            }

            const owner = yield* sourceChain.toCosmosDisplay(sender)
            const result = yield* Effect.tryPromise({
              try: () =>
                cosmwasmClient.queryContractSmart(decoded, {
                  allowance: { owner, spender }
                }),
              catch: e => {
                console.log("lukas: Error in queryContractSmart:", e)
                return e
              }
            })

            const allowance = result?.allowance ? BigInt(result.allowance) : 0n
            console.log("lukas: Query result allowance:", allowance)
            return { token: tokenAddress, allowance }
          }

          // This should not happen given our filtering, but just in case
          return { token: tokenAddress, allowance: 0n }
        }).pipe(Effect.provideService(CosmWasmClientSource, { client: cosmwasmClient }))
      )
    )

    console.log("lukas: handleCosmosAllowances final checks:", checks)
    return Option.some(checks)
  })
}
