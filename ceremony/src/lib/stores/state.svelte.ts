import {onDestroy} from "svelte";
import {checkState} from "$lib/client";
import {checkContributionState, getUserQueueInfo} from "$lib/supabase";

type IntervalID = NodeJS.Timeout | number;

type UserState =
  | 'idle'
  | 'loading'
  | 'waitingList'
  | 'inQueue'
  | 'contribute'
  | 'contributing'
  | 'verifying'
  | 'contributed'
  | 'error'
  | 'offline'

export type ContributionState = 'contribute' | 'contributed' | 'verifying' | 'notContributed' | null;
export type ClientState =
  "idle"
  | "initializing"
  | "contributionStarted"
  | "contributionEnded"
  | "successful"
  | "offline"

interface UserContext {
  queuePosition: number | null;
  queueCount: number | null;
  estimatedTime: number | null;
  error: string | null;
}

interface QueueInfoSuccess {
  inQueue: true;
  position: number;
  count: number;
}

interface QueueInfoError {
  inQueue: false;
  message: string;
}

type QueueInfoResult = QueueInfoSuccess | QueueInfoError;

export class ContributorState {
  userId = $state<string | undefined>(undefined);
  state = $state<UserState>('offline');
  clientState = $state<ClientState>('offline')
  contributionState= $state<ContributionState>()
  pollingState = $state<'stopped' | 'polling'>('stopped');
  loggedIn = $state<boolean>(false);
  ctx = $state<UserContext>({
    queuePosition: null,
    queueCount: null,
    estimatedTime: null,
    error: null,
  });

  private pollIntervals: {
    client: IntervalID | null;
    queue: IntervalID | null;
    contribution: IntervalID | null;
  } = {
    client: null,
    queue: null,
    contribution: null,
  };

  constructor(userId?: string) {
    if (userId) {
      this.userId = userId;
      this.loggedIn = true
      this.startPolling();
    }
    onDestroy(() => {
      this.stopPolling();
    });
  }

  setUserId(userId: string | undefined) {
    if (this.userId === undefined && userId) {
      this.userId = userId;
      this.loggedIn = true
      this.startPolling();
    }
  }

  startPolling() {
    if (this.pollingState === 'polling') {
      console.log('Polling is already running.');
      return;
    }

    if (!this.userId) {
      console.log('Cannot start polling without userId.');
      return;
    }

    this.pollingState = 'polling';
    this.startClientStatePolling();
    this.startQueueInfoPolling();
    this.startContributionStatePolling();
  }

  stopPolling() {
    if (this.pollingState === 'stopped') {
      console.log('Polling is already stopped.');
      return;
    }

    this.pollingState = 'stopped';
    this.stopClientStatePolling();
    this.stopQueueInfoPolling();
    this.stopContributionStatePolling();
  }

  private startClientStatePolling() {
    this.pollClientState();
    this.pollIntervals.client = setInterval(() => this.pollClientState(), 5000) as IntervalID;
  }

  private stopClientStatePolling() {
    if (this.pollIntervals.client) {
      clearInterval(this.pollIntervals.client);
      this.pollIntervals.client = null;
    }
  }

  private async pollClientState() {
    const state = await checkState();
    this.updateClientState(state);
  }

  private startQueueInfoPolling() {
    this.pollQueueInfo();
    this.pollIntervals.queue = setInterval(() => this.pollQueueInfo(), 5000) as IntervalID;
  }

  private stopQueueInfoPolling() {
    if (this.pollIntervals.queue) {
      clearInterval(this.pollIntervals.queue);
      this.pollIntervals.queue = null;
    }
  }

  private async pollQueueInfo() {
    try {
      const queueInfo = await getUserQueueInfo();
      this.updateQueueInfo(queueInfo);
    } catch (error) {
      console.log('Error polling queue info:', error);
      this.setError(error instanceof Error ? error.message : 'Unknown error occurred');
    }
  }

  private startContributionStatePolling() {
    this.pollContributionState();
    this.pollIntervals.contribution = setInterval(() => this.pollContributionState(), 5000) as IntervalID;
  }

  private stopContributionStatePolling() {
    if (this.pollIntervals.contribution) {
      clearInterval(this.pollIntervals.contribution);
      this.pollIntervals.contribution = null;
    }
  }

  private async pollContributionState() {
    try {
      const state = await checkContributionState();
      this.updateContributionState(state);
    } catch (error) {
      console.log('Error polling contribution state:', error);
      this.setError(error instanceof Error ? error.message : 'Unknown error occurred');
    }
  }

  private updateClientState(state: ClientState) {
    this.clientState = state
    this.updateState();
  }

  private updateQueueInfo(queueInfo: QueueInfoResult) {
    if (queueInfo.inQueue) {
      this.ctx = {
        ...this.ctx,
        queuePosition: queueInfo.position,
        queueCount: queueInfo.count,
        estimatedTime: queueInfo.position * 30,
      };
    } else {
      this.ctx = {
        ...this.ctx,
        queuePosition: null,
        queueCount: null,
        estimatedTime: null,
      };
    }
    this.updateState();
  }

  private updateContributionState(state: ContributionState) {
    this.contributionState = state
    this.updateState();
  }

  private setError(message: string) {
    this.ctx = {...this.ctx, error: message};
    this.state = 'error';
  }

  private updateState() {
    if (this.contributionState === 'contribute') {
      this.state = 'contribute';
    } else if (this.ctx.queuePosition !== null) {
      this.state = 'inQueue';
    } else if (this.clientState === 'contributionStarted') {
      this.state = 'contributing';
    } else if (this.contributionState === 'verifying') {
      this.state = 'verifying';
    } else if (this.clientState === 'successful') {
      this.state = 'contributed';
    } else {
      this.state = 'loading';
    }
  }

}