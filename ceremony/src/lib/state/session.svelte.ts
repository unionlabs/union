import type { Session } from "@supabase/supabase-js"
import { supabase } from "$lib/supabase/client.ts"
import { err, ok, type Result } from "neverthrow"
import { goto, invalidateAll } from "$app/navigation"
import type { Terminal } from "$lib/state/terminal.svelte.ts"

export type UserSession = {
  session: Session | null | false
}

export type SessionError = {
  message: string
}

export let user = $state<UserSession>({ session: false })

export async function checkAuth(): Promise<Result<null, SessionError>> {
  const {
    data: { session },
    error
  } = await supabase.auth.getSession()
  if (error || !session) {
    return err({ message: "User not authenticated" })
  }
  if (session) {
    user.session = session
  }
  return ok(null)
}

export async function logout(terminal: Terminal) {
  if (user.session === null || user.session === undefined) return
  terminal.updateHistory("user logged out")
  const { error } = await supabase.auth.signOut()
  if (error) {
    terminal.updateHistory("error logging out")
  } else {
    user.session = null
    invalidateAll()
    goto("/")
  }
}
