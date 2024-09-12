import { writable } from "svelte/store"

export const reactiveQueryArgs = <T>(cb: () => T) => {
  const store = writable<T>()

  $effect.pre(() => {
    store.set(cb())
  })

  return store
}
