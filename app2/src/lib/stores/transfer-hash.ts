import { writable } from "svelte/store"
import { Option } from "effect"

const createTransferHashStore = () => {
  const { subscribe, set, update } = writable(Option.none<string>())

  return {
    reset: () => set(Option.none()),
    set,
    setHash: (hash: string) => set(Option.some(hash)),
    subscribe,
    update
  }
}

export const transferHash = createTransferHashStore()
