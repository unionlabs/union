<script lang="ts">
import { goto } from "$app/navigation"
import { page } from "$app/state"
import { mapSupabaseErrorToCustomError } from "$lib/dashboard/errors"
import { errorStore } from "$lib/dashboard/stores/errors.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { onMount } from "svelte"

let hasError = $state(false)
let errorMessage = $state("")
let errorCause = $state("")
let isSuccess = $state(false)

onMount(() => {
  const returnUrl = page.url.searchParams.get("returnTo") || "/dashboard"
  const isLinking = page.url.searchParams.get("linking") === "true"
  const error = page.url.searchParams.get("error")
  const errorDescription = page.url.searchParams.get("error_description")
  const errorCode = page.url.searchParams.get("error_code")

  if (error) {
    hasError = true
    const mappedError = mapSupabaseErrorToCustomError(error, errorDescription || undefined)
    errorMessage = mappedError.message
    errorCause = errorDescription || error

    // Also show in the global error store
    errorStore.showError(mappedError)

    // Redirect back after showing the error
    setTimeout(() => {
      goto(returnUrl)
    }, 4000) // Slightly longer to read the error
    return
  }

  // Check if authentication was successful
  if (Option.isSome(dashboard.session)) {
    isSuccess = true
  }

  // Then ensure minimum wait time for successful auth
  setTimeout(() => {
    // Then check session and redirect
    if (Option.isSome(dashboard.session)) {
      goto(returnUrl)
    } else if (!isLinking) {
      goto("/")
    } else {
      // If linking but no session, something went wrong - redirect back
      goto(returnUrl)
    }
  }, 3000)
})
</script>

<div class="flex items-center justify-center min-h-screen">
  <div class="text-center max-w-md mx-auto px-4">
    {#if hasError}
      <!-- Error State -->
      <div class="mb-6">
        <svg
          class="w-16 h-16 text-red-500 mx-auto mb-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.232 15.5c-.77.833.192 2.5 1.732 2.5z"
          />
        </svg>
      </div>
      <h1 class="text-2xl font-bold mb-6 text-red-400">Authentication Failed</h1>

      <!-- Error Message Box -->
      <div class="bg-red-500/10 border border-red-500/30 rounded-lg p-4 mb-6">
        <div class="flex items-start gap-3">
          <svg
            class="w-5 h-5 text-red-400 mt-0.5 flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <div class="flex-1">
            <p class="text-red-100 text-sm leading-relaxed mb-3">{errorMessage}</p>
            {#if errorCause}
              <div class="pt-3 border-t border-red-500/20">
                <p class="text-red-300/70 text-xs font-medium mb-1">Cause:</p>
                <p class="text-red-200/80 text-xs font-mono bg-red-500/5 rounded px-2 py-1 break-words">
                  {errorCause}
                </p>
              </div>
            {/if}
          </div>
        </div>
      </div>

      <p class="text-sm text-zinc-500">Redirecting you back in a moment...</p>
    {:else if isSuccess}
      <!-- Success State -->
      <div class="mb-6">
        <svg
          class="w-16 h-16 text-green-500 mx-auto mb-4"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      </div>
      <h1 class="text-2xl font-bold mb-6 text-green-400">Account Updated!</h1>

      <!-- Success Message Box -->
      <div class="bg-green-500/10 border border-green-500/30 rounded-lg p-4 mb-6">
        <div class="flex items-start gap-3">
          <svg
            class="w-5 h-5 text-green-400 mt-0.5 flex-shrink-0"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <p class="text-green-100 text-sm leading-relaxed">
            Your account has been successfully updated.
          </p>
        </div>
      </div>
    {:else}
      <!-- Loading State -->
      <div class="mb-6">
        <div class="w-16 h-16 mx-auto mb-4 border-4 border-zinc-700 border-t-blue-500 rounded-full animate-spin">
        </div>
      </div>
      <h1 class="text-2xl font-bold mb-4">Updating account...</h1>
      <p class="text-zinc-400">Please wait while we update your account.</p>
    {/if}
  </div>
</div>
