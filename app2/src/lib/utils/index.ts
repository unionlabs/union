import { bech32 } from "@scure/base"
import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: Array<ClassValue>) {
  return twMerge(clsx(inputs))
}

export function debounce<T extends (...args: Array<any>) => void>(
  handler: T,
  delay = 500,
): (...args: Parameters<T>) => void {
  let id: number
  return (...args: Parameters<T>) => {
    window.clearTimeout(id)
    id = window.setTimeout(handler, delay, ...args)
  }
}

// XXX: remove in favor of schema validation
export function isValidBech32ContractAddress(address: unknown) {
  if (typeof address !== "string") {
    return false
  }
  try {
    // doesn't matter because it's a trycatch and preserves behavior
    const { prefix: _ } = bech32.decode(address as unknown as any)
    return true
  } catch {
    return false
  }
}

export const vw = () => Math.max(document.documentElement.clientWidth || 0, window.innerWidth || 0)
export const vh = () =>
  Math.max(document.documentElement.clientHeight || 0, window.innerHeight || 0)
