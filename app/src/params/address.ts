import { isValidCosmosAddress, isValidEvmAddress } from "$lib/wallet/utilities/validate"

/**
 * what is this?
 * answer: https://kit.svelte.dev/docs/advanced-routing#matching
 */

export function match(param: string) {
  const addresses = param.indexOf("-") === -1 ? [param] : param.split("-")
  return addresses.every(address => isValidEvmAddress(address) || isValidCosmosAddress(address))
}
