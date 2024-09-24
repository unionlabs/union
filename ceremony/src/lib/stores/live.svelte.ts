import { supabase } from "$lib/supabase/client"
import { getContext, onDestroy, setContext } from "svelte"
import type { RealtimeChannel, RealtimePostgresInsertPayload } from "@supabase/supabase-js"

export class Logs {
  #subscription: RealtimeChannel
  data = $state<Array<any>>([])

  constructor() {
    this.#getInitialData()
    this.#subscription = this.#setUpSubscription()
    onDestroy(() => {
      this.#subscription.unsubscribe()
    })
  }

  #getInitialData = async () => {
    const { data: logs, error } = await supabase
      .from("log")
      .select("message, created_at")
      .order("created_at", { ascending: false })
      .limit(10)

    if (error) {
      console.error("Error fetching initial data:", error)
    } else {
      this.data = logs
    }
  }

  #setUpSubscription = (): RealtimeChannel => {
    return supabase
      .channel("table_db_changes")
      .on(
        "postgres_changes",
        {
          event: "INSERT",
          schema: "public",
          table: "log"
        },
        (payload: RealtimePostgresInsertPayload<{ message: string; created_at: string }>) =>
          this.#handleInserts(payload.new)
      )
      .subscribe()
  }

  #handleInserts = (payload: { message: string; created_at: string }) => {
    this.data = [payload, ...this.data]
  }
}

const CONTEXT_KEY = Symbol("LOGS")

export function setLiveLogsState() {
  return setContext(CONTEXT_KEY, new Logs())
}

export function getLiveLogsState(): Logs {
  return getContext<Logs>(CONTEXT_KEY)
}
