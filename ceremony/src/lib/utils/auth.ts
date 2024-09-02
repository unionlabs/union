import { supabase } from "$lib/supabase.ts";
import { userSession } from "$lib/stores/session.ts";

export const isLoggedIn = () => {
  return supabase.auth.getSession().then(({ data }) => {
    userSession.set(data.session ?? null);
    return !!data.session;
  });
};