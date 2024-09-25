import { checkState, start } from "$lib/client"
import type {ClientStatus} from "$lib/client/types.ts";

type ClientState = undefined | "contributing" | "error" | "noClient" | "successful" | "idle"

const CLIENT_POLLING_INTERVAL = 1000 // 1 second

export class Client {
  status = $state<ClientStatus>()
  state = $state<ClientState>(undefined)
  isPolling = $state<boolean>(false)
  private pollingInterval: NodeJS.Timeout | null = null

  constructor() {
    console.log("Creating client state")
    this.startPolling()
  }

  async checkStatus(): Promise<void> {
    const status = await checkState()
    console.log(status)
    this.updateState(status)
  }

  private updateStatus(newStatus: ClientStatus): void {
    this.status = newStatus
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

  updateState(status: ClientStatus) {
    switch (status) {
      case "initializing":
      case "downloadStarted":
      case "downloading":
      case "downloadEnded":
      case "contributionStarted":
      case "contributionEnded":
      case "uploadStarted":
      case "uploadEnded":
        this.state = "contributing"
        break
      case "successful":
        this.state = "successful"
        break
      case "failed":
        this.state = "error"
        break
      case "offline":
        this.state = "noClient"
        break
      default:
        this.state = "idle"
        break
    }
  }

}