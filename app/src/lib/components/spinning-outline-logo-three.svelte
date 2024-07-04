<script lang="ts"> 
  import * as THREE from 'three'; 
  import Square from './spinning-logo/square.svelte';
  import { onMount } from 'svelte';
  import { createCube } from '$lib/three/cube';

	let cubeWidth = 64;
	let cubeCount = 12;
	let gap = 64;
	let logoWidth = cubeWidth * cubeCount + gap * (cubeCount - 1);
	let cubesY = cubeWidth * 2 + gap;
	$: cubeDelta = (20 - cubeWidth) / 2;
	let strokeWidth = 4;

	let threeContainer: HTMLElement;
	let threeCanvas: HTMLCanvasElement;

	onMount(() => {
		const scene = new THREE.Scene();
		const renderer = new THREE.WebGLRenderer({ antialias: true, canvas: threeCanvas, alpha: true });
		const devicePixelRatio = window.devicePixelRatio || 1;
		renderer.setPixelRatio(devicePixelRatio);
		renderer.setClearColor(0x000000, 0);

		const camera = new THREE.PerspectiveCamera(70, 2, 1, 1000);
		camera.position.z = 400;

		const cube = createCube(strokeWidth);
		scene.add(cube);

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

		function animate(time: number) {
			time *= 0.001; // seconds

			resizeCanvasToDisplaySize();

			cube.rotation.x = time * 2;
			cube.rotation.y = time * 1;

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
