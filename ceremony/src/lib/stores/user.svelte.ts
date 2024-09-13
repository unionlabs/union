import type { Session } from "@supabase/supabase-js"

type UserState = {
  session: Session | null;
};

export const user = $state<UserState>({ session: null });
