<script lang="ts"> 
  import Square from './spinning-logo/square.svelte';
	let cubeWidth = 128;
	let gap = 64;
	let logoWidth= cubeWidth * 4 + gap * 3;
	let cubesY = cubeWidth * 2 + gap;
	$: cubeDelta = (20 - cubeWidth) / 2;
	let strokeWidth = 16;
</script>


<div class="w-full flex items-center justify-center">
  <div class="logo-scene" style={`width: ${logoWidth}px; height: ${cubeWidth}px`};}>
    <div class="logo" style={`width: ${logoWidth}px; height: ${cubeWidth}px`};}>
      <Square {strokeWidth} size={cubeWidth} x={0} y={0}/>
      <Square {strokeWidth} size={cubeWidth} x={cubeWidth + gap} y={0}/>
      <Square {strokeWidth} size={cubeWidth} x={(cubeWidth + gap) * 2} y={0}/>
      <Square {strokeWidth} size={cubeWidth} x={(cubeWidth + gap) * 3} y={0}/>
    </div>
  </div>
</div>

<style lang="postcss">
@keyframes spinning-logo {
  from {transform: rotateY(0deg);}
  to {transform: rotateY(360deg);}
}


.logo-scene {
  perspective: 1000px;
  position: relative;
}

.logo {
  animation: spinning-logo 8s linear 0s infinite ;
  position: absolute;
  transform-style: preserve-3d;
}
</style>
