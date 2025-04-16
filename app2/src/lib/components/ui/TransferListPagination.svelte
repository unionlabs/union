<script lang="ts">
import { Option } from "effect"
import type { TransferList } from "@unionlabs/sdk/schema"
import Button from "./Button.svelte"
import DateTimeComponent from "./DateTimeComponent.svelte"

type Props = {
  data: Option.Option<typeof TransferList.Type>
  onLive: () => void
  onPrevPage: () => void
  onNextPage: () => void
}

const { data, onLive, onPrevPage, onNextPage }: Props = $props()
</script>

<div class="flex gap-4 md:gap-6">
  <Button onclick={onLive}>LIVE</Button>
  <div class="rounded shadow flex">
    <button
      onclick={onPrevPage}
      class="cursor-pointer border-l border-t border-b bg-zinc-700 border-zinc-600 h-9 w-10 rounded-tl rounded-bl"
    >
      ←
    </button>
    <div
      class="bg-zinc-900 border-t border-b border-zinc-800 flex items-center text-sm justify-center px-4 w-[180px]"
    >
      {#if Option.isSome(data) && data.value.length > 0}
        <DateTimeComponent value={data.value[0].transfer_send_timestamp} />
      {/if}
    </div>
    <button
      onclick={onNextPage}
      class="cursor-pointer border-r border-t border-b bg-zinc-700 border-zinc-600 h-9 w-10 rounded-tr rounded-br"
    >
      →
    </button>
  </div>
</div>
