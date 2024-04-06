import { isKeyOf, raise } from "#/lib/utilities.ts"
import versions from "~root/versions/versions.json" with { type: "json" }

export function chainVersion(
  parameters: { chainId?: keyof typeof versions } = {
    chainId: import.meta.env.CHAIN_ID_VERSION
  }
) {
  const chainId = parameters.chainId ?? Object.keys(versions).at(-1)
  if (!(chainId && isKeyOf(versions, chainId))) raise(`Invalid chainId: ${parameters.chainId}`)
  return versions[chainId]
}

/**
 * [nodeId]@[address]:[port]
 */
export function getSeedsParts(chainId = import.meta.env.CHAIN_ID_VERSION) {
  const { seeds } = chainVersion({ chainId })
  const [nodeId, address] = seeds.split("@")
  const [addressPart, port] = address.split(":")
  return { nodeId, address: addressPart, port }
}
