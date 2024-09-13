import {get, post} from "$lib/client/http.ts"
import {user} from "$lib/stores/user.svelte.ts"
import {getQueuePayloadId} from "$lib/supabase/queries.ts"
import type {ClientState, ContributeBody} from "$lib/client/types.ts";

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
    userEmail: email,
  }

  return post<ClientState>("contribute", {}, contributeBody)
}


export const checkState = async (): Promise<ClientState> => {
  try {
    const response = await get<ClientState>("contribute", {});

    console.log('resssss',response)
    return response ?? 'offline';
  } catch (error) {
    console.log('Error fetching status:', error);
    return 'offline';
  }
};


