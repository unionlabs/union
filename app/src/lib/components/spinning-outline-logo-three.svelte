<script lang="ts"> 
  import * as THREE from 'three'; 
  import Square from './spinning-logo/square.svelte';
  import { onMount } from 'svelte';
  import { createCube } from '$lib/three/cube';

	let cubeWidth = 128;
	let gap = 128;
	let cubeCount = 12;
	let logoWidth = cubeWidth * cubeCount + gap * (cubeCount - 1);
	let cubesY = cubeWidth * 2 + gap;
	$: cubeDelta = (20 - cubeWidth) / 2;
	let strokeWidth = 3;

	let threeContainer: HTMLElement;
	let threeCanvas: HTMLCanvasElement;

	onMount(() => {
		const scene = new THREE.Scene();
		const renderer = new THREE.WebGLRenderer({ antialias: true, canvas: threeCanvas, alpha: true });
		const devicePixelRatio = window.devicePixelRatio || 1;
		renderer.setPixelRatio(devicePixelRatio);
		renderer.setClearColor(0x000000, 0);

		const camera = new THREE.PerspectiveCamera(30, 2, 1, 1000);
		camera.position.z = cubeWidth * 3;

		let cubes: Array<THREE.Group> = [];
		for (let x = 0; x <  cubeCount; x++) {
			const cube = createCube(cubeWidth, strokeWidth);
			cube.position.z = -cubeWidth;
			cube.position.x = (x * (cubeWidth + gap));
			cubes.push(cube);
			scene.add(cube);
		}

		function resizeCanvasToDisplaySize() {
			const canvas = renderer.domElement;
			const width = canvas.clientWidth;
			const height = canvas.clientHeight;

			if (canvas.width !== width || canvas.height !== height) {
				renderer.setSize(width, height, false);
				camera.aspect = width / height;
				camera.updateProjectionMatrix();
			}
		}

		const clock = new THREE.Clock();

		function animate(time: number) {
			const secs = clock.getDelta();

			resizeCanvasToDisplaySize();

			camera.position.x += (secs * 100);
			cubes.forEach(cube => {cube.rotation.z += secs * -2})

			renderer.render(scene, camera);
			requestAnimationFrame(animate);
		}

		requestAnimationFrame(animate);
	});
</script>

<div class="relative flex-1">
  <div class="absolute size-full" bind:this={threeContainer}>
    <canvas class="size-full" bind:this={threeCanvas}></canvas>
  </div>
</div>
