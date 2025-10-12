import { defineConfig } from 'vite';
import FullReload from 'vite-plugin-full-reload';
import tailwindcss from '@tailwindcss/vite';

const disableHotReloadForTemplateAndRustFilesPlugin = {
  handleHotUpdate({modules}) {
    return modules.filter(
      i=>!i.url.endsWith('.html.jinja') && !i.url.endsWith('.rs')
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
  server: {
    watch: {
      ignored: ['**/target/debug/**/*'],
    }
  },
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
      treeshake: {
        preset: 'smallest',
        moduleSideEffects: (id, external) => {
          return !!id.match(/node_modules\/basecoat-css/)
        }
      }
    },
    outDir: 'static/target'
  },
  base: '/static/target'
}));
