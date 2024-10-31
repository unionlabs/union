import * as glMatrix from "gl-matrix"

let currentPlaneRotation = 0
let targetPlaneRotation = 0
let isRotating = false
let rotationStartTime = 0
let queuedRotations = 0
const ROTATION_DURATION = 1200
const EASE_POWER = 4

const colorSets = [
  {
    primary: [0.71, 0.94, 0.99, 1.0], // Cyan
    mid: [0.51, 0.94, 0.99, 1.0],
    dark: [0.37, 0.87, 0.99, 1.0]
  },
  {
    primary: [0.99, 0.71, 0.94, 1.0], // Pink
    mid: [0.99, 0.51, 0.94, 1.0],
    dark: [0.99, 0.37, 0.87, 1.0]
  },
  {
    primary: [0.94, 0.99, 0.71, 1.0], // Yellow
    mid: [0.94, 0.99, 0.51, 1.0],
    dark: [0.87, 0.99, 0.37, 1.0]
  },
  {
    primary: [0.71, 0.99, 0.78, 1.0], // Mint
    mid: [0.51, 0.99, 0.61, 1.0],
    dark: [0.37, 0.99, 0.45, 1.0]
  }
]

let currentColorSet = 0
let targetColorSet = 0
let lastColorIndex = 0
let initialRotation = 0 // Store the starting rotation when animation begins
let colorTransitionProgress = 0
let currentRotationDirection: "left" | "right" = "right"

export function rotateCamera(direction: "left" | "right", colorIndex = -1) {
  const MAX_QUEUED_ROTATIONS = 4
  if (queuedRotations >= MAX_QUEUED_ROTATIONS) return

  queuedRotations++

  // Set or update rotation direction
  if (!isRotating) {
    currentRotationDirection = direction
  } else if (currentRotationDirection !== direction) {
    // If trying to rotate in opposite direction while rotating, ignore the new direction
    direction = currentRotationDirection
  }

  if (colorIndex >= 0) {
    const normalizedIndex = colorIndex % colorSets.length
    targetColorSet = normalizedIndex
    lastColorIndex = normalizedIndex
  } else {
    lastColorIndex =
      direction === "right"
        ? (lastColorIndex + 1) % colorSets.length
        : (lastColorIndex - 1 + colorSets.length) % colorSets.length
    targetColorSet = lastColorIndex
  }

  if (isRotating) {
    const rotationAmount = direction === "right" ? Math.PI / 2 : -Math.PI / 2
    targetPlaneRotation = initialRotation + rotationAmount * queuedRotations
  } else {
    isRotating = true
    rotationStartTime = performance.now()
    initialRotation = currentPlaneRotation
    targetPlaneRotation =
      currentPlaneRotation + (direction === "right" ? Math.PI / 2 : -Math.PI / 2)
    colorTransitionProgress = 0
  }
}

function interpolateColors(colorA, colorB, progress) {
  return colorA.map((c, i) => c + (colorB[i] - c) * progress)
}

export function setInitialColor(colorIndex = 1) {
  const normalizedIndex = colorIndex % colorSets.length
  currentColorSet = normalizedIndex
  targetColorSet = normalizedIndex
  lastColorIndex = normalizedIndex
}

let canvas: HTMLCanvasElement
let mouseX = 0
let mouseY = 0
let targetMouseX = 0
let targetMouseY = 0
let displayWidth = 1000
let displayHeight = 1000
const RETINA_ENABLED = true
const WIDTH = 80 // Must be even
const endRotationY = Math.PI / 4 // Ending rotation angle
const endRotationX = Math.PI / 4 // Ending rotation angle

const W2 = WIDTH / 2

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

function initWebGL(initialColorIndex: number) {
  setInitialColor(initialColorIndex)
  canvas = document.querySelector("#waveCanvas") // Make sure we're using querySelector
  if (!canvas) {
    console.error("Canvas not found")
    return
  }
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
    vColor = vec4(mix(vec3(0.0, 0.05, 0.05), aVertexColor.rgb, fadeAmount), aVertexColor.a);
  }
