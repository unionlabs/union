<script lang="ts">
import { goto } from "$app/navigation"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { getChain } from "@unionlabs/sdk/schema"
import type { DustWithdrawal } from "@unionlabs/sdk/schema/stake"
import { Option, pipe } from "effect"
import DateTimeComponent from "../ui/DateTimeComponent.svelte"
import ChainComponent from "./ChainComponent.svelte"

interface Props {
  item: DustWithdrawal
}

const { item }: Props = $props()

const sourceChain = $derived(pipe(
  chains.data,
  Option.flatMap(data => getChain(data, item.source_chain.universal_chain_id)),
))

const destinationChain = $derived(pipe(
  chains.data,
  Option.flatMap(data => getChain(data, item.destination_chain.universal_chain_id)),
))

const handleClick = () => {
  goto(`/explorer/dust/${item.packet_hash}`)
}

const status = $derived(
  pipe(
    { dustWithdraw: item.dust_withdraw_success, delivery: item.delivery_success },
    ({ dustWithdraw, delivery }) => {
      // Check if either is explicitly false (error state)
      if (Option.isSome(dustWithdraw) && !dustWithdraw.value) {
        return "failure"
      }
      if (Option.isSome(delivery) && !delivery.value) {
        return "failure"
      }

      // Both are true = success
      if (Option.isSome(dustWithdraw) && dustWithdraw.value && Option.isSome(delivery) && delivery.value) {
        return "success"
      }

      // Otherwise pending (includes cases where dust_withdraw is true and delivery is null)
      return "pending"
    },
  )
)

type StatusConfigType = {
  bg: string
  icon: string
  type: "checkmark" | "warning" | "spinner"
}

const statusConfig = $derived<StatusConfigType>(
  status === "success"
    ? { bg: "bg-accent/20 border-accent/40", icon: "text-accent", type: "checkmark" }
    : status === "failure"
    ? { bg: "bg-red-500/20 border-red-500/40", icon: "text-red-400", type: "warning" }
    : { bg: "bg-orange-500/20 border-orange-500/40", icon: "text-orange-400", type: "spinner" }
)
</script>

{#if Option.isSome(chains.data)}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex justify-between gap-8 px-4 py-3 h-16 cursor-pointer hover:bg-zinc-50 dark:hover:bg-zinc-900 transition-colors duration-75 items-center border-b border-zinc-800 last:border-b-0"
    onclick={handleClick}
  >
    <div>
      <div class="flex items-center gap-2 font-semibold">
        <span class="text-zinc-300 text-xs uppercase font-mono">
          Dust
        </span>
        {#if Option.isSome(sourceChain)}
          <TokenComponent
            showWrapping={false}
            chain={sourceChain.value}
            showIcon={false}
            denom={item.quote_token}
            amount={item.quote_amount as any}
          />
        {/if}
      </div>
      <div class="flex items-center gap-1 text-zinc-400 text-sm">
        {#if Option.isSome(sourceChain)}
          <ChainComponent
            class="font-normal"
            chain={sourceChain.value}
            withToken={item.quote_token}
          />
        {/if}
        {#if Option.isSome(destinationChain)}
          <span class="text-zinc-500">→</span>
          <ChainComponent
            class="font-normal"
            chain={destinationChain.value}
            withToken={item.quote_token}
          />
        {/if}
      </div>
    </div>
    <div class="flex items-center gap-2">
      <DateTimeComponent
        class="text-sm text-zinc-400 hidden md:block"
        value={item.dust_withdraw_send_timestamp}
        showSeconds={false}
      />
      <div class="size-6 rounded border {statusConfig.bg} flex items-center justify-center flex-shrink-0">
        {#if statusConfig.type === "spinner"}
          <div class="w-3 h-3 border border-orange-400 border-t-transparent rounded-full animate-spin">
          </div>
        {:else if statusConfig.type === "checkmark"}
          <svg
            class="w-3 h-3 {statusConfig.icon}"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            />
          </svg>
        {:else if statusConfig.type === "warning"}
          <svg
            class="w-3 h-3 {statusConfig.icon}"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 9v2m0 4h.01"
            />
          </svg>
        {/if}
      </div>
    </div>
  </div>
{/if}

