<script lang="ts">
import AppleGraph from "#/components/graphs/AppleGraph.svelte"

type BarProps = {
  primary?: boolean
  width: number
  label: string
}

export let bars: Array<BarProps>
</script>

<div class="flex gap-4 w-full">
	<div class="flex-1 grid gap-2">
		{#each bars as { primary, width, label }}
			<AppleGraph {primary} {width} {label} />
		{/each}
	</div>

	<div class="grid grid-cols-1 gap-4">
		<div class="flex flex-col items-center text-accent text-semibold">
			<div class="text-3xl sm:text-5xl">60x</div>
			<div class="text-2xl sm:text-3xl text-gray-200 -mt-2">faster</div>
		</div>
		<!--
		<div class="flex flex-col items-center text-accent-900 text-semibold">
			<div class="text-4xl sm:text-6xl">150x</div>
			<div class="text-2xl sm:text-3xl text-gray-200 -mt-2">cheaper</div>
		</div>
		!-->
	</div>
</div>

