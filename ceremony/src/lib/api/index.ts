import {get, post} from "$lib/api/http.ts";
import type {ContributeBody, Status} from "$lib/api/types.ts";

export const fetchStatus = (): Promise<Status | undefined> => {
  return get<Status>('contribute', {});
};

export const contribute = (body: Partial<ContributeBody>): Promise<Status | undefined> => {
  return post<Status>('contribute', {}, {
    ...body,
    supabase_project: import.meta.env.VITE_SUPABASE_URL,
    api_key: import.meta.env.VITE_SUPABASE_URL,
  });
};