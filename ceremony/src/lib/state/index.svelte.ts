import { getContext, setContext } from "svelte"
import { Client } from "./client.svelte.ts"
import { Contributor } from "./contributor.svelte.ts"
import { Terminal } from "./terminal.svelte.ts"
import { user, type UserSession } from "$lib/state/session.svelte.ts"
import { Activity } from "$lib/state/live.svelte.ts"
import { Contributions } from "$lib/state/contributions.svelte.ts"

export interface AppState {
  client: Client
  contributor: Contributor
  terminal: Terminal
  activity: Activity
  contributions: Contributions
  user: UserSession
}

const STATE_KEY = Symbol("STATE")

export function createState() {
  console.log("Creating state")
  const state: AppState = {
    client: new Client(),
    contributor: new Contributor(),
    terminal: new Terminal(),
    activity: new Activity(),
    contributions: new Contributions(),
    user: user
  }

  setContext(STATE_KEY, state)
  return state
}

export function getState(): AppState {
  return getContext<AppState>(STATE_KEY)
}
