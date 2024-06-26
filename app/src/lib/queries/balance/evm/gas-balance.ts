import { raise } from "$lib/utilities"
import type { Balance } from "../types.ts"
import { config } from "$lib/wallet/evm/config.ts"
import { createPublicClient, getAddress, http } from "viem"

export async function evmGasBalance({
  address,
  chainId,
  url
}: { address: string; chainId: string; url?: string }): Promise<Balance> {
  console.info("evmGasBalance", { address, chainId })
  const validAddress = getAddress(address)
  const chain = config.chains.find(chain => String(chain.id) === chainId)
  if (!chain) raise(`chain with id ${chainId} not found`)
  const viemClient = createPublicClient({
    chain,
    // if no url is provided, use the first http rpc url as the default
    transport: http(
      url ? (URL.canParse(url) ? url : `https://${url}`) : chain.rpcUrls.default.http.at(0)
    )
  })
  const balance = await viemClient.getBalance({ address: validAddress })
  return {
    balance,
    gasToken: true,
    ...chain.nativeCurrency,
    address: chain.nativeCurrency.symbol
  }
}
