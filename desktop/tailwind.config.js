/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        fluent: {
          bg: '#0c0e12',
          sidebar: 'rgba(18, 22, 28, 0.75)',
          card: 'rgba(26, 32, 42, 0.65)',
          cardHover: 'rgba(36, 44, 58, 0.8)',
          border: 'rgba(255, 255, 255, 0.08)',
          accent: '#0078d4',
          accentHover: '#1084d8',
          textMuted: '#94a3b8',
        }
      },
      fontFamily: {
        sans: ['"Segoe UI Variable"', '"Segoe UI"', 'system-ui', 'sans-serif'],
        mono: ['"Cascadia Code"', 'Consolas', 'monospace'],
      }
    },
  },
  plugins: [],
}
