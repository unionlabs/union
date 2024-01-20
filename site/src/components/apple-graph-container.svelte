<script lang="ts">
  import { afterUpdate, onMount, tick } from 'svelte'
  import AppleGraph from '#/components/apple-graph.svelte'

  type BarProps = {
  	primary: boolean,
  	width: number,
  	label: string,
  }

  export let bars: BarProps[];

  let div: HTMLElement;

  onMount(() => {})
</script>

<div class="grid grid-5 gap-4 w-full min-w-full" bind:this={div}>
	<!-- hack???? first one doesnt lmfao -->
	<AppleGraph width={70} label="" element={div}/>

	{#each bars as { primary, width, label }}
		<AppleGraph {primary} {width} {label}  element={div}/>
	{/each}
</div>

