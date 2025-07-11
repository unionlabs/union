/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        union: {
          DEFAULT: '#5865F2',
          800: '#1e293b',
          900: '#0f172a',
        }
      }
    },
  },
  plugins: [],
}
