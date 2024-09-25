import type {Session} from "@supabase/supabase-js"
import {supabase} from "$lib/supabase/client.ts"
import {err, ok, type Result} from "neverthrow"
import {invalidateAll} from "$app/navigation"
import type {Terminal} from "$lib/state/terminal.svelte.ts"


export type SessionError = {
  message: string
}

export type UserSession = {
  session: Session | null
  loading: boolean
}

export let user = $state<UserSession>({session: null, loading: true})

export async function checkAuth(): Promise<Result<null, SessionError>> {
  user.loading = true;
  const {
    data: {session},
    error
  } = await supabase.auth.getSession()
  if (error || !session) {
    user.session = null;
    user.loading = false;
    return err({message: "User not authenticated"})
  }
  user.session = session;
  user.loading = false;
  return ok(null)
}

export async function logout(terminal: Terminal): Promise<void> {
  if (!user.session) {
    console.log('user already logged out');
    return;
  }

  terminal.updateHistory("logging out user...");

  try {
    terminal.setHash(undefined);
    terminal.setTab(1);

    terminal.updateHistory("user successfully logged out");
    const {error} = await supabase.auth.signOut();

    user.session = null;

    await invalidateAll();
  } catch (error) {
    terminal.updateHistory(`error logging out`);

    terminal.setHash(undefined);
    terminal.setTab(1);
    user.session = null;
  }
}
