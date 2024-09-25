import { getContext, setContext } from "svelte"

export type AuthProviders = "github" | "google"
export type State = "hasRedeemed" | "inQueue" | "inWaitlist" | "join" | undefined

export class Terminal {
  state = $state<State>(undefined)
  history = $state<Array<string>>([])
  tab = $state<1 | 2 | 3 | number>(1)
  hash = $state<string | undefined>(undefined)

  constructor() {
    console.log("Creating terminal state")
  }

  updateHistory(text: string) {
    if (!this.history.includes(text)) {
      this.history.push(text)
      return true
    }
  }

  setTab(tab: 1 | 2 | 3 | number) {
    this.tab = tab
  }

  setHash(hash: string) {
    this.hash = hash
  }

  private updateState() {}
}

const TERMINAL_KEY = Symbol("TERMINAL")

export function setTerminal() {
  return setContext(TERMINAL_KEY, new Terminal())
}

export function getTerminal(): Terminal {
  return getContext<Terminal>(TERMINAL_KEY)
}
