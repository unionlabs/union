<script lang="ts">
  import Chain from "$lib/components/Transfer/Chain.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import {hasFailedExit, isComplete} from "$lib/services/transfer-ucs03-evm";
  import Button from "$lib/components/ui/Button.svelte";
  import Assets from "$lib/components/Transfer/Assets.svelte";
  import Amount from "$lib/components/Transfer/Amount.svelte";
  import Receiver from "$lib/components/Transfer/Receiver.svelte";
  import ShowData from "$lib/components/Transfer/ShowData.svelte";
  import {getQuoteToken} from "$lib/services/transfer-ucs03-evm/quote-token.ts";
  import {Effect} from "effect";
  import {getWethQuoteToken} from "$lib/services/transfer-ucs03-evm/weth-token.ts";
  import {getTransfer} from "$lib/components/Transfer/transfer.svelte.ts";

  const {transfer} = getTransfer()

  //test
  const runthis = async (): Promise<void> => {
    if (!transfer.sourceChain?.chain_id ||
      !transfer.baseToken?.denom ||
      !transfer.channel ||
      !transfer.destinationChain?.rpc_type) {
      console.log('Missing required parameters');
      return;
    }

    const quote = await Effect.runPromise(
      getQuoteToken(
        transfer.sourceChain.chain_id,
        transfer.baseToken.denom,
        transfer.channel,
        transfer.destinationChain.rpc_type
      )
    );

    if (!transfer.ucs03address) return

    const weth = await Effect.runPromise(
      getWethQuoteToken(
        transfer.sourceChain,
        transfer.ucs03address,
        transfer.channel,
      )
    );

    console.log('quote', quote);
    console.log('weth', weth);
  };

  $effect(() => {
    console.log(transfer.sourceChain?.chain_id, transfer.baseToken?.denom, transfer.channel, transfer.sourceChain?.rpc_type)
    runthis()
  })
</script>

<Card class="max-w-md relative flex flex-col gap-2">
  <Chain type="source"/>
  <Chain type="destination"/>
  <Assets/>
  <Amount/>
  <Receiver/>

  <!-- For testing -->
  <ShowData/>
  <!-- For testing -->

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