<script lang="ts">
import { beforeNavigate } from "$app/navigation"
import A from "$lib/components/ui/A.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte"
import { Option } from "effect"

beforeNavigate(() => {
  transferHashStore.reset()
})

let transactionHash = $state("")
</script>

<Sections>
  <Card divided >
    <h2 class="p-4">Find packet / transfer</h2>
    <div class="flex flex-col gap-4 p-4">
      <section>
        Find a transfer or packet based on the hash of the transaction that initiated the transfer.
      </section>
      <section>
        <Input id="transaction-hash" value="" oninput={(e) => {
            // used for conditionally displaying result
            transactionHash = e.currentTarget.value

            // actually make the query
            transferHashStore.startPolling(e.currentTarget.value)
          }} label="Transaction Hash"/>
      </section>
      {#if transactionHash}
      <section>
        {#if Option.isSome(transferHashStore.data)}
          <Label>Packet Hash</Label>
          {transferHashStore.data.value}
          <div class="flex flex-col gap-2 mt-4">
          <A external={false} href={`/explorer/transfers/${transferHashStore.data.value}`}>Go to transfer</A>
          <A external={false} href={`/explorer/packets/${transferHashStore.data.value}`}>Go to packet</A>
          </div>
        {:else}
          <p>No packet or transfer found.</p>
        {/if}
      </section>
      {/if}
    </div>
  </Card>
</Sections>
