import { defineConfig } from 'vite';
import FullReload from 'vite-plugin-full-reload';
import tailwindcss from '@tailwindcss/vite'

export default defineConfig(({ command }) => ({
  plugins: [
    // Only enable full reload in development
    command === 'serve' && FullReload(['templates/**/*.html.jinja']),
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
        'js/old-style.ts',
      ],
      treeshake: 'smallest'
    },
    outDir: 'static/target'
  },
  base: '/static/target'
}));
