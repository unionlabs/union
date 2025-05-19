<script lang="ts">
import { goto } from "$app/navigation"
import { page } from "$app/state"
import { createAuthError } from "$lib/dashboard/errors"
import { uiStore } from "$lib/dashboard/stores/ui"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { onMount } from "svelte"

onMount(() => {
  const returnUrl = page.url.searchParams.get("returnTo") || "/dashboard"
  const isLinking = page.url.searchParams.get("linking") === "true"
  const error = page.url.searchParams.get("error")
  const errorDescription = page.url.searchParams.get("error_description")

  if (error) {
    uiStore.showError(createAuthError(error, errorDescription || undefined))
  }

  // Then ensure minimum wait time
  setTimeout(() => {
    // Then check session and redirect
    if (Option.isSome(dashboard.session)) {
      goto(returnUrl)
    } else if (!isLinking) {
      goto("/")
    }
  }, 3000)
})
</script>

<div class="flex items-center justify-center min-h-screen">
  <div class="text-center">
    <h1 class="text-2xl font-bold mb-4">Updating account...</h1>
    <p class="text-zinc-400">Please wait while we update your account.</p>
  </div>
</div>
