import { getContributions } from "$lib/supabase"

export class Contributions {
  data = $state()
  private readonly intervalId: NodeJS.Timeout | null = null

  constructor() {
    this.loadContributions()
    this.intervalId = setInterval(() => this.loadContributions(), 5000)
  }

  async loadContributions() {
    try {
      this.data = await getContributions()
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
