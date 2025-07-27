import { defineConfig } from 'vite';
import FullReload from 'vite-plugin-full-reload';

export default defineConfig(({ command }) => ({
  plugins: [
    // Only enable full reload in development
    command === 'serve' && FullReload(['templates/**/*.html.jinja'])
  ].filter(Boolean),
  build: {
    // generate .vite/manifest.json in outDir
    manifest: true,
    rollupOptions: {
      // overwrite default .html entry
      input: [
        'js/index.ts',
        'js/comments.ts',
        'js/old-style.css',
      ],
      treeshake: 'smallest'
    },
    outDir: 'static/target'
  },
  base: '/static/target'
}));
