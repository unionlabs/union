import { getContributions } from "$lib/supabase"

type ContributionsData = {
  public_key_hash: string
  payload_id: string
}

export class Contributions {
  data = $state<Array<ContributionsData>>([])
  private readonly intervalId: NodeJS.Timeout | null = null

  constructor() {
    this.loadContributions()
    this.intervalId = setInterval(() => this.loadContributions(), 5000)
  }

  async loadContributions() {
    try {
      this.data = (await getContributions()) ?? []
    } catch (error) {
      console.error("Failed to load contributions:", error)
    }
  }

  destroy() {
    if (this.intervalId) {
      clearInterval(this.intervalId)
    }
  }
}
