import { uneval } from "devalue"
import { browser } from "$app/environment"
import { writable, type Writable } from "svelte/store"

export function storedWritable<T>(key: string, initialValue: T | (() => T)): Writable<T> {
  let initial: T | undefined

  if (browser) {
    const storedValue = localStorage.getItem(key)
    if (storedValue) {
      console.info(`Restoring ${key} from localStorage`, storedValue)
      // biome-ignore lint/security/noGlobalEval: it's from rich harris it's ok
      initial = (0, eval)(`(${storedValue})`)
    }
  }

  if (!initial) {
    if (typeof initialValue === "function") {
      initial = (initialValue as () => T)()
    } else initial = initialValue
  }

  const store = writable<T>(initial)

  store.subscribe(value => {
    if (!browser) return
    localStorage.setItem(key, uneval(value))
  })

  return store
}
