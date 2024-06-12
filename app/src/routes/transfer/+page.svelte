<script lang="ts">
import ChainsGate from '$lib/components/chains-gate.svelte'
import WalletGate from '$lib/components/wallet-gate.svelte';
import TransferForm from './(components)/transfer-form.svelte';
import { cn } from "$lib/utilities/shadcn.ts"
import * as Card from "$lib/components/ui/card/index.ts"
</script>

<svelte:head>
  <title>Union | Send</title>
</svelte:head>
<main
  class="overflow-scroll flex flex-col gap-4 items-center size-full px-0 sm:px-3 max-h-full sm:py-8"
>
  <Card.Root class={cn('max-w-[490px] w-full')}>
    <Card.Header class="flex flex-row w-full items-center gap-x-2">
      <Card.Title tag="h1" class="flex-1 font-bold text-2xl">Transfer</Card.Title>
    </Card.Header>
    <ChainsGate let:chains>
      <WalletGate let:userAddr>
        <TransferForm {chains} {userAddr} />
      </WalletGate>
    </ChainsGate>
  </Card.Root>
</main>