`

  function updateBufferColors(gl, buffers, colorSet) {
    const faceColors = [
      colorSet.primary,
      colorSet.primary,
      colorSet.dark,
      colorSet.dark,
      colorSet.mid,
      colorSet.mid
    ]

    let colors: Array<number> = []
    for (let j = 0; j < faceColors.length; ++j) {
      const c = faceColors[j]
      colors = colors.concat(c, c, c, c)
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, buffers.color)
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(colors), gl.STATIC_DRAW)
  }

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

    if (!(vertexShader && fragmentShader)) {
      return null
    }

    const shaderProgram = gl.createProgram()
    gl.attachShader(shaderProgram, vertexShader)
    gl.attachShader(shaderProgram, fragmentShader)
    gl.linkProgram(shaderProgram)

    if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
      console.error(
        `Unable to initialize the shader program: ${gl.getProgramInfoLog(shaderProgram)}`
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
      console.error(`An error occurred compiling the shaders: ${gl.getShaderInfoLog(shader)}`)
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
      yOffset: gl.getUniformLocation(shaderProgram, "uYOffset"),
      colorMix: gl.getUniformLocation(shaderProgram, "uColorMix"),
      nextColorPrimary: gl.getUniformLocation(shaderProgram, "uNextColorPrimary"),
      nextColorMid: gl.getUniformLocation(shaderProgram, "uNextColorMid"),
      nextColorDark: gl.getUniformLocation(shaderProgram, "uNextColorDark")
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

    const currentColor = colorSets[currentColorSet]
    const colors = generateColors(currentColor)

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

    const cameraMatrix = glMatrix.mat4.create()
    const modelMatrix = glMatrix.mat4.create()

    if (isRotating) {
      const elapsed = performance.now() - rotationStartTime
      const progress = Math.min(elapsed / ROTATION_DURATION, 1)

      // Smooth easing function
      let easeProgress =
        progress < 0.5 ? (progress * 2) ** EASE_POWER / 2 : 1 - (2 - progress * 2) ** EASE_POWER / 2

      // Smoothly update rotation
      const rotationDelta = targetPlaneRotation - initialRotation
      currentPlaneRotation = initialRotation + rotationDelta * easeProgress

      // Update color transition with a slightly different timing
      colorTransitionProgress = Math.min(progress * 1.2, 1) // Slightly faster color transition

      // Calculate color interpolation
      const currentColors = colorSets[currentColorSet]
      const nextColors = colorSets[targetColorSet]

      const interpolatedColors = {
        primary: interpolateColors(
          currentColors.primary,
          nextColors.primary,
          colorTransitionProgress
        ),
        mid: interpolateColors(currentColors.mid, nextColors.mid, colorTransitionProgress),
        dark: interpolateColors(currentColors.dark, nextColors.dark, colorTransitionProgress)
      }

      // Update color buffer with interpolated colors
      const faceColors = [
        interpolatedColors.primary,
        interpolatedColors.primary,
        interpolatedColors.dark,
        interpolatedColors.dark,
        interpolatedColors.mid,
        interpolatedColors.mid
      ]

      let colors: Array<number> = []
      for (const c of faceColors) {
        colors = colors.concat(c, c, c, c)
      }

      gl.bindBuffer(gl.ARRAY_BUFFER, buffers.color)
      gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(colors), gl.STATIC_DRAW)

      if (progress >= 1) {
        currentPlaneRotation = targetPlaneRotation
        currentColorSet = targetColorSet
        queuedRotations = Math.max(0, queuedRotations - 1)

        if (queuedRotations > 0) {
          rotationStartTime = performance.now()
          initialRotation = currentPlaneRotation
          targetPlaneRotation = currentPlaneRotation + Math.PI / 2
          colorTransitionProgress = 0
        } else {
          isRotating = false
        }
      }

      if (progress >= 1) {
        currentPlaneRotation = targetPlaneRotation
        currentColorSet = targetColorSet
        queuedRotations = Math.max(0, queuedRotations - 1)

        if (queuedRotations > 0) {
          rotationStartTime = performance.now()
          initialRotation = currentPlaneRotation
          targetPlaneRotation =
            currentPlaneRotation +
            (currentRotationDirection === "right" ? Math.PI / 2 : -Math.PI / 2)
          colorTransitionProgress = 0
        } else {
          isRotating = false
          currentRotationDirection = "right"
        }
      }
    }

    mouseX += (targetMouseX - mouseX) * 0.1
    mouseY += (targetMouseY - mouseY) * 0.1

    glMatrix.mat4.translate(cameraMatrix, cameraMatrix, [0, 0, -16])
    glMatrix.mat4.rotate(cameraMatrix, cameraMatrix, endRotationY + mouseY * 0.1, [1, 0, 0])
    glMatrix.mat4.rotate(cameraMatrix, cameraMatrix, -endRotationX + mouseX * 0.1, [0, 1, 0])
    glMatrix.mat4.rotate(modelMatrix, modelMatrix, currentPlaneRotation, [0, 1, 0])

    const modelViewMatrix = glMatrix.mat4.create()
    glMatrix.mat4.multiply(modelViewMatrix, cameraMatrix, modelMatrix)

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

    for (let i = 0; i < cubePositions.length; i++) {
      const cubeMatrix = glMatrix.mat4.create()
      const [x, _, z] = cubePositions[i]
      glMatrix.mat4.translate(cubeMatrix, modelViewMatrix, cubePositions[i])

      const waveOffset = calculateWaveOffset(x, z, totalTime)
      glMatrix.mat4.translate(cubeMatrix, cubeMatrix, [0, waveOffset * 1.2, 0])

      gl.uniform1f(programInfo.uniformLocations.yOffset, waveOffset)

      glMatrix.mat4.scale(cubeMatrix, cubeMatrix, [0.4, 0.4, 0.4])

      gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, cubeMatrix)

      const vertexCount = 36
      const type = gl.UNSIGNED_SHORT
      const offset = 0
      gl.drawElements(gl.TRIANGLES, vertexCount, type, offset)
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
  const cubePositions: Array<glMatrix.vec3> = []

  // Manhattan distance based rotation from square to diamond
  for (let x = -W2; x <= W2; x++) {
    for (let z = -W2; z <= W2; z++) {
      if (Math.abs(x) + Math.abs(z) <= W2) {
        cubePositions.push([x * 1.2 + 0.6, 0, z * 1.2 + 0.6])
      }
    }
  }
  // const cubePositions = []
  // for (let x = -28; x < 10; x++) {
  //   for (let z = -28; z < 10; z++) {
  //     cubePositions.push([x * 1.2 + 0.6, 0, z * 1.2 + 0.6])
  //   }
  // }

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

    const x = event.clientX - rect.left
    const y = event.clientY - rect.top

    // Reduced multiplier from 2 to 1 for less movement
    targetMouseX = (x / rect.width) * 2 - 1
    targetMouseY = -(y / rect.height) * 2 + 1
  }

  function handleTouch(event) {
    event.preventDefault()
    const touch = event.touches[0]
    if (touch) {
      const mouseEvent = new MouseEvent("mousemove", {
        clientX: touch.clientX,
        clientY: touch.clientY
      })
      updateMousePosition(mouseEvent)
    }
  }

  function handleMouseLeave(event) {
    const rect = canvas.getBoundingClientRect()
    if (event.clientY > rect.bottom) {
      targetMouseX = 0
      targetMouseY = 0
    }
  }

  document.addEventListener("mousemove", updateMousePosition)
  canvas.addEventListener("mouseleave", handleMouseLeave)
  canvas.addEventListener("touchstart", handleTouch)
  canvas.addEventListener("touchmove", handleTouch)
  canvas.addEventListener("touchend", handleMouseLeave)

  return () => {
    document.removeEventListener("mousemove", updateMousePosition)
    canvas.removeEventListener("mouseleave", handleMouseLeave)
  }
}

function generateColors(colorSet) {
  const faceColors = [
    colorSet.primary, // Front face
    colorSet.primary, // Back face
    colorSet.dark, // Top face
    colorSet.dark, // Bottom face
    colorSet.mid, // Right face
    colorSet.mid // Left face
  ]

  let colors: Array<number> = []
  for (const c of faceColors) {
    colors = colors.concat(c, c, c, c)
  }
  return colors
}

// Initialize WebGL when the component mounts
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => initWebGL(1))
} else {
  initWebGL(1)
}
