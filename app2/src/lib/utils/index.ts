import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"
import { bech32 } from "@scure/base"

export function cn(...inputs: Array<ClassValue>) {
  return twMerge(clsx(inputs))
}

export function debounce<T extends (...args: Array<any>) => void>(
  handler: T,
  delay = 500
): (...args: Parameters<T>) => void {
  let id: number
  return (...args: Parameters<T>) => {
    window.clearTimeout(id)
    id = window.setTimeout(handler, delay, ...args)
  }
}

export function isValidBech32ContractAddress(address: unknown) {
  if (typeof address !== "string") return false
  try {
    const { prefix: _, words } = bech32.decode(address)
    return true
  } catch {
    return false
  }
}
