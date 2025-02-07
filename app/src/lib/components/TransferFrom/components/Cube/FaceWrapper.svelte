<script>
export let width = 0 // Pixel value
export let height = 0 // Pixel value
export let translateZ = 0
export let rotateY = "0deg"
</script>

<div
        class="absolute bg-muted flex flex-col items-center border-2 h-full overflow-hidden"
        style={`width: ${width}px; height: ${height}px; transform: rotateY(${rotateY}) translateZ(${translateZ}px);`}
>
  <slot />
</div>