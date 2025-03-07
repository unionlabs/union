<script lang="ts">
  import Chain from "$lib/components/Transfer/Chain.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import {hasFailedExit, isComplete} from "$lib/services/transfer-ucs03-evm";
  import Button from "$lib/components/ui/Button.svelte";
  import Assets from "$lib/components/Transfer/Assets.svelte";
  import Amount from "$lib/components/Transfer/Amount.svelte";
  import Receiver from "$lib/components/Transfer/Receiver.svelte";
  import {getTransfer} from "../../../routes/transfer/transfer.svelte.ts";

  const {transfer} = getTransfer()

</script>

<Card class="max-w-md relative flex flex-col gap-2">
  <Chain type="source"/>
  <Chain type="destination"/>
  <Assets/>
  <Amount/>
  <Receiver/>

  <Button
          class="mt-2"
          variant="primary"
          onclick={transfer.submit}
          disabled={transfer.state._tag !== "Filling" && !hasFailedExit(transfer.state) && !isComplete(transfer.state)}
  >
    {#if transfer.state._tag !== "Filling" && !hasFailedExit(transfer.state) && !isComplete(transfer.state)}
      Submitting...
    {:else if hasFailedExit(transfer.state)}
      Retry
    {:else}
      Submit
    {/if}
  </Button>
</Card>
{JSON.stringify(transfer.state, null, 2)}