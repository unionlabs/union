import type { Chain, Ucs03Channel, UserAddresses } from "$lib/types"
import type { FormFields } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import { fromHex } from "viem"
import { bech32ToBech32Address, getChannelInfo } from "@unionlabs/client"
import type { Intents, QuoteData } from "$lib/components/TransferFrom/transfer/types.ts"
import type { Balances } from "$lib/stores/balances.ts"
import type { TokenInfos } from "$lib/stores/tokens.ts"

export const createIntents = (
  rawIntents: FormFields,
  balances: Balances,
  userAddress: UserAddresses,
  chains: Array<Chain>,
  ucs03channels: Array<Ucs03Channel>,
  tokenInfos: TokenInfos,
  quoteToken: QuoteData | null
): Intents => {
  // Source Chain
  const sourceChain = chains.find(chain => chain.chain_id === rawIntents.source) ?? null

  // Destination Chain
  const destinationChain = chains.find(chain => chain.chain_id === rawIntents.destination) ?? null

  // Channel
  const channel =
    rawIntents.source && rawIntents.destination
      ? getChannelInfo(rawIntents.source, rawIntents.destination, ucs03channels)
      : null

  // Receiver
  const receiver = rawIntents.receiver

  // UCS03 Address
  const ucs03address =
    sourceChain && channel?.source_port_id
      ? sourceChain.rpc_type === "cosmos"
        ? fromHex(`0x${channel.source_port_id}`, "string")
        : `0x${channel.source_port_id}`
      : null

  const baseTokens = sourceChain
    ? sourceChain.tokens
        .map(token => {
          const balance = balances[rawIntents.source]?.[token.denom]
          return {
            denom: token.denom,
            balance
          }
        })
        .sort((a, b) => {
          if (!a?.balance) return 1
          if (!b?.balance) return -1
          if (a.balance.kind === "error") return 1
          if (b.balance.kind === "error") return -1

          if (a.balance.kind === "loading") return 1
          if (b.balance.kind === "loading") return -1

          if (a.balance.amount === null) return 1
          if (b.balance.amount === null) return -1

          const balanceA = BigInt(a.balance.amount)
          const balanceB = BigInt(b.balance.amount)
          return balanceB > balanceA ? 1 : balanceB < balanceA ? -1 : 0
        })
    : []

  const baseToken =
    rawIntents.asset && sourceChain
      ? (baseTokens.find(token => token.denom === rawIntents.asset) ?? null)
      : null

  const tokenInfo =
    sourceChain && baseToken?.denom ? tokenInfos[sourceChain.chain_id]?.[baseToken.denom] : null
  const baseTokenInfo = (tokenInfo?.kind === "tokenInfo" ? tokenInfo.info : null) ?? null

  if (!quoteToken) {
    console.log(`[QuoteToken] is null`)
  }

  const quoteTokenDenom =
    quoteToken && quoteToken.type === "NO_QUOTE_AVAILABLE"
      ? "NO_QUOTE_AVAILABLE"
      : (quoteToken?.quote_token ?? null)

  console.log(
    `[QuoteToken] quote for ${baseToken?.denom} from ${sourceChain?.chain_id} -> ${destinationChain?.chain_id}:`,
    quoteTokenDenom
  )

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
    baseTokens,
    baseToken,
    baseTokenInfo,
    channel,
    receiver,
    ucs03address,
    amount: rawIntents.amount,
    ownWallet,
    quoteToken: quoteTokenDenom
  }
}
