<script lang="ts">
  import AppleGraph from '#/components/graphs/AppleGraph.svelte'

  type BarProps = {
  	primary?: boolean,
  	width: number,
  	label: string,
  }

  export let bars: BarProps[];
</script>

<div class="flex gap-8">
	<div class="flex-1 grid gap-4">
		{#each bars as { primary, width, label }}
			<AppleGraph {primary} {width} {label} />
		{/each}
	</div>

	<div class="flex flex-col items-center text-accent text-semibold">
		<div class="text-6xl">60x</div>
		<div class="text-3xl">faster</div>
	</div>
</div>

