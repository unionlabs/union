import { Effect, Option, Match } from "effect";
import { type AddressCanonicalBytes, Chain } from "@unionlabs/sdk/schema";
import { isHex, fromHex, http } from "viem";
import {
  createViemPublicClient,
  readErc20Allowance,
  ViemPublicClient
} from "@unionlabs/sdk/evm";
import {
  createCosmWasmClient,
  CosmWasmClientSource
} from "@unionlabs/sdk/cosmos";
import { isValidBech32ContractAddress } from "@unionlabs/client";
import type { TransferIntent } from "$lib/components/Transfer/transfer.svelte.ts";

/**
 * Represents a single approval step indicating that a specific token
 * needs an approval for the requiredAmount if currentAllowance < requiredAmount.
 */
type ApprovalStep = {
  token: string;
  requiredAmount: bigint;
  currentAllowance: bigint;
};

/**
 * Collect total needed amounts for each token from an array of TransferIntents.
 */
function gatherNeededAmounts(intents: TransferIntent[]): Map<string, bigint> {
  console.log("lukas: gatherNeededAmounts called with", intents);
  const neededMap = new Map<string, bigint>();
  for (const { baseToken, baseAmount } of intents) {
    const current = neededMap.get(baseToken) ?? 0n;
    neededMap.set(baseToken, current + baseAmount);
  }
  console.log("lukas: gatherNeededAmounts result:", neededMap);
  return neededMap;
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
  intents: TransferIntent[],
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
    });

    // 1) Summarize amounts needed for each token
    const neededMap = gatherNeededAmounts(intents);
    const tokenAddresses = [...neededMap.keys()];

    // 2) Based on chain type, do EVM or Cosmos
    const allowancesOpt = yield* Match.value(chain.rpc_type).pipe(
      Match.when("evm", () =>
        handleEvmAllowances(tokenAddresses, sender, spenderAddress, chain)
      ),
      Match.when("cosmos", () =>
        handleCosmosAllowances(tokenAddresses, sender, spenderAddress, chain)
      ),
      Match.orElse(() => {
        console.log("lukas: Unsupported chain type for allowances:", chain.rpc_type);
        return Effect.succeed(Option.none<Array<{ token: string; allowance: bigint }>>());
      })
    );

    if (Option.isNone(allowancesOpt)) {
      console.log("lukas: No allowances returned (Option.none)");
      return Option.none<ApprovalStep[]>();
    }

    const allowances = allowancesOpt.value;
    console.log("lukas: Fetched allowances:", allowances);

    // 3) Compare each token’s needed amount with fetched allowance
    const steps: ApprovalStep[] = [];

    for (const { token, allowance } of allowances) {
      const requiredAmount = neededMap.get(token) ?? 0n;
      if (allowance < requiredAmount) {
        console.log("lukas: Token requires approval:", {
          token,
          requiredAmount,
          currentAllowance: allowance
        });
        steps.push({
          token,
          requiredAmount,
          currentAllowance: allowance
        });
      }
    }

    // 4) Return Option.some(...) if steps needed, else Option.none()
    if (steps.length === 0) {
      console.log("lukas: No approval steps needed – returning Option.none");
      return Option.none<ApprovalStep[]>();
    }

    console.log("lukas: Returning approval steps:", steps);
    return Option.some(steps);
  });
}

/**
 * handleEvmAllowances:
 *  For each ERC20 token, read allowance in parallel.
 *  Returns Option.some([{ token, allowance }, ...]) or Option.none.
 */
function handleEvmAllowances(
  tokenAddresses: string[],
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
    });

    const viemChainOpt = sourceChain.toViemChain();
    if (Option.isNone(viemChainOpt)) {
      console.log("lukas: Could not get viemChain for EVM chain:", sourceChain.display_name);
      return Option.none<Array<{ token: string; allowance: bigint }>>();
    }

    const publicClientSource = yield* createViemPublicClient({
      chain: viemChainOpt.value,
      transport: http()
    });

    // Parallel readErc20Allowance calls
    const results = yield* Effect.all(
      tokenAddresses.map(tokenAddress =>
        Effect.gen(function* () {
          console.log("lukas: Checking allowance for EVM token:", tokenAddress);
          const allowance = yield* readErc20Allowance(tokenAddress, sender, spender);
          console.log("lukas: readErc20Allowance result", { tokenAddress, allowance });
          return { token: tokenAddress, allowance };
        }).pipe(
          Effect.provideService(ViemPublicClient, {
            client: publicClientSource
          })
        )
      )
    );

    console.log("lukas: handleEvmAllowances final results:", results);
    return Option.some(results);
  });
}

/**
 * handleCosmosAllowances:
 *  If token is not hex => native => no approval needed => 0n allowance
 *  If token is hex => treat as CW20 => query allowance
 */
function handleCosmosAllowances(
  tokenAddresses: string[],
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
    });

    const rpcUrlOpt = sourceChain.getRpcUrl("rpc");
    if (Option.isNone(rpcUrlOpt) || !sourceChain.toCosmosDisplay) {
      console.log("lukas: Missing rpcUrl or toCosmosDisplay on chain:", sourceChain.display_name);
      return Option.none<Array<{ token: string; allowance: bigint }>>();
    }

    const rpcUrl = rpcUrlOpt.value;
    console.log("lukas: Creating CosmWasm client with rpcUrl:", rpcUrl);

    const cosmwasmClient = yield* createCosmWasmClient(rpcUrl);

    // For each token, either skip or query
    const checks = yield* Effect.all(
      tokenAddresses.map(tokenAddress =>
        Effect.gen(function* () {
          console.log("lukas: Checking token (Cosmos):", tokenAddress);

          // If not hex => native => set allowance=0n to reflect "no approval needed"
          if (!isHex(tokenAddress)) {
            console.log("lukas: Token is native denom (not hex) => skip approval:", tokenAddress);
            return { token: tokenAddress, allowance: 0n };
          }

          // If hex => we treat it as a CW20
          const decoded = fromHex(tokenAddress, "string");
          if (!isValidBech32ContractAddress(decoded)) {
            console.log("lukas: Not valid bech32 contract address => skip:", decoded);
            return { token: tokenAddress, allowance: 0n };
          }

          const owner = yield* sourceChain.toCosmosDisplay(sender);
          console.log("lukas: Querying CW20 allowance for:", {
            tokenAddress,
            owner,
            spender
          });

          const result = yield* Effect.tryPromise({
            try: () =>
              cosmwasmClient.queryContractSmart(decoded, {
                allowance: { owner, spender }
              }),
            catch: e => {
              console.log("lukas: Error in queryContractSmart:", e);
              return e;
            }
          });

          const allowance = result?.allowance ? BigInt(result.allowance) : 0n;
          console.log("lukas: Query result allowance:", allowance);

          return { token: tokenAddress, allowance };
        }).pipe(
          Effect.provideService(CosmWasmClientSource, { client: cosmwasmClient })
        )
      )
    );

    console.log("lukas: handleCosmosAllowances final checks:", checks);
    return Option.some(checks);
  });
}
