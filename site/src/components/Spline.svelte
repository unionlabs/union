<script lang="ts">
import { Application } from "@splinetool/runtime"
import { onMount } from "svelte"
export let model: string

let canvas: HTMLCanvasElement

onMount(() => {
  const app = new Application(canvas)
  app.load(model)
})
</script>

<div class="size-full aspect-h-1 aspect-w-2 overflow-hidden rounded-t">
  <canvas
    class="overflow-hidden size-auto"
    bind:this={canvas}
  ></canvas>
</div>
