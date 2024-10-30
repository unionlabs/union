import { getContext, setContext } from "svelte"
import { Contributor } from "./contributor.svelte.ts"
import { Terminal } from "./terminal.svelte.ts"
import { user, type UserSession } from "$lib/state/session.svelte.ts"

export interface AppState {
  contributor: Contributor
  terminal: Terminal
  user: UserSession
}

const STATE_KEY = Symbol("STATE")

export function createState() {
  console.log("Creating state")
  const state: AppState = {
    contributor: new Contributor(),
    terminal: new Terminal(),
    user: user
  }

  setContext(STATE_KEY, state)
  return state
}

export function getState(): AppState {
  return getContext<AppState>(STATE_KEY)
}
