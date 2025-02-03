import { derived, type Readable } from "svelte/store"
import type { Chain, TokenInfoMulti } from "$lib/types"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { fromHex, isHex } from "viem"
import {
  bech32AddressToHex,
  bech32ToBech32Address,
  getChannelInfo,
  isValidBech32Address
} from "@unionlabs/client"
import { getTokenInfoSimple } from "$lib/components/TransferFrom/transfer/balances.ts"
import type { userBalancesQuery } from "$lib/queries/balance"

export type BaseToken = {
  denom: string
  balance: string
}

export interface IntentsStore {
  sourceChain: Chain | null
  destinationChain: Chain | null
  baseTokens: Array<BaseToken>
  baseToken: BaseToken | null
  baseTokenInfo: TokenInfoMulti | null
  channel: ReturnType<typeof getChannelInfo> | null
  receiver: string
  ucs03address: string | null
  amount: string
  ownWallet: string | null
}

export function createIntentStore(
  rawIntents: RawIntentsStore,
  context: Readable<ContextStore>,
  balances: ReturnType<typeof userBalancesQuery>
): Readable<IntentsStore> {
  const sourceChain = derived([rawIntents, context], ([$rawIntents, $context]) => {
    return $context.chains.find(chain => chain.chain_id === $rawIntents.source) ?? null
  })

  const destinationChain = derived(
    [rawIntents, context],
    ([$intents, $context]) =>
      $context.chains.find(chain => chain.chain_id === $intents.destination) ?? null
  )

  let channel = derived([rawIntents, context], ([$rawIntents, $context]) => {
    if (!($rawIntents.source && $rawIntents.destination)) return null
    return getChannelInfo($rawIntents.source, $rawIntents.destination, $context.ucs03channels)
  })

  const receiver = derived([rawIntents, destinationChain], ([$rawIntents, $destinationChain]) => {
    if (!($destinationChain && $rawIntents.receiver)) return $rawIntents.receiver

    return $destinationChain.rpc_type === "cosmos" && isValidBech32Address($rawIntents.receiver)
      ? bech32AddressToHex({ address: $rawIntents.receiver })
      : $rawIntents.receiver
  })

  const ucs03address = derived([sourceChain, channel], ([$sourceChain, $channel]) => {
    if (!($sourceChain && $channel?.source_port_id)) return null

    return $sourceChain.rpc_type === "cosmos"
      ? fromHex(`0x${$channel.source_port_id}`, "string")
      : `0x${$channel.source_port_id}`
  })

  const baseTokens = derived([balances, sourceChain], ([$balances, $sourceChain]) => {
    if (!$sourceChain) return []
    let balances = $balances.find(c => c.data?.chain_id === $sourceChain.chain_id)
    return $sourceChain.tokens.map(token => ({
      denom: token.denom,
      balance: balances?.data?.balances[token.denom] ?? "0"
    }))
  })

  const baseToken = derived(
    [rawIntents, baseTokens, sourceChain],
    ([$rawIntents, $baseTokens, $sourceChain]) => {
      if (!($rawIntents.asset && $sourceChain)) return null

      const denom =
        $sourceChain.rpc_type === "cosmos" && isHex($rawIntents.asset)
          ? fromHex($rawIntents.asset, "string")
          : $rawIntents.asset

      return $baseTokens.find(token => token.denom === denom) ?? null
    }
  )

  const baseTokenInfo = derived(
    [baseToken, sourceChain, context],
    ([$baseToken, $sourceChain, $context], set: (value: TokenInfoMulti | null) => void) => {
      if ($baseToken === null || $sourceChain === null) {
        set(null)
        return
      }

      getTokenInfoSimple($sourceChain.chain_id, $baseToken.denom, $context.chains)
        .then(tokenInfo => {
          set(tokenInfo)
        })
        .catch(error => {
          console.error("Error fetching token info:", error)
          set(null)
        })
    },
    null
  )

  const ownWallet = derived([destinationChain, context], ([$destinationChain, $context]) => {
    if (!$destinationChain) return null
    const userAddress = $context.userAddress

    switch ($destinationChain.rpc_type) {
      case "evm": {
        if (!userAddress.evm) return null
        return userAddress.evm.canonical
      }
      case "cosmos": {
        if (!userAddress.cosmos) return null
        return bech32ToBech32Address({
          address: userAddress.cosmos.canonical,
          toPrefix: $destinationChain.addr_prefix
        })
      }
      case "aptos": {
        return userAddress.aptos?.canonical ?? null
      }
      default:
        return null
    }
  })

  return derived(
    [
      rawIntents,
      sourceChain,
      destinationChain,
      channel,
      baseTokens,
      baseToken,
      baseTokenInfo,
      receiver,
      ucs03address,
      ownWallet
    ],
    ([
      $rawIntents,
      $sourceChain,
      $destinationChain,
      $channel,
      $baseTokens,
      $baseToken,
      $baseTokenInfo,
      $receiver,
      $ucs03address,
      $ownWallet
    ]) => ({
      sourceChain: $sourceChain,
      destinationChain: $destinationChain,
      baseTokens: $baseTokens,
      baseToken: $baseToken,
      baseTokenInfo: $baseTokenInfo,
      channel: $channel,
      receiver: $receiver,
      ucs03address: $ucs03address,
      amount: $rawIntents.amount,
      ownWallet: $ownWallet
    })
  )
}
