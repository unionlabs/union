<script lang="ts">
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Effect, Option, pipe } from "effect"
import { AccountError } from "$lib/dashboard/errors"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
  import { errorStore } from "../stores/errors.svelte";

let isDeleting = false

function handleDelete() {
  const effect = pipe(
    Effect.sync(() =>
      confirm("Are you sure you want to delete your account? This action cannot be undone.")
    ),
    Effect.flatMap((confirmed) =>
      confirmed
        ? pipe(
          Effect.sync(() => {
            isDeleting = true
          }),
          Effect.flatMap(() => dashboard.deleteAccount()),
          Effect.catchAll((error) => {
            errorStore.showError(new AccountError({ 
              cause: extractErrorDetails(error),
              operation: "delete"
            }))
            return Effect.succeed(undefined)
          }),
          Effect.ensuring(
            Effect.sync(() => {
              isDeleting = false
            }),
          ),
        )
        : Effect.succeed(undefined)
    ),
  )

  return Effect.runPromise(effect)
}
</script>

{#if Option.isSome(dashboard.user)}
  <div class="text-center text-zinc-400 pb-12 text-xs">
    <h1 class="text-zinc-200 text-sm">Account Info:</h1>
    <p><span class="text-zinc-300">Email:</span> {dashboard.user.value.email}</p>
    <p><span class="text-zinc-300">ID:</span> {dashboard.user.value.id}</p>
    <button
      on:click={handleDelete}
      class:pointer-events-none={isDeleting}
      class="hover:text-rose-500 underline cursor-pointer disabled:opacity-50"
    >
      {isDeleting ? "Deleting..." : "Delete Account"}
    </button>
  </div>
{/if}
