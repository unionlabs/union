<script lang="ts"> 
	let cubeWidth = 15;
	$: cubeDelta = (20 - cubeWidth) / 2;
</script>


<div class="logo-scene">
  <div class="flex flex-col gap-20 items-center justify-center min-h-screen mx-6 spinning-animation">
  	<svg width="500px" height="500px" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
  		<rect x={60 + cubeDelta } y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<rect x={40 + cubeDelta} y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<rect x={20 + cubeDelta} y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<rect x = {cubeDelta} y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<path transform={`translate(20, ${-cubeDelta})`} d="M50 30C50 18.9543 41.0457 10 30 10C18.9543 10 10 18.9543 10 30"  stroke="black" stroke-width={cubeWidth}/>
  		<path transform={`translate(-20, ${cubeDelta})`} d="M30 50C30 61.0457 38.9543 70 50 70C61.0457 70 70 61.0457 70 50" stroke="black" stroke-width={cubeWidth}/>
  	</svg>
  	<label class="sr-only" for="range-slider">Thickness range:</label>
  	<input id="range-slider" class="w-full max-w-80" type=range min=0 max=20 step=0.01 bind:value={cubeWidth} />
  </div>
</div>

<div class="logo-scene logo-scene-2">
  <div class="flex flex-col gap-20 items-center justify-center min-h-screen mx-6 spinning-animation">
  	<svg width="500px" height="500px" viewBox="0 0 80 80" fill="none" xmlns="http://www.w3.org/2000/svg">
  		<rect x={60 + cubeDelta } y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<rect x={40 + cubeDelta} y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<rect x={20 + cubeDelta} y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<rect x = {cubeDelta} y={30 + cubeDelta} width={cubeWidth} height={cubeWidth} stroke="black"/>
  		<path transform={`translate(20, ${-cubeDelta})`} d="M50 30C50 18.9543 41.0457 10 30 10C18.9543 10 10 18.9543 10 30"  stroke="black" stroke-width={cubeWidth}/>
  		<path transform={`translate(-20, ${cubeDelta})`} d="M30 50C30 61.0457 38.9543 70 50 70C61.0457 70 70 61.0457 70 50" stroke="black" stroke-width={cubeWidth}/>
  	</svg>
  	<label class="sr-only" for="range-slider">Thickness range:</label>
  	<input id="range-slider" class="w-full max-w-80" type=range min=0 max=20 step=0.01 bind:value={cubeWidth} />
  </div>
</div>

<style lang="postcss">


@keyframes spinning-logo {
  from {transform: rotateY(0deg);}
  to {transform: rotateY(360deg);}
}

.logo-scene {
  perspective: 1000px;
  padding: 32px;
  position: relative;
}

.spinning-animation {
  animation: spinning-logo 5s linear 2s infinite ;
  position: absolute;
}


</style>
