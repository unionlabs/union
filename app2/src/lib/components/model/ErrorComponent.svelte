<script lang="ts">
import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { TimeoutException, UnknownException } from "effect/Cause"
import type { ParseError } from "effect/ParseResult"
import { slide } from "svelte/transition"
import Button from "$lib/components/ui/Button.svelte"

interface Props {
  error: UnknownException | HttpClientError | ParseError | TimeoutException
}

let { error }: Props = $props()
let showDetails = $state(false)

function getUserFriendlyMessage(error: Props["error"]): string {
  switch (error._tag) {
    case "RequestError":
      return "Unable to connect to the server. Please check your internet connection."
    case "ResponseError":
      return "The server encountered an error processing your request."
    case "ParseError":
      return "There was an error processing the data from the server."
    case "TimeoutException":
      return "The request timed out because it took too long. Please try again."
    case "UnknownException":
      return "An unexpected error occurred."
    default:
      return "Something went wrong. Please try again later."
  }
}
</script>

<div class="p-4 rounded bg-red-500 overflow-hidden flex flex-col">
  <div class="flex justify-between gap-2">
    <div>
      <h3 class="text-xl font-bold">Error</h3>
      <p>{getUserFriendlyMessage(error)}</p>
    </div>
    <Button 
      variant="secondary" 
      class="self-start mt-2"
      onclick={() => showDetails = !showDetails}
    >
      {showDetails ? "Hide Details ↑" : "Show Details ↓"}
    </Button>
  </div>

  {#if showDetails}
    <div in:slide out:slide|local={{delay: 0}}>
      <section class="mt-4">
        <h3 class="text-lg font-bold">Error Type</h3>
        <pre>{error._tag}</pre>
        <p class="mt-2">{error.message}</p>
      </section>

      {#if error.cause}
        <section class="mt-4">
          <h3 class="text-lg font-bold">Cause</h3>
          <pre>{error.cause}</pre>
        </section>
      {/if}

      {#if error.stack}
        <section class="mt-4">
          <h3 class="text-lg font-bold">Stack</h3>
          <pre class="text-sm">{error.stack}</pre>
        </section>
      {/if}

      <section class="mt-4">
        <h3 class="text-lg font-bold">Additional Details</h3>
        {#if error._tag === "RequestError"}
          <p>{error.description}</p>
          <p>Method and URL: {error.methodAndUrl}</p>
        {:else if error._tag === "ResponseError"}
          <p>{error.description}</p>
          <p>Method and URL: {error.methodAndUrl}</p>
        {:else if error._tag === "ParseError"}
          <p>Actual data that was parsed:</p>
          <pre class="text-sm">{JSON.stringify(error.issue.actual, null, 2)}</pre>
        {:else if error._tag === "UnknownException"}
          <p>This is an unknown exception. Full details here:</p>
          <pre class="text-sm">{JSON.stringify(error, null, 2)}</pre>
        {/if}
      </section>
    </div>
  {/if}
</div>

