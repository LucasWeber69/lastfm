/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        background: '#0a0a0a',
        surface: '#141414',
        'surface-hover': '#1f1f1f',
        primary: '#a855f7',
        'primary-hover': '#9333ea',
        secondary: '#ec4899',
        'text-primary': '#ffffff',
        'text-secondary': '#a1a1aa',
        border: '#27272a',
      },
      backgroundImage: {
        'accent-gradient': 'linear-gradient(135deg, #a855f7 0%, #ec4899 100%)',
      },
      boxShadow: {
        'glow-purple': '0 0 20px rgba(168, 85, 247, 0.3)',
        'glow-pink': '0 0 20px rgba(236, 72, 153, 0.3)',
        'glow-gradient': '0 0 30px rgba(168, 85, 247, 0.4), 0 0 60px rgba(236, 72, 153, 0.2)',
      },
      backdropBlur: {
        xs: '2px',
      },
    },
  },
  plugins: [],
}
