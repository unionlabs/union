<script lang="ts">
import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { TimeoutException, UnknownException } from "effect/Cause"
import type { ParseError } from "effect/ParseResult"

interface Props {
  error: UnknownException | HttpClientError | ParseError | TimeoutException
}

let { error }: Props = $props()
</script>

<div class="p-4 rounded m-4 bg-red-500 overflow-auto flex flex-col gap-4">
  <section>
    <h3 class="text-xl font-bold">{error._tag}</h3>
    <pre>{error.message}</pre>
  </section>

  {#if error.cause}
    <section>
      <h3 class="text-xl font-bold">Cause</h3>
      <pre>{error.cause}</pre>
    </section>
  {/if}

  {#if error.stack}
    <section>
      <h3 class="text-xl font-bold">Stack</h3>
      <pre>{error.stack}</pre>
    </section>
  {/if}


  <section>
    <h3 class="text-xl font-bold">Additional Details</h3>
    {#if error._tag === "RequestError"}
      {error.description}
      Mehtod and URL: {error.methodAndUrl}
    {:else if error._tag === "ResponseError"}
      {error.description}
      Mehtod and URL: {error.methodAndUrl}
    {:else if error._tag === "ParseError"}
      Actual data that was parsed:
      <pre>{JSON.stringify(error.issue.actual, null, 2)}</pre>
    {:else if error._tag === "UnknownException"}
      This is an unknown exception. Full details here:
      <pre>{JSON.stringify(error, null, 2)}</pre>
    {/if}
  </section>
</div>

