import { get, post } from "$lib/client/http.ts"
import type { ContributeBody, Status } from "$lib/client/types.ts"
import { user } from "$lib/stores/user.svelte.ts"
import { getQueuePayloadId } from "$lib/supabase/queries.ts"

export const contribute = async (): Promise<Status | undefined> => {
  const userId = user?.session?.user.id

  if (!userId) {
    console.error("User not logged in")
    return
  }

  const { data, error } = await getQueuePayloadId(userId)

  if (error) {
    console.error("Error fetching payload_id:", error)
    return
  }

  if (!data) {
    console.log("No data found for the given user ID")
    return
  }

  const contributeBody: Partial<ContributeBody> = {
    payloadId: data.payload_id,
    contributorId: userId,
    jwt: user?.session?.access_token,
    supabaseProject: import.meta.env.VITE_SUPABASE_URL,
    apiKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
    bucket: import.meta.env.VITE_BUCKET_ID
  }

  return post<Status>("contribute", {}, contributeBody)
}

export const checkStatus = async (): Promise<{ status: Status }> => {
  try {
    const status = await get<Status>("contribute", {})
    if (status === undefined) {
      throw new Error("Status is undefined. Is the client up?")
    }
    return { status }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    throw new Error(`Error fetching status: ${errorMessage}`)
  }
}
