import path from 'path'

// bundle.js is built from lib/**/*.ts, so every custom element under
// lib/web-components/ is already registered when a page bundle runs. Importing
// one again throws NotSupportedError.
//
// Page-safe support code lives in lib/web-components/base/. Everything else
// under lib/web-components/ is a component, so a page build rejects it.
// onResolve also sees nested imports, which covers re-exports.
const webRoot = path.resolve(import.meta.dir, '..')
const componentsDir = path.join(webRoot, 'lib/web-components')
const baseDir = path.join(componentsDir, 'base')

function isInside(dir: string, file: string): boolean {
  const rel = path.relative(dir, file)
  return rel !== '' && !rel.startsWith('..') && !path.isAbsolute(rel)
}

function importError(importer: string, specifier: string): string {
  return (
    `${path.relative(webRoot, importer)} imports ${specifier}, which registers a custom element. ` +
    'Page bundles must not import a component: bundle.js already registers it, and the second definition throws. ' +
    'Move page-safe code to lib/web-components/base/, or use the component in the page markup instead.'
  )
}

export const rejectComponentImports: Bun.BunPlugin = {
  name: 'reject-component-imports',
  setup(build) {
    build.onResolve({ filter: /.*/ }, (args) => {
      if (!args.importer) return

      let resolved: string
      try {
        resolved = Bun.resolveSync(args.path, path.dirname(args.importer))
      } catch {
        // Unresolved imports are Bun's error to report.
        return
      }

      if (isInside(componentsDir, resolved) && !isInside(baseDir, resolved)) {
        throw new Error(importError(args.importer, args.path))
      }
    })
  },
}
