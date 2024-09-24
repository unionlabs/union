<script lang="ts">
import { getLiveLogsState } from "$lib/stores/live.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import H6 from "$lib/components/typography/H6.svelte"
import Icon from "@iconify/svelte"

const liveLogs = getLiveLogsState()

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const time = date.toLocaleString("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false
  })
  const datePart = date.toLocaleString("en-GB", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric"
  })
  return `${time}`
}
</script>

<div class="fixed bottom-0 inset-x-0 bg-black">
  <div class="flex items-center w-full h-8 relative">

    <div class="absolute left-0 h-full items-center z-10 hidden sm:flex">
      <svg class="h-full w-auto" viewBox="0 0 101.145676 40" version="1.1" xmlns="http://www.w3.org/2000/svg"
           xmlns:xlink="http://www.w3.org/1999/xlink">
        <g id="Page-1" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
          <polygon id="Rectangle" fill="#A0ECFD" points="0 0.173139069 61.0753141 -4.10227408e-14 101.145676 40 0 40">
          </polygon>
        </g>
      </svg>
      <div class="absolute  left-1.5 flex items-center gap-1.5">
        <div class="blinking-dot h-1.5 w-1.5 bg-black rounded-full"></div>
        <H6 class="!text-black">Live</H6>
      </div>


    </div>

    <div class="flex h-full w-full items-center gap-3 sm:gap-6 overflow-x-scroll hide-scrollbar whitespace-nowrap px-4 sm:pl-24 text-white">
      {#each liveLogs.data as item, i}
        {@const type = item.message.type}

        <div class="flex items-center gap-1.5 text-xs sm:text-sm has-tooltip">
          {#if type === "join_waitlist"}
            <Icon icon="material-symbols-light:patient-list" class="size-5 text-union-accent-500"/>
            <Text>Someone joined the waitlist</Text>

          {:else if type === "redeem"}
            <Icon icon="material-symbols-light:barcode" class="size-5 text-union-accent-500"/>
            <Text>A user have redeemed a code</Text>

          {:else if type === "join_queue"}
            <Icon icon="mdi:human-queue" class="size-5 text-union-accent-500"/>
            <Text>Someone joined the queue</Text>

          {:else if type === "contribution_started"}
            <Icon icon="material-symbols-light:not-started-outline-rounded" class="size-5 text-union-accent-500"/>
            <Text>A user have started their contribution</Text>

          {:else if type === "contribution_submitted"}
            <Icon icon="material-symbols-light:arrow-upload-ready-rounded" class="size-5 text-union-accent-500"/>
            <Text>A user has submitted their contribution</Text>

          {:else if type === "contribution_verified"}
            <Icon icon="material-symbols-light:verified-user-outline-rounded" class="size-5 text-union-accent-500"/>
            <Text>A contribution just verified</Text>

          {/if}
        </div>

        {#if i !== liveLogs.data.length - 1}
          <div class="rounded-full h-1 min-w-1 bg-white"></div>
        {/if}
      {/each}
    </div>
  </div>
</div>


<style>
    .hide-scrollbar {
        &::-webkit-scrollbar {
            display: none;
        }

        -ms-overflow-style: none;
        scrollbar-width: none;
        overflow: auto;
    }

    .hide-scrollbar:hover {
        &::-webkit-scrollbar {
            display: auto;
        }

        -ms-overflow-style: auto;
        scrollbar-width: auto;
    }

    .blinking-dot {
        animation: blink 1s infinite;
    }

    @keyframes blink {
        0%, 100% {
            opacity: 1;
        }
        50% {
            opacity: 0;
        }
    }
</style>