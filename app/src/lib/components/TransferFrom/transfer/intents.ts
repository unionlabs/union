import type {Chain, Ucs03Channel, UserAddresses} from "$lib/types"
import type {FormFields} from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import {fromHex} from "viem"
import {
  bech32AddressToHex,
  bech32ToBech32Address,
  getChannelInfo,
  isValidBech32Address
} from "@unionlabs/client"
import type {ChainBalance, Intents} from "$lib/components/TransferFrom/transfer/types.ts";

export const createIntents = (
  rawIntents: FormFields,
  chains: Array<Chain>,
  balances: ChainBalance[],
  ucs03channels: Array<Ucs03Channel>,
  userAddress: UserAddresses
): Intents => {
  // Source Chain
  const sourceChain = chains.find(chain => chain.chain_id === rawIntents.source) ?? null

  // Destination Chain
  const destinationChain = chains.find(chain => chain.chain_id === rawIntents.destination) ?? null

  // Channel
  const channel = rawIntents.source && rawIntents.destination
    ? getChannelInfo(rawIntents.source, rawIntents.destination, ucs03channels)
    : null

  // Receiver
  const receiver = destinationChain && rawIntents.receiver
    ? (destinationChain.rpc_type === "cosmos" && isValidBech32Address(rawIntents.receiver)
      ? bech32AddressToHex({address: rawIntents.receiver})
      : rawIntents.receiver)
    : rawIntents.receiver

  // UCS03 Address
  const ucs03address = sourceChain && channel?.source_port_id
    ? (sourceChain.rpc_type === "cosmos"
      ? fromHex(`0x${channel.source_port_id}`, "string")
      : `0x${channel.source_port_id}`)
    : null

  // Get base tokens directly from balances and chain data
  const getBaseTokens = () => {
    if (!sourceChain) return []

    const chainBalances = balances.find(b => b.data?.chain_id === rawIntents.source)?.data
    return sourceChain.tokens.map(token => ({
      denom: token.denom,
      balance: chainBalances?.balances[token.denom] ?? "0"
    }))
  }

  // Base Token
  const baseTokens = getBaseTokens()
  const baseToken = rawIntents.asset && sourceChain
    ? baseTokens.find(token => token.denom === rawIntents.asset) ?? null
    : null

  // Own Wallet
  const ownWallet = (() => {
    if (!destinationChain) return null

    switch (destinationChain.rpc_type) {
      case "evm": {
        if (!userAddress.evm) return null
        return userAddress.evm.canonical
      }
      case "cosmos": {
        if (!userAddress.cosmos) return null
        return bech32ToBech32Address({
          address: userAddress.cosmos.canonical,
          toPrefix: destinationChain.addr_prefix
        })
      }
      case "aptos": {
        return userAddress.aptos?.canonical ?? null
      }
      default:
        return null
    }
  })()

  return {
    sourceChain,
    destinationChain,
    baseToken,
    baseTokens,
    channel,
    receiver,
    ucs03address,
    amount: rawIntents.amount,
    ownWallet
  }
}