<script lang="ts"> 
  import Square from './spinning-logo/square.svelte';
	let cubeWidth = 64;
	let gap = 32;
	let logoWidth= cubeWidth * 4 + gap * 3;
	let cubesY = cubeWidth * 2 + gap;
	$: cubeDelta = (20 - cubeWidth) / 2;
	let strokeWidth = 4;
</script>


<div class="overflow-hidden max-size-full flex-1">
  <div class="logo-scene max-size-full overflow-hidden size-full">
  <!--<div class="logo-scene max-size-full overflow-hidden size-full" style={`width: ${logoWidth}px; height: ${cubeWidth}px`};}>!-->
    <div class="logo" style={`left: calc(50% - (${logoWidth}px / 2)); top: calc(50% - (${cubeWidth}px / 2)); width: ${logoWidth}px; height: ${cubeWidth}px`};}>
      <Square {strokeWidth} size={cubeWidth} x={0} y={0}/>
      <Square {strokeWidth} size={cubeWidth} x={cubeWidth + gap} y={0}/>
      <Square {strokeWidth} size={cubeWidth} x={(cubeWidth + gap) * 2} y={0}/>
      <Square {strokeWidth} size={cubeWidth} x={(cubeWidth + gap) * 3} y={0}/>
    </div>
  </div>
</div>

<style lang="postcss">

@keyframes spinning-logo {
  0% {transform: rotateY(0deg) translateX(0);}
  10% {transform: rotateY(90deg) translateX(0);}
  20% {transform: rotateY(90deg) translateX(-1000px);}
  30% {transform: rotateY(90deg) translateX(0);}
  80% {transform: rotateY(360deg);}
  100% {transform: rotateY(0deg);}
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
