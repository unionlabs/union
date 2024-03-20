<script lang="ts">
  import { onMount } from "svelte";
  import { Application } from "@splinetool/runtime";
  export let model: string;

  let canvas: HTMLCanvasElement;

  onMount(() => {
    const app = new Application(canvas);
    app.load(model);
  });
</script>

<div class="w-full aspect-h-1 aspect-w-1 overflow-hidden rounded">
  <canvas class="overflow-hidden" bind:this={canvas}></canvas>
</div>
