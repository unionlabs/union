import colors from "tailwindcss/colors";
import plugin from "tailwindcss/plugin";
import type { Config } from "tailwindcss";
import tailwindAnimate from "tailwindcss-animate";
import defaultTheme from "tailwindcss/defaultTheme";
import typographyPlugin from "@tailwindcss/typography";
import starlightPlugin from "@astrojs/starlight-tailwind";
import aspectRatioPlugin from "@tailwindcss/aspect-ratio";
import containerQueriesPlugin from "@tailwindcss/container-queries";

const linkHeadingStyles = {
  color: colors.gray[100],
  borderBottomColor: "transparent",
  "&:hover": { color: `${colors.gray[900]}` },
};

const extendTheme = {
  screens: {
    xs: "320px",
  },
  typography: ({
    theme,
  }: {
    theme: (
      path: string
    ) => string | number | Record<string, unknown> | Array<unknown> | undefined;
  }) => ({
    DEFAULT: {
      css: {
        // h1: { fontFamily: theme('fontFamily.obviously') },
        pre: { background: "rgba(205, 200, 255, 0.05)" },
        "h2 a": linkHeadingStyles,
        "h3 a": linkHeadingStyles,
        "h4 a": linkHeadingStyles,
        "h5 a": linkHeadingStyles,
        "h6 a": linkHeadingStyles,
        blockquote: {
          fontSize: "90%",
          color: colors.zinc[500],
          borderLeftColor: colors.zinc[700],
          "p::before": { display: "none" },
          "p::after": { display: "none" },
        },
        a: {
          textDecoration: "none",
          borderBottom: `2px solid ${colors.cyan[800]}`,
          color: colors.cyan[400],
          transition:
            "color 0.2s ease, border-color 0.2s ease, background 0.2s ease",
          "&:hover": {
            color: `${colors.zinc[900]} !important`,
            borderBottomColor: `${colors.cyan[200]} !important`,
            background: colors.cyan[200],
          },
        },
        code: {
          color: "#86e1fc",
          "&::before": { content: `unset !important` },
          "&::after": { content: `unset !important` },
          fontWeight: "normal",
        },
        "[data-rehype-pretty-code-fragment]:nth-of-type(2) pre": {
          "[data-line]::before": {
            content: "counter(line)",
            counterIncrement: "line",
            display: "inline-block",
            width: "1rem",
            marginRight: "1rem",
            textAlign: "right",
            color: colors.slate[600],
          },
          "[data-highlighted-line]::before": { color: colors.slate[400] },
        },
      },
    },
  }),
  colors: {
    background: "#1b1b1d",
    bgDark: "#242526",
    sub: "#252525",
    primary: "#fff",
    accent: {
      "50": "#fdf2f8",
      "100": "#fce7f3",
      "200": "#ffc4e1",
      "300": "#f9a8d4",
      "400": "#FF87C3",
      "500": "#ec4899",
      "600": "#ff63c1",
    },
    gray: colors.zinc,
  },
  fontFamily: {
    mono: ['"IBM Plex Mono"', ...defaultTheme.fontFamily.mono],
    sans: ["Inter"],
    serif: ["Inter"],
    argon: ['"Monospace Argon"'],
  },
};

export default {
  content: ["./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}"],
  darkMode: "class",
  important: true,
  future: { hoverOnlyWhenSupported: true },
  theme: {
    transparent: "transparent",
    current: "currentColor",
    extend: extendTheme,
  },
  plugins: [
    starlightPlugin(),
    tailwindAnimate,
    typographyPlugin,
    aspectRatioPlugin,
    containerQueriesPlugin,
    plugin(({ addVariant, addUtilities, matchUtilities, theme }) => {
      matchUtilities(
        { "animation-delay": (value) => ({ "animation-delay": value }) },
        { values: theme("transitionDelay") }
      );
      addVariant("optional", "&:optional");
      addVariant("hocus", ["&:hover", "&:focus"]);
      addVariant("inverted-colors", "@media (inverted-colors: inverted)");
      addUtilities({
        ".content-auto": { "content-visibility": "auto" },
        ".content-hidden": { "content-visibility": "hidden" },
        ".content-visible": { "content-visibility": "visible" },
      });
    }),
  ],
} satisfies Config;
