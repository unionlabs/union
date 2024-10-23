import { createClient, type SupabaseClient } from "@supabase/supabase-js"
import { browser } from "$app/environment"

const SUPABASE_ANON_KEY = import.meta.env.VITE_SUPABASE_ANON_KEY
const SUPABASE_URL = import.meta.env.VITE_SUPABASE_URL

export const createSupabaseClient = () => {
  let client: SupabaseClient | null = null
  let refreshInterval: NodeJS.Timeout | null = null
  const REFRESH_INTERVAL = 5 * 60 * 1000 // 5 minutes

  const getClient = () => {
    if (client) return client

    client = createClient(SUPABASE_URL, SUPABASE_ANON_KEY)

    if (browser) {
      const refreshSession = async (supabase: SupabaseClient) => {
        try {
          const {
            data: { session },
            error
          } = await supabase.auth.getSession()

          if (error || !session) {
            if (refreshInterval) clearInterval(refreshInterval)
            refreshInterval = null
            return
          }

          const {
            data: { session: newSession },
            error: refreshError
          } = await supabase.auth.refreshSession({
            refresh_token: session.refresh_token
          })

          if (refreshError) {
            console.error("Session refresh failed:", refreshError)
            return
          }

          if (!newSession) {
            if (refreshInterval) clearInterval(refreshInterval)
            refreshInterval = null
          }
        } catch (error) {
          console.error("Session refresh failed:", error)
        }
      }

      refreshInterval = setInterval(() => {
        if (client) {
          refreshSession(client)
        }
      }, REFRESH_INTERVAL)

      // Clean up on window unload
      window.addEventListener("beforeunload", () => {
        if (refreshInterval) {
          clearInterval(refreshInterval)
        }
      })
    }

    return client
  }

  return getClient()
}

export const supabase = createSupabaseClient()
