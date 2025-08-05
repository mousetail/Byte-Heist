/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "../templates/**/*.html.jinja",
    "./**/*.js",
  ],
  plugins: [],
  theme: {
    extend: {
      fontFamily: {
          'kode': ['Kode Mono', 'monospace'],
          'inter': ['Inter', 'sans-serif'],
      },
      colors: {
        'byte-green-950': '#042F15', // very dark green
        'byte-green-900': '#063C1B', // dark green
        'byte-green-700': '#27B762', // emerald/primary
        'byte-green-100': '#BDFFD8', // mint/light
        'byte-green-50': '#E2FFEE', // honeydew/very light
        'byte-brown-950': '#130F0C', // very dark brown
        'byte-brown-700': '#2C2824', // dark brown
        'byte-brown-500': '#654E3D', // brown
        'byte-brown-200': '#967B60', // light brown
      }
    }
  },
} 