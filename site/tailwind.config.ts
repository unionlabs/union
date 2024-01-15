import colors from "tailwindcss/colors";
import plugin from "tailwindcss/plugin";
import type { Config } from "tailwindcss";
import tailwindAnimate from "tailwindcss-animate";
import defaultTheme from "tailwindcss/defaultTheme";
import typographyPlugin from "@tailwindcss/typography";
import starlightPlugin from "@astrojs/starlight-tailwind";
import aspectRatioPlugin from "@tailwindcss/aspect-ratio";
import containerQueriesPlugin from "@tailwindcss/container-queries";

export default {
  content: ["./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}"],
  darkMode: "class",
  important: true,
  future: { hoverOnlyWhenSupported: true },
  theme: {
    transparent: "transparent",
    current: "currentColor",
    extend: {
      screens: {
        xs: "320px",
      },
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
        sans: [
          '"Inter var", sans-serif',
          {
            fontFeatureSettings: '"cv11", "ss01"',
            fontVariationSettings: '"opsz" 32',
          },
        ],
        serif: ['"Inter var"'],
        argon: ['"Monospace Argon"'],
      },
    },
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
