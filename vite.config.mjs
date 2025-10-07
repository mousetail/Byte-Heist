import { defineConfig } from 'vite';
import FullReload from 'vite-plugin-full-reload';
import tailwindcss from '@tailwindcss/vite';

const disableHotReloadForTemplateAndRustFilesPlugin = {
  handleHotUpdate({modules}) {
    return modules.filter(
      i=>!i.url.endsWith('.html.jinja') || i.url.endsWith('.rs')
    )
  }
}

export default defineConfig(({ command }) => ({
  plugins: [
    // Only enable full reload in development
    command === 'serve' && FullReload(['templates/**/*.html.jinja'], {delay: 500, log: true}),
    tailwindcss(),
    disableHotReloadForTemplateAndRustFilesPlugin
  ].filter(Boolean),
  root: 'js',
  build: {
    // generate .vite/manifest.json in outDir
    manifest: true,
    rollupOptions: {
      // overwrite default .html entry
      input: [
        'index.ts',
        'comments.ts',
        'solve-page.ts',
        'create-challenge-page.ts'
      ],
      treeshake: 'smallest'
    },
    outDir: 'static/target'
  },
  base: '/static/target'
}));
