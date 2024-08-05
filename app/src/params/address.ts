import { address as addressUtilities } from "@union/client"

/**
 * what is this?
 * answer: https://kit.svelte.dev/docs/advanced-routing#matching
 */

export function match(param: string) {
  const addresses = param.indexOf("-") === -1 ? [param] : param.split("-")
  return addresses.every(
    address =>
      addressUtilities.isValidEvmAddress(address) || addressUtilities.isValidBech32Address(address)
  )
}