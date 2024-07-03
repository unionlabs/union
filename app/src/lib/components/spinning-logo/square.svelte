<script lang="ts">
  export let size: number;
  export let x: number;
  export let y: number;
  export let strokeWidth: number;
</script>


<!-- front !-->
<div class="cube-container" style={`top: ${y}px; left: ${x}px; width: ${size}px; height: ${size}px;`}> 
	<svg class="absolute" stroke-width={strokeWidth} style={`transform: translateZ(-${size/2}px)`} width={`${size}px`} height={`${size}px`} viewBox={`0 0 ${size} ${size}`} fill="none" xmlns="http://www.w3.org/2000/svg">
		<rect x=0 y=0 width={size} height={size} stroke="black"/>
	</svg>

	<!-- left !-->
	<svg class="absolute" stroke-width={strokeWidth} style={`transform: rotateY(90deg) translateZ(-${size/2}px)`} width={`${size}px`} height={`${size}px`} viewBox={`0 0 ${size} ${size}`} fill="none" xmlns="http://www.w3.org/2000/svg">
		<rect x=0 y=0 width={size} height={size} stroke="black"/>
	</svg>

	<!-- top !-->
	<svg class="absolute" stroke-width={strokeWidth} style={`transform: rotateX(-90deg) translateZ(-${size/2}px)`} width={`${size}px`} height={`${size}px`} viewBox={`0 0 ${size} ${size}`} fill="none" xmlns="http://www.w3.org/2000/svg">
		<rect x=0 y=0 width={size} height={size} stroke="black"/>
	</svg>

	<!-- back !-->
	<svg class="absolute" stroke-width={strokeWidth} style={`transform: translateZ(${size/2}px)`} width={`${size}px`} height={`${size}px`} viewBox={`0 0 ${size} ${size}`} fill="none" xmlns="http://www.w3.org/2000/svg">
		<rect x=0 y=0 width={size} height={size} stroke="black"/>
	</svg>

	<!-- right !-->
	<svg class="absolute" stroke-width={strokeWidth} style={`transform: rotateY(-90deg) translateZ(-${size/2}px)`} width={`${size}px`} height={`${size}px`} viewBox={`0 0 ${size} ${size}`} fill="none" xmlns="http://www.w3.org/2000/svg">
		<rect x=0 y=0 width={size} height={size} stroke="black"/>
	</svg>

	<!-- bottom !-->
	<svg class="absolute" stroke-width={strokeWidth} style={`transform: rotateX(90deg) translateZ(-${size/2}px)`} width={`${size}px`} height={`${size}px`} viewBox={`0 0 ${size} ${size}`} fill="none" xmlns="http://www.w3.org/2000/svg">
		<rect x=0 y=0 width={size} height={size} stroke="black"/>
	</svg>
</div>


<style lang="postcss">

@keyframes rotating-cube {
  0% {transform: rotateZ(0deg);}
  100% {transform: rotateZ(360deg);}
}

.cube-container {
	position: absolute;
	transform-style: preserve-3d;
  animation: rotating-cube 8s linear 0s infinite ;
}
</style>
