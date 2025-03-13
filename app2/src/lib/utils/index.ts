import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"
import {bech32} from "@scure/base";

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

/**
 * check if a string is a valid bech32 contract address
 * @example
 * ```ts
 * isValidBech32ContractAddress("union14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sgf2v9u")
 * ```
 */
export function isValidBech32ContractAddress(address: unknown) {
  if (typeof address !== "string") return false
  try {
    const { prefix: _, words } = bech32.decode(address)
    return true
  } catch {
    return false
  }
}