import { getContext, setContext } from "svelte"
import { readable } from "svelte/store"

export type AuthProviders = "github" | "google"
export type State = "hasRedeemed" | "inQueue" | "inWaitlist" | "join" | undefined

interface UpdateHistoryOptions {
  duplicate?: boolean
  replace?: boolean
}

export type KeyEvent = {
  key: string
  type: "keydown" | "keyup"
  shiftKey: boolean
  ctrlKey: boolean
}

export class Terminal {
  state = $state<State>(undefined)
  history = $state<Array<string>>([])
  tab = $state<1 | 2 | 3 | number>(1)
  hash = $state<string | undefined>(undefined)
  currentStep = $state<number>(0)
  maxStep = $state<number>(10)

  keys = readable<KeyEvent | null>(null, set => {
    const handleKeyEvent = (event: KeyboardEvent) => {
      set({
        key: event.key,
        type: event.type as "keydown" | "keyup",
        shiftKey: event.shiftKey,
        ctrlKey: event.ctrlKey
      })
    }

    if (typeof window !== "undefined") {
      window.addEventListener("keydown", handleKeyEvent)
      window.addEventListener("keyup", handleKeyEvent)
    }

    return () => {
      if (typeof window !== "undefined") {
        window.removeEventListener("keydown", handleKeyEvent)
        window.removeEventListener("keyup", handleKeyEvent)
      }
    }
  })

  constructor() {
    console.log("Creating terminal state")
  }

  updateHistory(text: string, options: UpdateHistoryOptions = {}) {
    const { duplicate = false, replace = false } = options

    const index = this.history.indexOf(text)

    if (duplicate) {
      this.history.push(text)
    } else if (replace && index !== -1) {
      this.history.splice(index, 1)
      this.history.push(text)
    } else if (!this.history.includes(text)) {
      this.history.push(text)
    }
  }

  clearHistory() {
    this.history = []
  }

  setTab(tab: 1 | 2 | 3 | number) {
    this.tab = tab
  }

  setHash(hash: string | undefined) {
    this.hash = hash
  }

  setStep(step: number) {
    this.currentStep = step
  }
}

const TERMINAL_KEY = Symbol("TERMINAL")

export function setTerminal() {
  return setContext(TERMINAL_KEY, new Terminal())
}

export function getTerminal(): Terminal {
  return getContext<Terminal>(TERMINAL_KEY)
}
