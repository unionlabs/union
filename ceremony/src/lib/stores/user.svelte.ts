import type { Session } from "@supabase/supabase-js"
import { writable } from "svelte/store"

export const user: { session: Session | null } = $state({ session: null })

export const reactiveQueryArgs = <T>(cb: () => T) => {
  const store = writable<T>()

  $effect.pre(() => {
    store.set(cb())
  })

  return store
}
