import { supabase } from "$lib/supabase.ts";
import {err, ok, Result} from "neverthrow";

export type SessionError = {
  message: string;
};

export async function checkAuth(): Promise<Result<null, SessionError>> {
  const { data: { session }, error } = await supabase.auth.getSession();

  if (error || !session) {
    return err({ message: 'User not authenticated' });
  }

  return ok(null);
}