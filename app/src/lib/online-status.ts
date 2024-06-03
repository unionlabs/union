import {toast} from "svelte-sonner"
import { readable, type Readable } from "svelte/store"
import { onlineManager } from "@tanstack/svelte-query"

/* Simple boolean store that reflects the online status of the user (browser) */
export const onlineStatus = readable(true, set =>
  onlineManager.subscribe(() => {
    const isOnline = onlineManager.isOnline()
    set(isOnline)
    if (isOnline) toast.success("Welcome back")
    else toast.error("Your connection is offline", { duration: 3_500 })
  })
) satisfies Readable<boolean>
