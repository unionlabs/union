import { writable } from "svelte/store"

export function persistedWritable<T>(key: string, data: T) {
  const { subscribe, set, update } = writable(data)
  const isBrowser = typeof window !== "undefined"

  return {
    set,
    update,
    subscribe,
    useLocalStorage: () => {
      const json = localStorage.getItem(key)
      if (!json) return
      if (json) set(JSON.parse(json) as T)

      subscribe(current => {
        localStorage.setItem(key, JSON.stringify(current))
      })
    }
  }
}
