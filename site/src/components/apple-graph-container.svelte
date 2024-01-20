<script lang="ts">
  import AppleGraph from '#/components/apple-graph.svelte'

  type BarProps = {
  	primary?: boolean,
  	width: number,
  	label: string,
  }

  export let bars: BarProps[];
</script>

<div class="grid grid-5 gap-4 w-full min-w-full">
	<!-- hack???? first one doesn't lmfao -->
	<AppleGraph width={70} label="" />

	{#each bars as { primary, width, label }}
		<AppleGraph {primary} {width} {label} />
	{/each}
</div>

