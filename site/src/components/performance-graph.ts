import { roundNumber } from "#/lib/utilities.ts";
/**
 * TODO:
 * red line: memory
 * - x-axis numbers of validators: 0, 8, 16, 24, 32, 40, 48, 56, 64
 * - y-axis seconds it takes to prove: 0, ..., 25s (0, 5, 10, 15, 20)
 */
import * as Plot from "@observablehq/plot";

export const [zkFastLineColor, zkSlowLineColor] = ["#1ED2FA", "#9DA3AE"];

const generateRandomNumber = (min: number, max: number) =>
  Math.random() * (max - min) + min;

const pauseAnimation = (element: SVGPathElement) =>
  (element.style.animationPlayState = "paused");
const resumeAnimation = (element: SVGPathElement) =>
  (element.style.animationPlayState = "running");

function getRelevantPathElements({ selector }: { selector: string }) {
  const gElements = document.querySelectorAll(selector);
  const pathElements = Array.from(gElements).map(
    (gElement) => gElement.querySelector("path") as SVGPathElement
  );
  return pathElements;
}

/**
 * Intersection Observer
 */
const chartElement = document.querySelector(
  'article[data-graph="performance"]'
) as HTMLElement;
const observer = new IntersectionObserver(
  (entries) => {
    const pathElements = getRelevantPathElements({
      selector: `g[stroke="${zkSlowLineColor}"], g[stroke="${zkFastLineColor}"]`,
    });
    entries.forEach((entry) => {
      console.log(
        "intersection observer",
        entry.isIntersecting,
        entry.intersectionRatio
      );
      if (entry.isIntersecting) pathElements.forEach(resumeAnimation);
      // else pathElements.forEach(pauseAnimation)
    });
  },
  { threshold: 0.5 }
);
observer.observe(chartElement);

/**
 * Plot data
 * TODO: replace with real data
 */
const totalLength = 50;
let zkSlowPlotLine = Array.from({ length: totalLength }, (_, index) => ({
  x: index,
  y: 1 + index * 0.1,
}));
let zkFastPlotLine = Array.from({ length: totalLength }, (_, index) => ({
  x: index,
  y: index + generateRandomNumber(-2, 2),
}));

chartElement?.append(
  // @ts-expect-error
  Plot.plot({
    width: 800,
    height: 400,

    x: {
      tickSize: 0,
      axis: "bottom",

      label: "# of validators",
      ariaLabel: "#-validators",
      tickFormat: (d: number) => d,
    },
    y: {
      tickSize: 0,
      axis: "left",
      labelAnchor: "top",
      label: "Seconds to prove",
      scheme: "Viridis",
      fontVariant: "tabular-nums",
      tickFormat: (d: number) => d,
      ariaDescription: "seconds-to-prove",
    },
    figure: true,

    marks: [
      Plot.gridY({
        stroke: "#ffffff",
        strokeWidth: 1,
        strokeOpacity: 0.3,
      }),
      Plot.gridY([0], {
        x: (y, _index) => y,
        color: "#ffffff",
        strokeWidth: 0.5,
        strokeOpacity: 0.5,
      }),
      Plot.line([{ x: 0, y: 2 }].concat(zkFastPlotLine), {
        x: "x",
        y: "y",
        // marginTop: 30,
        curve: "catmull-rom",
        stroke: zkFastLineColor,
      }),
      Plot.line([{ x: 0, y: 1 }].concat(zkSlowPlotLine), {
        x: "x",
        y: "y",
        // marginTop: 30,
        stroke: zkSlowLineColor,
        curve: "bump-y",
      }),
      Plot.tip(
        zkFastPlotLine,
        Plot.pointerX({
          x: "x",
          y: "y",
          fontSize: 12,
          fill: "#181A21",
          fillOpacity: 1,
          strokeWidth: 0,
          textAnchor: "start",
          fontWeight: "bolder",
          frameAnchor: "middle",
          pointerEvents: "none",
          fontVariant: "tabular-nums",
          title: ({ x, y }) => `↑ ${x}\n\n→ ${roundNumber(y, 2)}s`,
        })
      ),
      Plot.dot(
        zkFastPlotLine,
        Plot.pointerX({ x: "x", y: "y", stroke: "red", fill: "red", r: 3 })
      ),
    ],
  })
);
