import fluidPlugin, {
  extract as fluidExtract,
  screens as fluidScreens,
  fontSize as fluidFontSize
} from "fluid-tailwind"
import plugin from "tailwindcss/plugin"
import type { Config } from "tailwindcss"
import formsPlugin from "@tailwindcss/forms"
import tailwindAnimate from "tailwindcss-animate"
import defaultTheme from "tailwindcss/defaultTheme"
import typographyPlugin from "@tailwindcss/typography"
import tailwindScrollbarPlugin from "tailwind-scrollbar"
import aspectRatioPlugin from "@tailwindcss/aspect-ratio"
import containerQueriesPlugin from "@tailwindcss/container-queries"

export default (<Config>{
  content: {
    extract: fluidExtract,
    files: ["./src/**/*.{html,js,svelte,ts}"]
  },
  experimental: {
    matchVariant: true
  },
  safelist: ["dark", "light"],
  darkMode: "class",
  theme: {
    screens: _ => fluidScreens,
    fontSize: _ => fluidFontSize,
    container: _ => ({
      center: true,
      padding: "2rem",
      screens: { "2xl": "1400px" }
    }),
    extend: {
      brightness: _ => ({
        5: ".05",
        10: ".1",
        40: ".4",
        60: ".6",
        65: ".65",
        70: ".7",
        80: ".8",
        90: ".9",
        175: "1.75"
      }),
      fontSize: _ => ({
        "10xl": ["10rem", { lineHeight: "10rem" }],
        "11xl": ["11rem", { lineHeight: "11rem" }],
        "12xl": ["12rem", { lineHeight: "12rem" }],
        "13xl": ["13rem", { lineHeight: "13rem" }],
        "14xl": ["14rem", { lineHeight: "14rem" }],
        "15xl": ["15rem", { lineHeight: "15rem" }]
      }),
      cursor: _ => ({
        fancy: "url(hand.cur), pointer"
      }),
      colors: _ => ({
        "union-accent": {
          DEFAULT: "#A0ECFD",
          50: "#FAFEFF",
          100: "#F0FCFF",
          200: "#DCF8FE",
          300: "#C8F4FE",
          400: "#B4F0FD",
          500: "#A0ECFD",
          600: "#5FDFFC",
          700: "#1ED2FA",
          800: "#04ACD2",
          900: "#037791",
          950: "#025C70",
          foreground: "hsl(var(--accent-foreground) / <alpha-value>)"
        },
        border: "hsl(var(--border) / <alpha-value>)",
        input: "hsl(var(--input) / <alpha-value>)",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background) / <alpha-value>)",
        foreground: "hsl(var(--foreground) / <alpha-value>)",
        primary: {
          DEFAULT: "hsl(var(--primary) / <alpha-value>)",
          foreground: "hsl(var(--primary-foreground) / <alpha-value>)"
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary) / <alpha-value>)",
          foreground: "hsl(var(--secondary-foreground) / <alpha-value>)"
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive) / <alpha-value>)",
          foreground: "hsl(var(--destructive-foreground) / <alpha-value>)"
        },
        muted: {
          DEFAULT: "hsl(var(--muted) / <alpha-value>)",
          foreground: "hsl(var(--muted-foreground) / <alpha-value>)"
        },
        accent: {
          DEFAULT: "hsl(var(--accent) / <alpha-value>)",
          foreground: "hsl(var(--accent-foreground) / <alpha-value>)"
        },
        popover: {
          DEFAULT: "hsl(var(--popover) / <alpha-value>)",
          foreground: "hsl(var(--popover-foreground) / <alpha-value>)"
        },
        card: {
          DEFAULT: "hsl(var(--card) / <alpha-value>)",
          foreground: "hsl(var(--card-foreground) / <alpha-value>)"
        }
      }),
      borderRadius: _ => ({
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)"
      }),
      screens: _ => ({
        xs: "480px",
        tiny: "380px"
      }),
      margin: _ => ({
        "0.25": "0.0625rem",
        "0.75": "0.1875rem",
        18: "4.5rem",
        22: "5.5rem",
        28: "7rem"
      }),
      spacing: _ => ({
        2.75: "0.6875rem",
        4.5: "1.125rem",
        5.5: "1.375rem",
        13: "3.25rem",
        18: "4.5rem",
        22: "5.5rem",
        26: "6.5rem",
        28: "7rem"
      }),
      width: _ => ({
        "7xl": "80rem",
        26: "6.5rem"
      }),
      maxWidth: _ => ({
        26: "6.5rem",
        "8xl": "88rem",
        "9xl": "96rem"
      }),
      fontFamily: _ => ({
        gunship: ["gunship", ...defaultTheme.fontFamily.sans],
        supermolot: ["tt-supermolot-neue", ...defaultTheme.fontFamily.sans],
        sans: ["jetbrains-mono", ...defaultTheme.fontFamily.sans],
        mono: ["jetbrains-mono", ...defaultTheme.fontFamily.mono]
      }),
      animation: _ => ({
        wiggle: "wiggle 1s ease-in-out infinite",
        "text-gradient": "text-gradient 1.5s linear infinite",
        "background-shine": "background-shine 2s linear infinite",
        "pulse-slow": "pulse 6s infinite cubic-bezier(0.4, 0, 0.6, 1)",
        "border-width": "border-width 3s infinite alternate"
      }),
      animationDuration: { "2s": "2s" },
      keyframes: _ => ({
        "text-gradient": {
          to: { backgroundPosition: "200% center" }
        },
        "background-shine": {
          from: { backgroundPosition: "0 0" },
          to: { backgroundPosition: "-200% 0" }
        },
        "border-width": {
          from: { width: "10px", opacity: "0" },
          to: { width: "100px", opacity: "1" }
        },
        wiggle: {
          "0%, 100%": { transform: "rotate(-3deg)" },
          "50%": { transform: "rotate(3deg)" }
        }
      }),
      transitionDelay: _ => ({ "2000": "2000ms" }),
      transitionProperty: _ => ({ height: "height", spacing: "margin, padding" }),
      transitionTimingFunction: _ => ({
        "in-expo": "cubic-bezier(0.95, 0.05, 0.795, 0.035)",
        "out-expo": "cubic-bezier(0.19, 1, 0.22, 1)"
      })
    }
  },
  plugins: [
    tailwindAnimate,
    aspectRatioPlugin,
    typographyPlugin(),
    containerQueriesPlugin,
    customVariantsPlugin(),
    tailwindScrollbarPlugin,
    fluidPlugin({ checkSC144: false }),
    formsPlugin({ strategy: "class" })
  ]
})

function customVariantsPlugin() {
  return plugin(({ addVariant, addUtilities, matchUtilities, theme }) => {
    matchUtilities(
      { "animation-delay": value => ({ "animation-delay": value }) },
      { values: theme("transitionDelay") }
    )
    addVariant("optional", "&:optional")
    addVariant("hocus", ["&:hover", "&:focus"])
    addUtilities({
      ".content-auto": { "content-visibility": "auto" },
      ".content-hidden": { "content-visibility": "hidden" },
      ".content-visible": { "content-visibility": "visible" }
    })
    addVariant("inverted-colors", "@media (inverted-colors: inverted)")
  })
}
