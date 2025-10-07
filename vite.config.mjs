import { defineConfig } from 'vite';
import FullReload from 'vite-plugin-full-reload';
import tailwindcss from '@tailwindcss/vite'

export default defineConfig(({ command }) => ({
  plugins: [
    // Only enable full reload in development
    command === 'serve' && FullReload(['templates/**/*.html.jinja'], {delay: 1000, log: true}),
    tailwindcss(),
  ].filter(Boolean),
  build: {
    // generate .vite/manifest.json in outDir
    manifest: true,
    rollupOptions: {
      // overwrite default .html entry
      input: [
        'js/index.ts',
        'js/comments.ts',
        'js/solve-page.ts',
        'js/create-challenge-page.ts'
      ],
      treeshake: 'smallest'
    },
    outDir: 'static/target'
  },
  base: '/static/target'
}));
