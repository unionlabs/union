<script lang="ts"> 
  import * as THREE from 'three'; 
  import Square from './spinning-logo/square.svelte';
  import { onMount } from 'svelte';

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

		const boxGeometry = new THREE.BoxGeometry(200, 200, 200);
		const edgesGeometry = new THREE.EdgesGeometry(boxGeometry);

		const createTubeLine = (points, thickness) => {
			const path = new THREE.CatmullRomCurve3(points);
			const geometry = new THREE.TubeGeometry(path, 20, thickness, 8, false);
			const material = new THREE.MeshBasicMaterial({ color: 0x000000 });
			const tube = new THREE.Mesh(geometry, material);
			return tube;
		};

		const edges = [];
		for (let i = 0; i < edgesGeometry.attributes.position.count; i += 2) {
			const start = new THREE.Vector3(
				edgesGeometry.attributes.position.getX(i),
				edgesGeometry.attributes.position.getY(i),
				edgesGeometry.attributes.position.getZ(i)
			);
			const end = new THREE.Vector3(
				edgesGeometry.attributes.position.getX(i + 1),
				edgesGeometry.attributes.position.getY(i + 1),
				edgesGeometry.attributes.position.getZ(i + 1)
			);
			const line = createTubeLine([start, end], strokeWidth); // Adjust thickness to match desired size
			edges.push(line);
			scene.add(line);
		}

		const light1 = new THREE.PointLight(0xff80C0, 2, 0);
		light1.position.set(200, 100, 300);
		scene.add(light1);

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

			edges.forEach((edge) => {
				edge.rotation.x = time * 0.5;
				edge.rotation.y = time * 1;
			});

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
