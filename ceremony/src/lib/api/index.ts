import { get, post } from "$lib/api/http.ts"
import type { ContributeBody, Status } from "$lib/api/types.ts"

export const fetchStatus = (): Promise<Status | undefined> => {
  return get<Status>("contribute", {})
}

export const contribute = (body: Partial<ContributeBody>): Promise<Status | undefined> => {
  // Build the full body with defaults
  const b = {
    ...body,
    supabaseProject: import.meta.env.VITE_SUPABASE_URL,
    apiKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
    bucket: import.meta.env.VITE_BUCKET_ID
  }

  console.log("Request Body:", b) // Log the body for debugging
  return post<Status>("contribute", {}, b)
}
