import { get, post } from "$lib/client/http.ts"
import { getQueuePayloadId } from "$lib/supabase/queries.ts"
import type { ClientState, ContributeBody } from "$lib/client/types.ts"
import { supabase } from "$lib/supabase/client.ts"

export const start = async (): Promise<ClientState | undefined> => {
  const { data: session, error: sessionError } = await supabase.auth.refreshSession()

  if (sessionError) {
    console.error("Error refreshing session:", sessionError)
    return
  }

  const userId = session.session?.user.id
  const email = session.session?.user?.email

  if (!userId) {
    console.log("User not logged in")
    return
  }

  const { data, error } = await getQueuePayloadId(userId)

  if (error) {
    console.log("Error fetching payload_id:", error)
    return
  }

  if (!data) {
    console.log("No data found for the given user ID")
    return
  }

  const contributeBody: Partial<ContributeBody> = {
    payloadId: data.payload_id,
    contributorId: userId,
    jwt: session.session?.access_token,
    supabaseProject: import.meta.env.VITE_SUPABASE_URL,
    apiKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
    bucket: import.meta.env.VITE_BUCKET_ID,
    userEmail: email
  }

  return post<ClientState>("contribute", {}, contributeBody)
}

export const checkState = async (): Promise<ClientState> => {
  try {
    const response = await get<ClientState>("contribute", {})

    return response ?? "offline"
  } catch (error) {
    console.log("Error fetching status:", error)
    return "offline"
  }
}
