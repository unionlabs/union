<script lang="ts">
import StatsBarStat from "$lib/components/stats-bar-stat.svelte"
import { statsQuery, transfersAddressQuery, transfersPerDayQuery } from "$lib/queries/stats.ts"
import PixelGraph from "../(components)/pixel-graph.svelte"
import { userTime } from "$lib/utilities/user-time.ts"
import SpinningOutlineLogoThree from "$lib/components/spinning-outline-logo-three.svelte"
import { Separator } from "$lib/components/ui/separator"
import { deviceWidth, supportsWebGL } from "$lib/utilities/device.ts"
import { onMount } from "svelte"

export let addresses: Array<string>

$: console.log(addresses)

const statsData = statsQuery()
const transfersPerDayData = transfersPerDayQuery(30)
const transfersPerAddressData = transfersAddressQuery(addresses)

$: console.log($transfersPerAddressData.data)

let show = 0
let interval: any

onMount(() => {
  interval = setInterval(() => {
    show = (show + 1) % 2
  }, 5000)
  return () => {
    clearInterval(interval)
  }
})
</script>

<div class="bg-muted dark:bg-background border-b flex">
    <div class="w-full flex flex-1">
        {#if addresses}
            <StatsBarStat blink={true} label={"Total Transfers"} value={$transfersPerAddressData?.data?.count || 0}/>
        {:else}
            {#if $deviceWidth > 888 || show === 0}
                <StatsBarStat blink={true} label={"Total Transfers"} value={$statsData?.data?.total_transfers || 0}/>
                <Separator orientation="vertical"/>
                <StatsBarStat blink={true} label="Total Packets" value={$statsData?.data?.total_packets || 0}/>
            {/if}
            {#if $deviceWidth > 888}
                <Separator orientation="vertical"/>
            {/if}
            {#if $deviceWidth > 888 || show === 1}
                <StatsBarStat blink={false} label="Metrics" value={$userTime}>
                    {#if $transfersPerDayData.data}
                        <div class="ml-6 flex items-end">
                            <PixelGraph data={$transfersPerDayData.data}/>
                        </div>
                    {/if}
                </StatsBarStat>
            {/if}
        {/if}
        {#if $deviceWidth > 888 && $supportsWebGL}
            <Separator orientation="vertical"/>
            <SpinningOutlineLogoThree/>
        {/if}
    </div>
</div>


