import {
  getCurrentUserState,
  getUserQueueInfo,
  getContributionState,
  getUserWallet,
  getWaitListPosition
} from "$lib/supabase"
import type { Session } from "@supabase/supabase-js"

export type AllowanceState = "hasRedeemed" | "inWaitlist" | "inQueue" | "join" | undefined
export type ContributionState = "contribute" | "contributed" | "verifying" | "notContributed"

interface QueueState {
  position: number | null
  count: number | null
  estimatedTime: number | null
  error: string | null
}

type UserState = Session | null

const QUEUE_POLLING_INTERVAL = 5000
const CONTRIBUTION_POLLING_INTERVAL = 5000

export class Contributor {
  currentUserState = $state<AllowanceState>(undefined)
  contributionState = $state<ContributionState>("notContributed")
  userWallet = $state("")

  waitListPosition = $state<number | undefined>(undefined)

  queueState = $state<QueueState>({
    position: null,
    count: null,
    estimatedTime: null,
    error: null
  })

  private queuePollingInterval: number | null = null
  private contributionPollingInterval: number | null = null

  // if(this.session) {
  // await Promise.all([
  //                     this.checkWaitListPosition(),
  //                     this.checkUserWallet(),
  //                     this.checkCurrentUserState()
  //                   ])
  // this.startPolling()
  //}

  private async checkWaitListPosition(): Promise<void> {
    this.waitListPosition = await getWaitListPosition()
  }

  private async checkCurrentUserState(): Promise<void> {
    this.currentUserState = await getCurrentUserState("")
  }

  private async checkUserWallet(): Promise<void> {
    this.userWallet = await getUserWallet("")
  }

  startPolling(): void {
    this.startQueuePolling()
    this.startContributionPolling()
  }

  stopPolling(): void {
    this.stopQueuePolling()
    this.stopContributionPolling()
  }

  private startQueuePolling(): void {
    if (this.queuePollingInterval === null) {
      this.pollQueueInfo()
      this.queuePollingInterval = setInterval(
        () => this.pollQueueInfo(),
        QUEUE_POLLING_INTERVAL
      ) as unknown as number
    }
  }

  private stopQueuePolling(): void {
    if (this.queuePollingInterval !== null) {
      clearInterval(this.queuePollingInterval)
      this.queuePollingInterval = null
    }
  }

  private async pollQueueInfo(): Promise<void> {
    try {
      const queueInfo = await getUserQueueInfo()
      if ("inQueue" in queueInfo && queueInfo.inQueue) {
        this.queueState = {
          position: queueInfo.position,
          count: queueInfo.count,
          estimatedTime: queueInfo.position * 60,
          error: null
        }
      } else {
        this.queueState = {
          position: null,
          count: null,
          estimatedTime: null,
          error: null
        }
      }
    } catch (error) {
      console.error("Error polling queue info:", error)
      this.queueState = {
        ...this.queueState,
        error: error instanceof Error ? error.message : "Unknown error occurred"
      }
    }
  }

  private startContributionPolling(): void {
    if (this.contributionPollingInterval === null) {
      this.pollContributionState()
      this.contributionPollingInterval = setInterval(
        () => this.pollContributionState(),
        CONTRIBUTION_POLLING_INTERVAL
      ) as unknown as number
    }
  }

  private stopContributionPolling(): void {
    if (this.contributionPollingInterval !== null) {
      clearInterval(this.contributionPollingInterval)
      this.contributionPollingInterval = null
    }
  }

  private async pollContributionState(): Promise<void> {
    try {
      this.contributionState = await getContributionState()
    } catch (error) {
      console.error("Error polling contribution state:", error)
      // You might want to handle this error differently
    }
  }

  constructor() {
    console.log("Creating contributor state")
  }
}
