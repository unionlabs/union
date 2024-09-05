import { get, post } from "$lib/api/http.ts"
import type { ContributeBody, Status } from "$lib/api/types.ts"
import { supabase } from "$lib/supabase/client.ts"
import { user } from "$lib/stores/user.svelte.ts"

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

export const contribute = (body: Partial<ContributeBody>): Promise<Status | undefined> => {
  const data = {
    ...body,
    contributorId: user?.session?.user.id,
    jwt: user?.session?.access_token,
    supabaseProject: import.meta.env.VITE_SUPABASE_URL,
    apiKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
    bucket: import.meta.env.VITE_BUCKET_ID
  }

  console.log(data)

  return post<Status>("contribute", {}, data)
}

export const checkPosition = async (): Promise<{ position: number } | null> => {
  const { data: queueData, error } = await supabase
    .from("queue")
    .select("id, joined")
    .order("joined", { ascending: true }) // Order by the joined timestamp

  if (error) {
    console.error("Error fetching queue:", error)
    return null
  }

  const position = queueData.findIndex(row => row.id === user?.session?.user.id)
  const userPosition = position !== -1 ? position + 1 : -1

  return {
    position: userPosition
  }
}
