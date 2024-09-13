import {get, post} from "$lib/client/http.ts"
import type {ClientState, ContributeBody} from "$lib/client/types.ts"
import {user} from "$lib/stores/user.svelte.ts"
import {getQueuePayloadId} from "$lib/supabase/queries.ts"

export const start = async (): Promise<ClientState | undefined> => {
  const userId = user?.session?.user.id
  const email = user?.session?.user?.email

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
    jwt: user?.session?.access_token,
    supabaseProject: import.meta.env.VITE_SUPABASE_URL,
    apiKey: import.meta.env.VITE_SUPABASE_ANON_KEY,
    bucket: import.meta.env.VITE_BUCKET_ID,
  }

  return post<ClientState>("contribute", {}, contributeBody)
}


export const checkState = async (): Promise<ClientState> => {
  try {
    return await get<ClientState>("contribute", {});
  } catch (error) {
    console.log('Error fetching status:', error);
    return 'offline';
  }
};


