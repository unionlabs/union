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
	let strokeWidth = 2.5;

	let threeContainer: HTMLElement;
	let threeCanvas: HTMLCanvasElement;


	onMount(() => {
		const scene = new THREE.Scene();
		const renderer = new THREE.WebGLRenderer({ antialias: true, canvas: threeCanvas, alpha: true });
		const devicePixelRatio = window.devicePixelRatio || 1;
		renderer.setPixelRatio(devicePixelRatio);
		renderer.setClearColor(0x000000, 0);

		const camera = new THREE.PerspectiveCamera(32, 2, 1, 2000);

		const startCameraZ = cubeWidth * 3;
		
		camera.position.z = startCameraZ;

		let cubes: Array<THREE.Group> = [];
		for (let x = 0; x <  cubeCount; x++) {
			const cube = createCube(cubeWidth, strokeWidth);
			cube.position.z = 0;
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

		let animationState: "SLIDING_RIGHT" | "ROTATING_RIGHT" | "SLIDING_LEFT" | "ROTATING_LEFT" = "SLIDING_RIGHT";

		function animate(time: number) {
			const secs = clock.getDelta() * 2;

			resizeCanvasToDisplaySize();

			if (animationState === "SLIDING_RIGHT") {
				camera.position.x += (secs * 150);
				if (camera.position.x >= logoWidth + cubeWidth) {
					animationState = "ROTATING_RIGHT";
				}
			} 
			else if (animationState === "ROTATING_RIGHT") {
				const rotating = camera.rotation.y < Math.PI/2;
				const translating = camera.position.z > 0;
				if (!rotating && !translating) {
					animationState = "SLIDING_LEFT";
				}
				if (rotating) {
					camera.rotation.y += secs * 0.1
				}
				if (translating) {
					camera.position.z -= secs * 30;
				}
			}
			else if (animationState === "SLIDING_LEFT") {
				if (camera.position.x > -cubeWidth) {
					camera.position.x -= (secs * 150);
				} else {
					animationState = "ROTATING_LEFT";
				}
			}

			else if (animationState === "ROTATING_LEFT") {
				const rotating = camera.rotation.y > 0;
				const translating = camera.position.z < startCameraZ;
				if (!rotating && !translating) {
					animationState = "SLIDING_RIGHT";
				}
				if (rotating) {
					camera.rotation.y -= secs * 0.1
				}
				if (translating) {
					camera.position.z += secs * 30;
				}
			}

			cubes.forEach(cube => {cube.rotation.z += secs * -2})

			renderer.render(scene, camera);
			requestAnimationFrame(animate);
			// console.log(clock.elapsedTime);
		}

		requestAnimationFrame(animate);
	});
</script>

<div class="relative flex-1">
  <div class="absolute size-full" bind:this={threeContainer}>
    <canvas class="size-full" bind:this={threeCanvas}></canvas>
  </div>
</div>
