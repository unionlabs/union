<script lang="ts">
import { getLiveLogsState } from "$lib/stores/live.svelte.ts"
import Text from "$lib/components/typography/Text.svelte"
import H6 from "$lib/components/typography/H6.svelte"

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
            <svg xmlns="http://www.w3.org/2000/svg" class="size-5 text-union-accent-500" viewBox="0 0 24 24">
              <path fill="currentColor"
                    d="M16 14q-1.25 0-2.125-.875T13 11t.875-2.125T16 8t2.125.875T19 11t-.875 2.125T16 14m-6 6v-1.9q0-.525.25-1t.7-.75q1.125-.675 2.388-1.012T16 15t2.663.338t2.387 1.012q.45.275.7.75t.25 1V20zm-7-6v-2h8v2zm0-8V4h12v2zm8.1 4H3V8h9q-.35.425-.562.925T11.1 10"/>
            </svg>
            <Text>Someone joined the waitlist</Text>

          {:else if type === "redeem"}
            <svg xmlns="http://www.w3.org/2000/svg" class="size-5 text-union-accent-500" viewBox="0 0 24 24">
              <path fill="currentColor"
                    d="M1 19V5h2v14zm3 0V5h2v14zm3 0V5h1v14zm3 0V5h2v14zm3 0V5h3v14zm4 0V5h1v14zm3 0V5h3v14z"/>
            </svg>
            <Text>A user have redeemed a code</Text>

          {:else if type === "join_queue"}
            <svg xmlns="http://www.w3.org/2000/svg" class="size-5 text-union-accent-500" viewBox="0 0 20 20">
              <path fill="currentColor"
                    d="M8.75 5.25a2.25 2.25 0 1 1-4.5 0a2.25 2.25 0 0 1 4.5 0M4 8a1 1 0 0 0-1 1v4.5a3.5 3.5 0 1 0 7 0V9a1 1 0 0 0-1-1zm5.75-2.75c0 .65-.19 1.255-.52 1.763c.413.048.787.22 1.084.48q.091.006.186.007a2.25 2.25 0 1 0-1.312-4.078c.354.521.562 1.15.562 1.828M9.5 16.855A4.5 4.5 0 0 0 11 13.5V9c0-.364-.098-.706-.268-1H13a1 1 0 0 1 1 1v4.5a3.5 3.5 0 0 1-4.5 3.355M13.75 5.25c0 .65-.19 1.255-.52 1.763c.413.048.787.22 1.084.48q.091.006.186.007a2.25 2.25 0 1 0-1.312-4.078c.354.521.562 1.15.562 1.828m-.25 11.605A4.5 4.5 0 0 0 15 13.5V9c0-.364-.098-.706-.268-1H17a1 1 0 0 1 1 1v4.5a3.5 3.5 0 0 1-4.5 3.355"/>
            </svg>
            <Text>Someone joined the queue</Text>

          {:else if type === "contribution_started"}
            <svg xmlns="http://www.w3.org/2000/svg" class="size-5 text-union-accent-500" viewBox="0 0 24 24">
              <path fill="currentColor"
                    d="M9 16q.425 0 .713-.288T10 15V9q0-.425-.288-.712T9 8t-.712.288T8 9v6q0 .425.288.713T9 16m4.175-.775l3.9-2.6q.35-.225.35-.625t-.35-.625l-3.9-2.6q-.375-.25-.775-.038T12 9.4v5.2q0 .45.4.663t.775-.038M12 22q-2.075 0-3.9-.788t-3.175-2.137T2.788 15.9T2 12t.788-3.9t2.137-3.175T8.1 2.788T12 2t3.9.788t3.175 2.137T21.213 8.1T22 12t-.788 3.9t-2.137 3.175t-3.175 2.138T12 22m0-2q3.35 0 5.675-2.325T20 12t-2.325-5.675T12 4T6.325 6.325T4 12t2.325 5.675T12 20m0-8"/>
            </svg>
            <Text>A user have started their contribution</Text>

          {:else if type === "contribution_submitted"}
            <svg xmlns="http://www.w3.org/2000/svg" class="size-5 text-union-accent-500" viewBox="0 0 24 24">
              <path fill="currentColor"
                    d="M4.25 14q.15.575.363 1.1t.487 1q.225.375.175.8t-.325.7t-.688.263t-.637-.363q-.525-.8-.888-1.65t-.562-1.8q-.1-.4.163-.725T3.025 13t.763.275t.462.725m.85-6.1q-.275.475-.487 1T4.25 10q-.125.45-.462.725T3.025 11t-.687-.3t-.163-.7q.2-.975.575-1.875t.9-1.65q.225-.325.625-.337t.675.262t.325.7t-.175.8m2.775 10.95q.5.3 1.025.525t1.075.375q.425.125.7.45t.275.75t-.3.675t-.7.175q-.925-.2-1.787-.55T6.5 20.375q-.35-.225-.387-.638t.237-.712q.3-.3.725-.35t.8.175m2.15-14.6q-.55.15-1.062.363t-1.013.512q-.4.225-.837.188t-.738-.338t-.275-.7t.375-.625q.825-.525 1.713-.887t1.837-.563q.375-.075.675.175t.3.675t-.275.75t-.7.45m6.05 14.625q.375-.225.813-.187t.737.337t.275.713t-.375.612q-.8.525-1.7.888t-1.85.562q-.4.075-.712-.175t-.313-.675t.288-.75t.712-.45q.575-.15 1.1-.362t1.025-.513m-2.1-14.625q-.425-.125-.7-.45T13 3.05t.3-.675t.675-.175q.95.2 1.85.563t1.7.887q.35.225.375.625t-.25.7q-.3.3-.725.35T16.1 5.15q-.525-.3-1.05-.525t-1.075-.375m5.775 9.725q.125-.425.463-.7t.762-.275t.687.325t.163.725q-.2.95-.587 1.825T20.35 17.5q-.225.325-.625.35t-.675-.25t-.325-.712t.175-.813q.275-.5.488-1.012t.362-1.088M18.9 7.9q-.225-.375-.175-.8t.325-.7t.675-.25t.625.35q.55.8.925 1.675T21.85 10q.075.4-.188.7t-.687.3t-.763-.275T19.75 10q-.15-.575-.362-1.1t-.488-1M11.975 17q-.425 0-.712-.288T10.975 16v-5.125l-1.875 1.9q-.3.3-.712.3t-.713-.3t-.312-.712t.287-.713l3.625-3.65q.275-.275.7-.275t.7.275l3.575 3.575q.3.3.313.725t-.288.725t-.725.3t-.725-.3l-1.85-1.85V16q0 .425-.287.713t-.713.287"/>
            </svg>
            <Text>A user has submitted their contribution</Text>

          {:else if type === "contribution_verified"}
            <svg xmlns="http://www.w3.org/2000/svg" class="size-5 text-union-accent-500" viewBox="0 0 24 24">
              <path fill="currentColor"
                    d="m10.95 12.7l-1.4-1.4q-.3-.3-.7-.3t-.7.3t-.3.713t.3.712l2.1 2.125q.3.3.7.3t.7-.3l4.25-4.25q.3-.3.3-.712t-.3-.713t-.712-.3t-.713.3zM12 21.9q-.175 0-.325-.025t-.3-.075Q8 20.675 6 17.638T4 11.1V6.375q0-.625.363-1.125t.937-.725l6-2.25q.35-.125.7-.125t.7.125l6 2.25q.575.225.938.725T20 6.375V11.1q0 3.5-2 6.538T12.625 21.8q-.15.05-.3.075T12 21.9m0-2q2.6-.825 4.3-3.3t1.7-5.5V6.375l-6-2.25l-6 2.25V11.1q0 3.025 1.7 5.5t4.3 3.3m0-7.9"/>
            </svg>
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