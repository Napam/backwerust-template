import { Glob } from 'bun'
import fs from 'fs'
import path from 'path'

// Static URLs in the Rust views are plain strings, so a renamed asset or page
// file leaves a 404 that no compiler or test catches. This runs after the web
// build and asserts every /static/... literal resolves to a served file under
// OUTDIR.
//
// Run from web/ via `task check.web`; OUTDIR matches the build's output dir.
const webRoot = path.resolve(import.meta.dir, '..')
const repoRoot = path.resolve(webRoot, '..')
const outdir = path.resolve(webRoot, process.env.OUTDIR ?? 'static')
const viewsDir = path.join(repoRoot, 'src/views')

const STATIC_REF = /\/static\/[\w./-]+/g
// Only quoted string literals hold served URLs. Scanning raw source would also
// match commented-out paths and fail on code that is not part of a response.
const STRING_LITERAL = /"(?:[^"\\]|\\.)*"/g

if (!fs.existsSync(viewsDir)) {
  console.error(`static refs: views directory not found: ${viewsDir}`)
  process.exit(1)
}

const viewFiles = Array.from(new Glob('**/*.rs').scanSync(viewsDir))
const missing: string[] = []
let checked = 0

if (viewFiles.length === 0) {
  console.error(`static refs: no view files found under ${viewsDir}`)
  process.exit(1)
}

for (const file of viewFiles) {
  const source = fs.readFileSync(path.join(viewsDir, file), 'utf8')

  for (const literal of source.matchAll(STRING_LITERAL)) {
    for (const match of literal[0].matchAll(STATIC_REF)) {
      const url = match[0]
      const index = (literal.index ?? 0) + (match.index ?? 0)
      const line = source.slice(0, index).split('\n').length
      const rel = path.normalize(url.slice('/static/'.length))
      const target = path.join(outdir, rel)
      checked++

      // Reject traversal and absolute paths: the assets route only serves
      // files under the static root. `statSync().isFile()` also rejects
      // directories, which the route never serves.
      const escapes = rel === '..' || rel.startsWith(`..${path.sep}`) || path.isAbsolute(rel)
      let resolves = false
      if (!escapes) {
        try {
          resolves = fs.statSync(target).isFile()
        } catch {
          // Missing file: reported below.
        }
      }

      if (!resolves) {
        missing.push(`${url} (src/views/${file}:${line})`)
      }
    }
  }
}

if (checked === 0) {
  console.error(`static refs: no /static/... URLs found under ${viewsDir}; the guard has no coverage`)
  process.exit(1)
}

if (missing.length > 0) {
  console.error(`static refs: ${missing.length} of ${checked} URLs do not resolve to a file under ${outdir}:`)
  for (const ref of missing) {
    console.error(`  ${ref}`)
  }
  process.exit(1)
}

console.log(`static refs: ${checked} URLs resolve under ${outdir}`)
