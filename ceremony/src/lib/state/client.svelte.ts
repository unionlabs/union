import { checkState, start } from "$lib/client"

export type ClientState =
  | "idle"
  | "initializing"
  | "downloadStarted"
  | "downloading"
  | "downloadEnded"
  | "contributionStarted"
  | "contributionEnded"
  | "uploadStarted"
  | "uploadEnded"
  | "failed"
  | "successful"
  | "offline"
  | undefined

const CLIENT_POLLING_INTERVAL = 1000 // 1 second

export class Client {
  state = $state<ClientState>(undefined)
  isPolling = $state<boolean>(false)
  private pollingInterval: NodeJS.Timeout | null = null

  constructor() {
    console.log("Creating client state")
  }

  async checkStatus(): Promise<void> {
    const newState = await checkState()
    this.updateState(newState)
  }

  private updateState(newState: ClientState): void {
    this.state = newState
  }

  contribute() {
    start()
  }

  startPolling(): void {
    if (!this.isPolling) {
      this.pollingInterval = setInterval(
        () => this.checkStatus(),
        CLIENT_POLLING_INTERVAL
      )
      this.isPolling = true
    }
  }

  stopPolling(): void {
    if (this.isPolling && this.pollingInterval !== null) {
      clearInterval(this.pollingInterval)
      this.pollingInterval = null
      this.isPolling = false
    }
  }
}