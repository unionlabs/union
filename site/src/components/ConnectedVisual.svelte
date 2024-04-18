<script lang="ts">
  import { onMount } from "svelte";
  import tw from "tailwind.config";
  import union from "#/assets/union-logo/union-logo-transparent.svg?raw";
  import skip from "#/assets/partners/skip.svg?raw";
  import noble from "#/assets/partners/noble.svg?raw";
  import quasar from "#/assets/partners/quasar.svg?raw";
  import celestia from "#/assets/partners/celestia.svg?raw";
  import ethereum from "#/assets/partners/ethereum.svg?raw";
  import movement from "#/assets/partners/movement.svg?raw";
  import polygon from "#/assets/partners/polygon.svg?raw";
  import scroll from "#/assets/partners/scroll.svg?raw";
  import secret from "#/assets/partners/secret.svg?raw";
  import canto from "#/assets/partners/canto.svg?raw";

  /* Define our media query and media query object */
  let mq: MediaQueryList;
  let svg: Element | null;

  onMount(() => {
    mq = matchMedia("only screen and (max-width: 640px)");
    svg = document.querySelector("#connected-visual");

    /* Store the original value in a variable */
    const originalViewBox = svg?.getAttribute("viewBox");

    /* Define the handler */
    const updateViewBox = () => {
      if (mq.matches) {
        /* Change the viewBox dimensions to show the hexagon */
        svg?.setAttribute("viewBox", `96 64 ${12 * 32 - 1} ${8 * 32 - 1}`);
      } else {
        svg?.setAttribute("viewBox", `${originalViewBox}`);
      }
    };

    updateViewBox();

    svg?.addEventListener("load", () => {
      updateViewBox();
    });

    /* Fire if the media condition changes */
    mq.addEventListener("change", updateViewBox);
  });

  const pos = (p: number) => p * 32;

  type Node = {
    x: number;
    y: number;
    logo: string;
    url?: string;
    scale?: number;
  };

  const nodes: Record<string, Node> = {
    union: { x: 9, y: 6, logo: union, scale: 2.0, url: "https://union.build" },
    celestia: { x: 5, y: 4, logo: celestia, url: "https://celestia.org/" },
    ethereum: {
      x: 7,
      y: 3,
      logo: ethereum,
      url: "https://ethereum.org/",
      scale: 1.2,
    },
    movement: { x: 4, y: 6, logo: movement, url: "https://movementlabs.xyz/" },
    noble: { x: 13, y: 8, logo: noble, url: "https://nobleassets.xyz/" },
    secret: { x: 5, y: 8, logo: secret, url: "https://scrt.network/" },
    quasar: { x: 11, y: 9, logo: quasar, url: "https://quasar.fi/" },
    canto: { x: 8, y: 9, logo: canto, url: "https://canto.io/" },
    polygon: { x: 10, y: 3, logo: polygon, url: "https://polygon.technology/" },
    scroll: { x: 13, y: 4, logo: scroll, url: "https://scroll.io/" },
    skip: { x: 14, y: 6, logo: skip, url: "https://skip.money/", scale: 1.2 },
  };

  const conns: Array<{ from: string; to: string; delay: number }> = [
    { to: "celestia", from: "union", delay: 2 },
    { to: "ethereum", from: "union", delay: 3 },
    { to: "polygon", from: "union", delay: 4 },
    { to: "scroll", from: "union", delay: 3 },
    { to: "secret", from: "union", delay: 2 },
    { to: "canto", from: "union", delay: 4 },
    { to: "movement", from: "union", delay: 4 },
    { to: "noble", from: "union", delay: 5 },
    { to: "quasar", from: "union", delay: 6 },
    { to: "skip", from: "union", delay: 7 },
    { to: "union", from: "celestia", delay: 7 },
    { to: "union", from: "ethereum", delay: 6 },
    { to: "union", from: "movement", delay: 5 },
    { to: "union", from: "polygon", delay: 6 },
    { to: "union", from: "scroll", delay: 5 },
    { to: "union", from: "secret", delay: 5 },
    { to: "union", from: "canto", delay: 3 },
    { to: "union", from: "noble", delay: 4 },
    { to: "union", from: "quasar", delay: 3 },
    { to: "union", from: "skip", delay: 2 },
  ];

  const scale = (
    input: number,
    srcRange: [number, number],
    dstRange: [number, number],
  ) => {
    const [dstMin, dstMax] = dstRange;
    const [srcMin, srcMax] = srcRange;

    const percent = (input - srcMin) / (srcMax - srcMin);
    const dstOutput = percent * (dstMax - dstMin) + dstMin;

    return dstOutput;
  };

  const SECONDS_PER_CELL = 0.7;

  const LOGO_SIZE = 28;

  const connectionData = conns.map(({ from, to, delay }) => {
    const { x: fromX, y: fromY } = nodes[from];
    const { x: toX, y: toY } = nodes[to];

    const dx = toX - fromX;
    const dy = toY - fromY;

    const totalDistance = Math.abs(dx) + Math.abs(dy);

    // console.log(dx, dy, totalDistance)

    const duration = totalDistance * SECONDS_PER_CELL;

    const totalTime = duration + delay;

    // console.log('dx', dx, 'dy', dy, 'duration', duration, 'totalTime', totalTime)

    // time (clamped between [0, 1]) for the h and v components
    const hTime = (Math.abs(dx) * SECONDS_PER_CELL) / totalTime;
    const vTime = (Math.abs(dy) * SECONDS_PER_CELL) / totalTime;

    // console.log('hTime', hTime, 'vTime', vTime)

    // clamp between [0, (duration / totalTime)]
    const radiusKeyTimes = [0, 0.1, 0.4, 0.5, 1]
      .map((x) => scale(x, [0, 1], [0, duration / totalTime]))
      .join(";");

    const cxKeyTimes = [0, 0.5, 1]
      .map((x) => scale(x, [0, 1], [0, hTime]))
      .join(";");

    const cyKeyTimes = [0, 0.5, 1]
      .map((x) => scale(x, [0, 1], [hTime, hTime + vTime]) - hTime / 2)
      .join(";");

    return {
      from,
      to,
      dx,
      dy,
      fromX,
      fromY,
      toX,
      toY,
      totalTime,
      cxKeyTimes,
      cyKeyTimes,
      radiusKeyTimes,
      delay,
    };
  });

  const smallViewBox = `0 0 ${18 * 32 - 1} ${11 * 32 - 1}`;
