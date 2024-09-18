import { writable } from "svelte/store"

export const reactiveQueryArgs = <T>(cb: () => T) => {
  const store = writable<T>()

  $effect.pre(() => {
    store.set(cb())
  })

  return store
}

export function getNumberSuffix(n: number | null): string {
  if (!n) return ""
  const j = n % 10
  const k = n % 100
  if (j == 1 && k != 11) {
    return "st"
  }
  if (j == 2 && k != 12) {
    return "nd"
  }
  if (j == 3 && k != 13) {
    return "rd"
  }
  return "th"
}
