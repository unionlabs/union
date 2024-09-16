import * as glMatrix from "gl-matrix"

let state: "start" | "rotating" | "main"

let canvas: HTMLCanvasElement
let mouseX = 0
let mouseY = 0
let targetMouseX = 0
let targetMouseY = 0
let displayWidth = 1000
let displayHeight = 1000
const RETINA_ENABLED = false

// Perlin noise implementation
class PerlinNoise {
  constructor() {
    this.permutation = new Array(512)
    this.p = new Array(256).fill(0).map((_, i) => i)
    for (let i = 255; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[this.p[i], this.p[j]] = [this.p[j], this.p[i]]
    }
    for (let i = 0; i < 512; i++) {
      this.permutation[i] = this.p[i & 255]
    }
  }

  fade(t) {
    return t * t * t * (t * (t * 6 - 15) + 10)
  }

  lerp(t, a, b) {
    return a + t * (b - a)
  }

  grad(hash, x, y, z) {
    const h = hash & 15
    const u = h < 8 ? x : y
    const v = h < 4 ? y : h === 12 || h === 14 ? x : z
    return ((h & 1) === 0 ? u : -u) + ((h & 2) === 0 ? v : -v)
  }

  noise(x, y, z) {
    const X = Math.floor(x) & 255
    const Y = Math.floor(y) & 255
    const Z = Math.floor(z) & 255

    x -= Math.floor(x)
    y -= Math.floor(y)
    z -= Math.floor(z)

    const u = this.fade(x)
    const v = this.fade(y)
    const w = this.fade(z)

    const A = this.permutation[X] + Y
    const AA = this.permutation[A] + Z
    const AB = this.permutation[A + 1] + Z
    const B = this.permutation[X + 1] + Y
    const BA = this.permutation[B] + Z
    const BB = this.permutation[B + 1] + Z

    return this.lerp(
      w,
      this.lerp(
        v,
        this.lerp(
          u,
          this.grad(this.permutation[AA], x, y, z),
          this.grad(this.permutation[BA], x - 1, y, z)
        ),
        this.lerp(
          u,
          this.grad(this.permutation[AB], x, y - 1, z),
          this.grad(this.permutation[BB], x - 1, y - 1, z)
        )
      ),
      this.lerp(
        v,
        this.lerp(
          u,
          this.grad(this.permutation[AA + 1], x, y, z - 1),
          this.grad(this.permutation[BA + 1], x - 1, y, z - 1)
        ),
        this.lerp(
          u,
          this.grad(this.permutation[AB + 1], x, y - 1, z - 1),
          this.grad(this.permutation[BB + 1], x - 1, y - 1, z - 1)
        )
      )
    )
  }
}

