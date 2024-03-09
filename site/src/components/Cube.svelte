<script lang="ts">
  import { onMount } from 'svelte';
  import { Application } from '@splinetool/runtime';
  import cube from '#/spline/cube-responsive.splinecode'; 

  let canvas: HTMLCanvasElement;

  onMount(() => {
    const app = new Application(canvas);
    app.load(cube);
  })
</script>

<div class="w-full aspect-h-1 aspect-w-1 bg-yellow-500">
  <canvas class="bg-red-500" bind:this={canvas}></canvas>
</div>
