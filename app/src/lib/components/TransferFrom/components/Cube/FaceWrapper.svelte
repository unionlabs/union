<script>
export let visible = true
export let width = 0 // Pixel value
export let height = 0 // Pixel value
export let translateZ = 0
export let rotateY = "0deg"
</script>

<div
        class="absolute bg-neutral-900 flex flex-col items-center border-2"
        class:opacity-100={visible}
        class:opacity-0={!visible}
        class:pointer-events-none={!visible}
        style={`width: ${width}px; height: ${height}px; transform: rotateY(${rotateY}) translateZ(${translateZ}px);`}
>
  <slot />
</div>