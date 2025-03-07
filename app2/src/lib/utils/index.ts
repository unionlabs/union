import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

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