</script>

<svg
  id="connected-visual"
  class="size-full"
  viewBox="32 32 {16 * 32 - 1} {10 * 32 - 1}"
  version="1.1"
  xmlns="http://www.w3.org/2000/svg"
>
  <defs>
    <linearGradient id="edge-gradient-ns" x1="0" x2="0" y1="0" y2="1">
      <stop offset="0%" stop-color="#000" />
      <stop offset="50%" stop-color="#000" stop-opacity="0" />
      <stop offset="100%" stop-color="#000" />
    </linearGradient>
    <linearGradient id="edge-gradient-ew" x1="0" x2="1" y1="0" y2="0">
      <stop offset="0%" stop-color="#000" />
      <stop offset="20%" stop-color="#000" stop-opacity="0" />
      <stop offset="80%" stop-color="#000" stop-opacity="0" />
      <stop offset="100%" stop-color="#000" />
    </linearGradient>

    <pattern id="grid" width="32" height="32" patternUnits="userSpaceOnUse">
      <path
        d="M 32 0 L 32 32 0 32"
        fill="none"
        stroke="#0f1013"
        vector-effect="non-scaling-stroke"
        stroke-width="1px"
      />
    </pattern>
  </defs>

  <!--
  Uncomment for black background color
  <rect width="100%" height="100%" fill="#000" />
  !-->

  <rect width="150%" height="150%" fill="url(#grid)" />

  {#each connectionData as { from, to, dx, dy, fromX, fromY, toX, toY, totalTime, cxKeyTimes, cyKeyTimes, radiusKeyTimes, delay }}
    <g>
      <path
        d={`M ${pos(fromX)} ${pos(fromY)} h ${pos(dx)} v ${pos(dy)}`}
        fill="none"
        stroke="url(#gradient-{from}-{to})"
        stroke-linecap="round"
        stroke-width="4"
        vector-effect="non-scaling-stroke"
        id="path-{from}-{to}"
      >
      </path>
      <defs>
        <radialGradient
          cx={pos(fromX)}
          cy={pos(fromY)}
          r="12"
          gradientUnits="userSpaceOnUse"
          id="gradient-{from}-{to}"
        >
          <stop offset="0" stop-color={tw.theme.extend.colors.accent[800]}
          ></stop>
          <stop offset="0.4" stop-color={tw.theme.extend.colors.accent[800]}
          ></stop>
          <stop
            offset="1"
            stop-color={tw.theme.extend.colors.accent[800]}
            stop-opacity="0"
          ></stop>
          <animate
            attributeName="cx"
            dur="{totalTime}s"
            begin="{delay}s"
            keyTimes="{cxKeyTimes};1"
            repeatCount="indefinite"
            values="{pos(fromX)};{pos(toX)};{pos(toX)};{pos(toX)}"
            id="cx-{from}-{to}"
          ></animate>
          <animate
            attributeName="cy"
            dur="{totalTime}s"
            begin="{delay}s"
            keyTimes="0;{cyKeyTimes};1"
            repeatCount="indefinite"
            values="{pos(fromY)};{pos(fromY)};{pos(toY)};{pos(toY)};{pos(toY)}"
            id="cy-{from}-{to}"
          ></animate>
          <animate
            attributeName="r"
            dur="{totalTime}s"
            begin="{delay}s"
            keyTimes="{radiusKeyTimes};1"
            values="12;16;16;12;0;0"
            repeatCount="indefinite"
            id="radius-{from}-{to}"
          ></animate>
        </radialGradient>
      </defs>
    </g>
  {/each}
  {#each Object.entries(nodes) as [id, { x, y, logo, url, scale }]}
    <g>
      <defs>
        <radialGradient
          cx={pos(x)}
          cy={pos(y)}
          r="32"
          gradientUnits="userSpaceOnUse"
          id="gradient-{id}"
        >
          <stop offset="0" stop-color="black" />
          <stop offset="0.1" stop-color="black" />
          <stop offset="1" stop-color="black" stop-opacity="0" />
        </radialGradient>
      </defs>
      <a
        href={url}
        aria-label={id}
        target="_blank"
        class={id === "union" ? "pointer-events-none" : ""}
      >
        <circle
          id="circle-{id}"
          cx={pos(x)}
          cy={pos(y)}
          r="32"
          fill="url(#gradient-{id})"
        >
        </circle>
        <svg
          width={(scale || 1) * LOGO_SIZE}
          height={(scale || 1) * LOGO_SIZE}
          x={pos(x) - (scale || 1) * (LOGO_SIZE / 2)}
          y={pos(y) - (scale || 1) * (LOGO_SIZE / 2)}
        >
          {@html logo}
        </svg>
      </a>
    </g>
  {/each}
</svg>
