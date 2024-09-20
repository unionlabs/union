<script lang="ts">
import { afterNavigate, beforeNavigate, goto } from "$app/navigation"
import { checkAuth, type SessionError } from "$lib/utils/auth.ts"
import { supabase } from "$lib/supabase/client.ts"
import { user } from "$lib/stores/user.svelte.ts"
import { Toaster } from "svelte-sonner"
import Navbar from "$lib/layout/Navbar/index.svelte"
import { setContributorState } from "$lib/stores/state.svelte.ts"

import "../styles/tailwind.css"

import { watch } from "runed"

let { children } = $props()

let contributor = setContributorState()

beforeNavigate(async ({ from, to, cancel }) => {
  const pathname = to?.route?.id
  if (pathname) {
    const segments = pathname.split("/").filter(Boolean)
    if (segments[0] === "0____0") {
      const authCheck = await checkAuth()

      authCheck.match(
        () => {
          console.log("User authenticated")
        },
        (error: SessionError) => {
          console.error(error.message)
          cancel()
          goto("/")
        }
      )
    }
  }
})

afterNavigate(() => {
  const url = new URL(window.location.href)
  if (url.hash) {
    url.hash = ""
    history.replaceState(null, "", url.toString())
  }
})

$effect(() => {
  const {
    data: { subscription }
  } = supabase.auth.onAuthStateChange((event, session) => {
    user.session = session
  })
  return () => {
    subscription.unsubscribe()
  }
})

watch(
  () => user.session?.user.id,
  () => {
    contributor.setUserId(user.session?.user.id)
  }
)
</script>

<Toaster position="bottom-right" toastOptions={{ class: 'rounded-none border border-black',}}/>
<Navbar/>

<main>
  {@render children()}
</main>