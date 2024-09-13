import { bech32 } from "@scure/base"

type Bech32Address<T extends string = string> = `${T}1${string}`
export function isValidBech32Address(address: unknown): address is Bech32Address {
  if (typeof address !== "string") return false

  try {
    const { prefix: _, words } = bech32.decode(address as Bech32Address)
    const size = words.length
    if ([20, 32].indexOf(size) === -1) return false
    return true
  } catch {
    return false
  }
}
