<script lang="ts">
  import { onMount } from "svelte";
  import { Application } from "@splinetool/runtime";
  import cube from "#/spline/cube-responsive.splinecode";

  let canvas: HTMLCanvasElement;

  onMount(() => {
    const app = new Application(canvas);
    app.load(cube);
  });
</script>

<div class="w-full aspect-h-1 aspect-w-1">
  <canvas bind:this={canvas}></canvas>
</div>
