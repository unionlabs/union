import { type Writable, writable } from "svelte/store";
import type {Session} from "@supabase/supabase-js";

export const userSession: Writable<null | Session> = writable(null)