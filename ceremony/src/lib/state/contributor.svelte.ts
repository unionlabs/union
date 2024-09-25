import { getContext, onDestroy, setContext } from "svelte"
import {
  getCurrentUserState,
  getUserQueueInfo,
  getContributionState,
  getUserWallet,
  getWaitListPosition,
  checkIfOpen
} from "$lib/supabase"

type IntervalID = NodeJS.Timeout | number

type State =
  | "loading"
  | "inQueue"
  | "contribute"
  | "contributing"
  | "verifying"
  | "contributed"
  | "error"
  | "offline"
  | "noClient"

export type AllowanceState = "hasRedeemed" | "inWaitlist" | "inQueue" | "join" | undefined
export type ContributionState = "contribute" | "contributed" | "verifying" | "notContributed"

interface UserContext {
  position: number | null
  count: number | null
  estimatedTime: number | null
  error: string | null
}

interface QueueInfoSuccess {
  inQueue: true
  position: number
  count: number
}

interface QueueInfoError {
  inQueue: false
  message: string
}

type QueueInfoResult = QueueInfoSuccess | QueueInfoError

const second = 1000
const CONTRIBUTION_POLLING_INTERVAL = second * 5
const QUEUE_POLLING_INTERVAL = second * 5

export class Contributor {
  userId = $state<string | undefined>(undefined)
  loggedIn = $state<boolean>(false)
  currentUserState = $state<AllowanceState>(undefined)
  pollingState = $state<"stopped" | "polling">("stopped")
  state = $state<State>("loading")

  openToPublic = $state(false)
  contributionState = $state<ContributionState>("notContributed")
  userWallet = $state("")
  waitListPosition = $state<number | undefined>(undefined)
  downloadedSecret = $state<boolean>(localStorage.getItem("downloaded-secret") === "true")

  queueState = $state<UserContext>({
    position: null,
    count: null,
    estimatedTime: null,
    error: null
  })

  private pollIntervals: {
    client: IntervalID | null
    queue: IntervalID | null
    contribution: IntervalID | null
  } = {
    client: null,
    queue: null,
    contribution: null
  }

  constructor(userId?: string) {
    if (userId) {
      this.userId = userId
      this.loggedIn = true
      this.checkCurrentUserState(userId)
      this.checkIfOpen()
      this.startPolling()
    }
    onDestroy(() => {
      this.stopPolling()
    })
  }

  setUserId(userId: string | undefined) {
    if (this.userId === undefined && userId) {
      this.userId = userId
      this.loggedIn = true
      this.checkWaitListPosition(userId)
      this.checkUserWallet(userId)
      this.checkCurrentUserState(userId)
      this.startPolling()
    }
  }

  async checkIfOpen() {
    this.openToPublic = await checkIfOpen()
  }

  async checkWaitListPosition(_userId: string | undefined): Promise<number | undefined> {
    this.waitListPosition = await getWaitListPosition()
    return this.waitListPosition
  }

  async checkCurrentUserState(userId: string | undefined): Promise<AllowanceState> {
    this.currentUserState = await getCurrentUserState(userId)
    return this.currentUserState
  }

  async checkUserWallet(userId: string | undefined): Promise<string> {
    this.userWallet = await getUserWallet(userId)
    return this.userWallet
  }

  setAllowanceState(state: AllowanceState) {
    this.currentUserState = state
    this.pollQueueInfo()
    this.pollContributionState()
  }

  startPolling() {
    if (this.pollingState === "polling") {
      console.log("Polling is already running.")
      return
    }

    if (!this.userId) {
      console.log("Cannot start polling without userId.")
      return
    }

    this.pollingState = "polling"
    this.startQueueInfoPolling()
    this.startContributionStatePolling()
  }

  stopPolling() {
    if (this.pollingState === "stopped") {
      console.log("Polling is already stopped.")
      return
    }

    this.pollingState = "stopped"
    this.stopClientStatePolling()
    this.stopQueueInfoPolling()
    this.stopContributionStatePolling()
  }

  private stopClientStatePolling() {
    if (this.pollIntervals.client) {
      clearInterval(this.pollIntervals.client)
      this.pollIntervals.client = null
    }
  }

  private startQueueInfoPolling() {
    this.pollQueueInfo()
    this.pollIntervals.queue = setInterval(
      () => this.pollQueueInfo(),
      QUEUE_POLLING_INTERVAL
    ) as IntervalID
  }

  private stopQueueInfoPolling() {
    if (this.pollIntervals.queue) {
      clearInterval(this.pollIntervals.queue)
      this.pollIntervals.queue = null
    }
  }

  private async pollQueueInfo() {
    try {
      const queueInfo = await getUserQueueInfo()
      this.updateQueueInfo(queueInfo)
    } catch (error) {
      console.log("Error polling queue info:", error)
      this.setError(error instanceof Error ? error.message : "Unknown error occurred")
    }
  }

  private startContributionStatePolling() {
    this.pollContributionState()
    this.pollIntervals.contribution = setInterval(
      () => this.pollContributionState(),
      CONTRIBUTION_POLLING_INTERVAL
    ) as IntervalID
  }

  private stopContributionStatePolling() {
    if (this.pollIntervals.contribution) {
      clearInterval(this.pollIntervals.contribution)
      this.pollIntervals.contribution = null
    }
  }

  private async pollContributionState() {
    try {
      const state = await getContributionState()
      this.updateContributionState(state)
    } catch (error) {
      console.log("Error polling contribution state:", error)
      this.setError(error instanceof Error ? error.message : "Unknown error occurred")
    }
  }

  private updateQueueInfo(queueInfo: QueueInfoResult) {
    if (queueInfo.inQueue) {
      this.queueState = {
        ...this.queueState,
        position: queueInfo.position,
        count: queueInfo.count,
        estimatedTime: queueInfo.position * 60
      }
    } else {
      this.queueState = {
        ...this.queueState,
        position: null,
        count: null,
        estimatedTime: null
      }
    }
  }

  private updateContributionState(state: ContributionState) {
    this.contributionState = state
  }

  private setError(message: string) {
    this.queueState = { ...this.queueState, error: message }
    this.state = "error"
  }
}

const CONTRIBUTOR_KEY = Symbol("CONTRIBUTOR")

export function setContributorState() {
  return setContext(CONTRIBUTOR_KEY, new Contributor())
}

export function getContributorState(): Contributor {
  return getContext<Contributor>(CONTRIBUTOR_KEY)
}
