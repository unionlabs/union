<script lang="ts">
  export let x: number;
  export let y: number;
</script>

<svelte:options namespace="svg" />

<rect
  x="0"
  y="0"
  width="100%"
  height="100%"
  stroke="#ccc"
  style="opacity: 0.5"
  fill="none"
  stroke-width="2"
/>

{#each { length: 9 } as _, index}
  {#if index < 6}
    <path
      d="M{(index + 1) * 200} 0 L{(index + 1) * 200} 1802"
      class="grid-line"
    />
  {/if}
  <path
    d="M0 {(index + 1) * 200} L1400 {(index + 1) * 200} "
    class="grid-line"
  />
{/each}

<path
  style="transform: translateX({x + 200}px)"
  d="M0 0 L0 1800"
  class="grid-line-xy"
/>
<path
  style="transform: translateY({y}px)"
  d="M0 400 L1400 400"
  class="grid-line-xy"
/>
<rect
  x="0"
  y="0"
  width="100%"
  height="100%"
  stroke="#999"
  fill="none"
  stroke-width="2"
/>

<style>
  .grid-line {
    stroke: #ccc;
    opacity: 0.5;
    stroke-width: 2;
  }

  .grid-line-xy {
    stroke: tomato;
    stroke-width: 2;
  }
</style>
