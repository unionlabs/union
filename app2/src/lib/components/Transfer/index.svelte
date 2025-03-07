<script lang="ts">
  import Chain from "$lib/components/Transfer/Chain.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import {hasFailedExit, isComplete, nextState, TransferSubmission} from "$lib/services/transfer-ucs03-evm";
  import Button from "$lib/components/ui/Button.svelte";
  import Assets from "$lib/components/Transfer/Assets.svelte";
  import {Option} from "effect";
  import {chains} from "$lib/stores/chains.svelte.ts";
  import Amount from "$lib/components/Transfer/Amount.svelte";
  import Receiver from "$lib/components/Transfer/Receiver.svelte";
  import {getTransfer} from "../../../routes/transfer/transfer.svelte.ts";

  const {transfer} = getTransfer()

  let transferState = $state<TransferSubmission>(TransferSubmission.Filling())

  const submit = async () => {
    if (Option.isNone(chains.data)) return
    if (!transfer.sourceChain) return
    transferState = await nextState(transferState, transfer.args, transfer.sourceChain)
    while (!hasFailedExit(transferState)) {
      transferState = await nextState(transferState, transfer.args, transfer.sourceChain)
      if (isComplete(transferState)) break
    }
  }

  $effect(() => {

    console.log(transfer.derivedReceiver)
  })
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
          onclick={submit}
          disabled={transferState._tag !== "Filling" && !hasFailedExit(transferState) && !isComplete(transferState)}
  >
    {#if transferState._tag !== "Filling" && !hasFailedExit(transferState) && !isComplete(transferState)}
      Submitting...
    {:else if hasFailedExit(transferState)}
      Retry
    {:else}
      Submit
    {/if}
  </Button>
</Card>
{JSON.stringify(transferState, null, 2)}