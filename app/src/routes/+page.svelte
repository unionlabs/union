<script lang="ts">
import { cn } from "$lib/utilities/shadcn.ts"
import { getContextClient, gql, queryStore } from "@urql/svelte"
import { unionToUnionTransfersQuery } from "$/lib/queries/union-uno"

$: unionUserTransfers = queryStore({
  client: getContextClient(),
  query: unionToUnionTransfersQuery,
  variables: { address: "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv", limit: 50 },
  requestPolicy: "cache-and-network"
})

const reexcute = () => unionUserTransfers.reexecute({ requestPolicy: "network-only" })
</script>

<main
  class={cn(['mt-12 flex min-w-full flex-col items-center justify-center space-y-6 text-white'])}
>
  {#if $unionUserTransfers.fetching}
    Loading...
  {:else if $unionUserTransfers.error}
    Oh no… ${$unionUserTransfers.error.message}
  {:else if typeof $unionUserTransfers?.data?.v0_transfers?.length === 'number'}
    {#each $unionUserTransfers.data.v0_transfers as transfer}
      <pre>{JSON.stringify(transfer, undefined, 2)}</pre>
    {/each}
  {/if}
</main>
