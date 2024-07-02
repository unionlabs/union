<script lang="ts">
  export let size: number;
  export let x: number;
  export let y: number;
</script>

<svg class="absolute" style={`top: ${y}; left: ${x}`} width={`${size}px`} height={`${size}px`} viewBox={`0 0 ${size} ${size}`} fill="none" xmlns="http://www.w3.org/2000/svg">
	<rect x=0 y=0 width={size} height={size} stroke="black"/>
</svg>
