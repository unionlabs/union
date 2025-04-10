import { Effect, identity, Match, Option, Schema } from "effect"
import { AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix, type Chain } from "@unionlabs/sdk/schema"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import type { TransferIntents } from "$lib/components/Transfer/transfer.svelte.ts"
import { isHex, toHex } from "viem"
import { ensureHex } from "@unionlabs/sdk/utils"

export const checkBalanceForIntents = (source: Chain, intents: TransferIntents) => {
  const grouped = intents.reduce(
    (acc, intent) => {
      const key = `${intent.sender}_${intent.baseToken}`
      if (acc[key]) {
        acc[key].required += intent.baseAmount
      } else {
        acc[key] = {
          sender: Match.value({
            sender: intent.sender,
            source: source.rpc_type,
            addr_prefix: source.addr_prefix,
          }).pipe(
            Match.when(
              {
                sender: Match.string,
                source: "evm",
              },
              ({ sender }) => sender
            ),
            Match.when(
              {
                sender: Match.string,
                addr_prefix: Match.string,
                source: "cosmos",
              },
              ({ addr_prefix, sender }) => Schema.encodeSync(
                AddressCosmosZkgmFromAddressCanonicalBytesWithPrefix(addr_prefix)
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              )(sender as unknown as any)
            ),
            Match.orElseAbsurd,
          ),
          baseToken: intent.baseToken,
          required: intent.baseAmount
        }
      }
      return acc
    },
    {} as Record<string, { sender: string; baseToken: string; required: bigint }>
  )

  const groupedValues = Object.values(grouped)

  return Effect.forEach(groupedValues, group =>
    Effect.flatMap(
      Effect.sync(() =>
        balancesStore.getBalance(
          source.universal_chain_id,
          // is zkgm
          group.sender,
          ensureHex(group.baseToken)
        )
      ),
      balance => {
        if (!Option.isSome(balance)) {
          console.log("[CTS] No balance found for grouping:", group)
          return Effect.succeed(false)
        }
        return Effect.try({
          try: () => {
            const balanceBigInt = BigInt(balance.value)
            const enough = group.required <= balanceBigInt
            console.log("[CTS] Balance check for grouping:", {
              group,
              balance: balanceBigInt.toString(),
              enough
            })
            return enough
          },
          catch: error => {
            console.error("[CTS] Error converting balance to BigInt for grouping:", group, error)
            return false
          }
        })
      }
    )
  ).pipe(Effect.map(results => results.every(identity)))
}
