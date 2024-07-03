<script lang="ts"> 
  import * as THREE from 'three'; 
  import Square from './spinning-logo/square.svelte';
  import { onMount } from 'svelte';
	let cubeWidth = 64;
	let cubeCount = 12;
	let gap = 64;
	let logoWidth= cubeWidth * cubeCount + gap * (cubeCount - 1);
	let cubesY = cubeWidth * 2 + gap;
	$: cubeDelta = (20 - cubeWidth) / 2;
	let strokeWidth = 4;

	let threeContainer: HTMLElement;
	let threeCanvas: HTMLCanvasElement;

	onMount(() => {

    const scene = new THREE.Scene();
    const renderer = new THREE.WebGLRenderer({ antialias: true, canvas: threeCanvas, alpha: true});

    const devicePixelRatio = window.devicePixelRatio || 1;
    renderer.setPixelRatio(devicePixelRatio);

    renderer.setClearColor( 0x000000, 0 ); // the default

    // There's no reason to set the aspect here because we're going
    // to set it every frame anyway so we'll set it to 2 since 2
    // is the the aspect for the canvas default size (300w/150h = 2)
    const  camera = new THREE.PerspectiveCamera(70, 2, 1, 1000);
    camera.position.z = 400;

    const boxGeometry = new THREE.BoxGeometry(200, 200, 200);
    const edgesGeometry = new THREE.EdgesGeometry(boxGeometry);
    const material = new THREE.LineBasicMaterial({
      color: 0x000000,
    });
    const mesh = new THREE.LineSegments(edgesGeometry, material);

    // const mesh = new THREE.Mesh(edgesGemoetry, material);
    scene.add(mesh);


    const light1 = new THREE.PointLight(0xff80C0, 2, 0);
    light1.position.set(200, 100, 300);
    scene.add(light1);

    function resizeCanvasToDisplaySize() {
      const canvas = renderer.domElement;
      const width = canvas.clientWidth;
      const height = canvas.clientHeight;

      if (canvas.width !== width ||canvas.height !== height) {
        // you must pass false here or three.js sadly fights the browser
        renderer.setSize(width, height, false);
        camera.aspect = width / height;
        camera.updateProjectionMatrix();

        // set render target sizes here
      }
    }

    function animate(time: number) {
      time *= 0.001;  // seconds

      resizeCanvasToDisplaySize();

      mesh.rotation.x = time * 0.5;
      mesh.rotation.y = time * 1;

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
