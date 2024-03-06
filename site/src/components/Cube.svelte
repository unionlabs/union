<script lang="ts">
  import { onMount } from 'svelte';
  import { Application } from '@splinetool/runtime';
  import cube from '#/spline/cube.splinecode'; 

  let canvas: HTMLCanvasElement;

  onMount(() => {
    const app = new Application(canvas);
    app.load(cube);
  })
</script>

<canvas bind:this={canvas}></canvas>