function initWebGL() {
  canvas = document.getElementById("waveCanvas")
  const gl = canvas.getContext("webgl")
  if (!gl) {
    console.error("WebGL not supported")
    return
  }

  // Vertex shader
  const vsSource = `
      attribute vec4 aVertexPosition;
      attribute vec4 aVertexColor;
      uniform mat4 uModelViewMatrix;
      uniform mat4 uProjectionMatrix;
      uniform float uYOffset;
      varying lowp vec4 vColor;
      void main(void) {
        gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
        float fadeAmount = smoothstep(-0.5, 0.5, uYOffset);
        vColor = vec4(mix(vec3(0.0, 0.1, 0.1), aVertexColor.rgb, fadeAmount), aVertexColor.a);
      }
    `

  // Fragment shader
  const fsSource = `
      precision mediump float;
      varying lowp vec4 vColor;
      void main(void) {
        gl_FragColor = vColor;
      }
    `

  // Initialize shaders
  function initShaderProgram(gl, vsSource, fsSource) {
    const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource)
    const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource)

    if (!vertexShader || !fragmentShader) {
      return null
    }

    const shaderProgram = gl.createProgram()
    gl.attachShader(shaderProgram, vertexShader)
    gl.attachShader(shaderProgram, fragmentShader)
    gl.linkProgram(shaderProgram)

    if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
      console.error(
        "Unable to initialize the shader program: " + gl.getProgramInfoLog(shaderProgram)
      )
      return null
    }

    return shaderProgram
  }

  function loadShader(gl, type, source) {
    const shader = gl.createShader(type)
    gl.shaderSource(shader, source)
    gl.compileShader(shader)

    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error("An error occurred compiling the shaders: " + gl.getShaderInfoLog(shader))
      gl.deleteShader(shader)
      return null
    }

    return shader
  }

  const shaderProgram = initShaderProgram(gl, vsSource, fsSource)
  if (!shaderProgram) {
    console.error("Failed to initialize shader program")
    return
  }

  const programInfo = {
    program: shaderProgram,
    attribLocations: {
      vertexPosition: gl.getAttribLocation(shaderProgram, "aVertexPosition"),
      vertexColor: gl.getAttribLocation(shaderProgram, "aVertexColor")
    },
    uniformLocations: {
      projectionMatrix: gl.getUniformLocation(shaderProgram, "uProjectionMatrix"),
      modelViewMatrix: gl.getUniformLocation(shaderProgram, "uModelViewMatrix"),
      yOffset: gl.getUniformLocation(shaderProgram, "uYOffset")
    }
  }

  // Create cube geometry
  function initBuffers(gl) {
    const positions = [
      // Front face
      -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, 0.5,
      // Back face
      -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, -0.5,
      // Top face
      -0.5, 0.5, -0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5,
      // Bottom face
      -0.5, -0.5, -0.5, 0.5, -0.5, -0.5, 0.5, -0.5, 0.5, -0.5, -0.5, 0.5,
      // Right face
      0.5, -0.5, -0.5, 0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5,
      // Left face
      -0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5
    ]

    const positionBuffer = gl.createBuffer()
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer)
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW)

    const CYAN = [0.71, 0.94, 0.99, 1.0]
    const MID_CYAN = [0.51, 0.94, 0.99, 1.0]
    const DARK_CYAN = [0.37, 0.87, 0.99, 1.0]
    const faceColors = [
      CYAN, // Front face
      CYAN, // Back face
      DARK_CYAN, // Top face
      DARK_CYAN, // Bottom face
      MID_CYAN, // Right face
      MID_CYAN // Left face
    ]

    let colors = []

    for (let j = 0; j < faceColors.length; ++j) {
      const c = faceColors[j]
      colors = colors.concat(c, c, c, c)
    }

    const colorBuffer = gl.createBuffer()
    gl.bindBuffer(gl.ARRAY_BUFFER, colorBuffer)
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(colors), gl.STATIC_DRAW)

    const indices = [
      0,
      1,
      2,
      0,
      2,
      3, // front
      4,
      5,
      6,
      4,
      6,
      7, // back
      8,
      9,
      10,
      8,
      10,
      11, // top
      12,
      13,
      14,
      12,
      14,
      15, // bottom
      16,
      17,
      18,
      16,
      18,
      19, // right
      20,
      21,
      22,
      20,
      22,
      23 // left
    ]

    const indexBuffer = gl.createBuffer()
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer)
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW)

    return {
      position: positionBuffer,
      color: colorBuffer,
      indices: indexBuffer
    }
  }

  const buffers = initBuffers(gl)

  const perlin = new PerlinNoise()

  // New function to calculate wave offset
  function calculateWaveOffset(x, z, time) {
    const scale = 0.1
    const speed = 0.5
    const amplitude = 1.0

    const noiseValue = perlin.noise(x * scale, z * scale, time * speed)
    return noiseValue * amplitude
  }

  // Draw scene
  function drawScene(gl, programInfo, buffers, cubePositions, totalTime) {
    gl.viewport(0, 0, displayWidth, displayHeight)
    gl.clearColor(0.0, 0.0, 0.0, 1.0)
    gl.clearDepth(1.0)
    gl.enable(gl.DEPTH_TEST)
    gl.depthFunc(gl.LEQUAL)

    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT)

    const fieldOfView = (50 * Math.PI) / 180
    const aspect = gl.canvas.clientWidth / gl.canvas.clientHeight
    const zNear = 0.1
    const zFar = 100.0
    const projectionMatrix = glMatrix.mat4.create()

    glMatrix.mat4.perspective(projectionMatrix, fieldOfView, aspect, zNear, zFar)

    const modelViewMatrix = glMatrix.mat4.create()

    // mouseX -= 0.05;
    // mouseY -= 0.05;

    // Smooth out mouse movement
    mouseX += (targetMouseX - mouseX) * 0.1
    mouseY += (targetMouseY - mouseY) * 0.1

    // glMatrix.mat4.translate(modelViewMatrix, modelViewMatrix, [-10, 0, 0])

    // glMatrix.mat4.translate(modelViewMatrix, modelViewMatrix, [0, 0, 0]);
    glMatrix.mat4.translate(modelViewMatrix, modelViewMatrix, [0, 0, -16])
    // glMatrix.mat4.rotate(modelViewMatrix, modelViewMatrix, Math.PI / 2, [1, 0, 0]);
    // glMatrix.mat4.rotate(modelViewMatrix, modelViewMatrix, -Math.PI / 4, [0, 1, 0]);
    console.log(totalTime)
    const startRotation = Math.PI / 2 // Starting rotation angle
    const endRotation = Math.PI / 4 // Ending rotation angle
    const duration = 4 // Total duration of the transition in seconds

    // Current time since the animation started (you need to define how you get this)

    // Normalize time to a value between 0 and 1
    const s = Math.max(0, Math.min(1, totalTime / duration))

    // Smoothstep easing function for smooth transition
    function smoothStep(s) {
      return s * s * (3 - 2 * s)
    }

    // Compute the timing factor using the easing function
    const timing = smoothStep(s)

    // Calculate the current rotation value
    const rotation = startRotation + (endRotation - startRotation) * timing

    glMatrix.mat4.rotate(modelViewMatrix, modelViewMatrix, rotation + mouseY * 0.05, [1, 0, 0])
    glMatrix.mat4.rotate(modelViewMatrix, modelViewMatrix, -rotation + mouseX * 0.05, [0, 1, 0])

    // Set up attribute buffers
    {
      const numComponents = 3
      const type = gl.FLOAT
      const normalize = false
      const stride = 0
      const offset = 0
      gl.bindBuffer(gl.ARRAY_BUFFER, buffers.position)
      gl.vertexAttribPointer(
        programInfo.attribLocations.vertexPosition,
        numComponents,
        type,
        normalize,
        stride,
        offset
      )
      gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition)
    }

    {
      const numComponents = 4
      const type = gl.FLOAT
      const normalize = false
      const stride = 0
      const offset = 0
      gl.bindBuffer(gl.ARRAY_BUFFER, buffers.color)
      gl.vertexAttribPointer(
        programInfo.attribLocations.vertexColor,
        numComponents,
        type,
        normalize,
        stride,
        offset
      )
      gl.enableVertexAttribArray(programInfo.attribLocations.vertexColor)
    }

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buffers.indices)

    gl.useProgram(programInfo.program)

    gl.uniformMatrix4fv(programInfo.uniformLocations.projectionMatrix, false, projectionMatrix)

    // Draw cubes
    for (let i = 0; i < cubePositions.length; i++) {
      const cubeMatrix = glMatrix.mat4.create()
      const [x, _, z] = cubePositions[i]
      glMatrix.mat4.translate(cubeMatrix, modelViewMatrix, cubePositions[i])

      // Apply wave motion with new calculation
      const waveOffset = calculateWaveOffset(x, z, totalTime)
      glMatrix.mat4.translate(cubeMatrix, cubeMatrix, [0, waveOffset, 0])

      // Set y-offset uniform for fading
      gl.uniform1f(programInfo.uniformLocations.yOffset, waveOffset)

      // Scale down the cubes
      glMatrix.mat4.scale(cubeMatrix, cubeMatrix, [0.4, 0.4, 0.4])

      gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, cubeMatrix)

      {
        const vertexCount = 36
        const type = gl.UNSIGNED_SHORT
        const offset = 0
        gl.drawElements(gl.TRIANGLES, vertexCount, type, offset)
      }
    }
  }

  function onResize(entries) {
    for (const entry of entries) {
      let width: number
      let height: number
      let dpr = window.devicePixelRatio
      let dprSupport = false
      if (entry.devicePixelContentBoxSize) {
        // NOTE: Only this path gives the correct answer
        // The other paths are an imperfect fallback
        // for browsers that don't provide anyway to do this
        width = entry.devicePixelContentBoxSize[0].inlineSize
        height = entry.devicePixelContentBoxSize[0].blockSize
        dpr = 1 // it's already in width and height
        dprSupport = true
      } else if (entry.contentBoxSize) {
        if (entry.contentBoxSize[0]) {
          width = entry.contentBoxSize[0].inlineSize
          height = entry.contentBoxSize[0].blockSize
        } else {
          // legacy
          width = entry.contentBoxSize.inlineSize
          height = entry.contentBoxSize.blockSize
        }
      } else {
        // legacy
        width = entry.contentRect.width
        height = entry.contentRect.height
      }
      if (!RETINA_ENABLED) {
        dpr = 0.71
      }
      // update global state reflecting ideal canvas size
      displayWidth = Math.round(width * dpr)
      displayHeight = Math.round(height * dpr)
    }
  }

  const resizeObserver = new ResizeObserver(onResize)
  resizeObserver.observe(canvas, { box: "content-box" })

  // Initialize cube positions
  const cubePositions = []
  for (let x = -28; x < 10; x++) {
    for (let z = -28; z < 10; z++) {
      cubePositions.push([x * 1.2 + 0.6, 0, z * 1.2 + 0.6])
    }
  }

  // Animation loop
  let then = 0

  function render(now) {
    now *= 0.001 // convert to seconds
    const deltaTime = now - then
    then = now

    canvas.width = displayWidth
    canvas.height = displayHeight
    // gl.canvas.clientWidth = displayWidth;
    // gl.canvas.clientHeight = displayHeight;

    drawScene(gl, programInfo, buffers, cubePositions, now)

    requestAnimationFrame(render)
  }

  requestAnimationFrame(render)

  // Update mouse position
  function updateMousePosition(event) {
    const rect = canvas.getBoundingClientRect()
    targetMouseX = ((event.clientX - rect.left) / canvas.width) * 4 - 1
    targetMouseY = -(((event.clientY - rect.top) / canvas.height) * 4) + 1
  }

  document.addEventListener("mousemove", updateMousePosition)

  // Cleanup function
  return () => {
    canvas.removeEventListener("mousemove", updateMousePosition)
  }
}

// Initialize WebGL when the component mounts
document.addEventListener("DOMContentLoaded", initWebGL)
