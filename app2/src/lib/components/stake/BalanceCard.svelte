<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import Card from "$lib/components/ui/Card.svelte"
import Tabs from "$lib/components/ui/Tabs.svelte"
import TokenBalanceRow from "./TokenBalanceRow.svelte"
import type { Chain, Token } from "@unionlabs/sdk/schema"
import * as O from "effect/Option"

interface Props {
  evmChain: O.Option<Chain>
  uOnEvmToken: O.Option<Token>
  eUOnEvmToken: O.Option<Token>
  uOnEvmBalance: O.Option<bigint>
  eUOnEvmBalance: O.Option<bigint>
}

let { 
  evmChain, 
  uOnEvmToken,
  eUOnEvmToken,
  uOnEvmBalance,
  eUOnEvmBalance
}: Props = $props()

type BalanceTab = "balances" | "rewards" 
let selectedTab = $state<BalanceTab>("balances")
</script>

  <Card divided>
    <div class="px-4 py-2.5 border-b border-zinc-800">
      <Tabs
        items={[
          { id: "balances", label: "Balances" },
          // { id: "rewards", label: "Rewards" },
        ]}
        activeId={selectedTab}
        onTabChange={(id) => selectedTab = id as BalanceTab}
      />
    </div>

  <div class="p-4">
    {#if selectedTab === "balances"}
      <div class="flex flex-col gap-3">
        <!-- U Balance Card -->
        <TokenBalanceRow
          chain={evmChain}
          token={uOnEvmToken}
          balance={uOnEvmBalance}
          symbol="U"
          hoverable={true}
          title="Union Token"
          class="group"
        />

        <!-- eU Balance Card -->
        <TokenBalanceRow
          chain={evmChain}
          token={eUOnEvmToken}
          balance={eUOnEvmBalance}
          symbol="eU"
          hoverable={true}
          title="Staked Union Token"
        />
        
        <!-- Get U Button -->
        <Button
          variant="primary"
          href="/transfer"
          class="w-full h-9 text-sm font-semibold mt-1"
        >
          Get U →
        </Button>
      </div>
    {:else}
      <div class="text-center py-8 text-zinc-500">
        Rewards coming soon...
      </div>
    {/if}
  </div>
</Card>
