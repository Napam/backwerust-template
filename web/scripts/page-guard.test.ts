import { afterAll, expect, test } from 'bun:test'
import fs from 'fs'
import path from 'path'

import { rejectComponentImports } from './page-guard'

// build.ts rejects page bundles that import a component. These fixtures pin
// the folder rule: support code under web-components/base/ stays importable,
// while components (directly or behind a re-export) fail with a useful message.
const webRoot = path.resolve(import.meta.dir, '..')
const fixtureDir = path.join(webRoot, 'tmp/page-guard')
const outdir = path.join(fixtureDir, 'out')
const GUARD_MESSAGE = 'a component module'

type BuildResult = { success: boolean; errors: string }

function writeFixture(name: string, source: string): string {
  const file = path.join(fixtureDir, name)
  fs.mkdirSync(path.dirname(file), { recursive: true })
  fs.writeFileSync(file, source)
  return file
}

async function build(entry: string): Promise<BuildResult> {
  const result = await Bun.build({
    entrypoints: [entry],
    outdir,
    target: 'browser',
    format: 'iife',
    throw: false,
    plugins: [rejectComponentImports],
  })

  return {
    success: result.success,
    errors: result.logs.map((log) => log.message).join('\n'),
  }
}

afterAll(() => {
  fs.rmSync(fixtureDir, { recursive: true, force: true })
})

test('base utilities stay importable', async () => {
  const entry = writeFixture(
    'base-utility.ts',
    "import { focusClass } from '../../lib/web-components/base/common'\n" +
      "import { getTailwindCSSHREF } from '../../lib/web-components/base/css'\n" +
      'console.log(focusClass, getTailwindCSSHREF())\n'
  )

  const result = await build(entry)
  expect(result.success).toBe(true)
})

test('lib/theme stays importable', async () => {
  const result = await build(writeFixture('theme.ts', "import '../../lib/theme'\n"))
  expect(result.success).toBe(true)
})

test('component import fails', async () => {
  const entry = writeFixture(
    'component.ts',
    "import { EditIcon } from '../../lib/web-components/icons/edit-icon'\n" + 'console.log(EditIcon)\n'
  )

  const result = await build(entry)
  expect(result.success).toBe(false)
  expect(result.errors).toContain(GUARD_MESSAGE)
})

test('re-exported component import fails', async () => {
  writeFixture('reexport.ts', "export { EditIcon } from '../../lib/web-components/icons/edit-icon'\n")
  const entry = writeFixture(
    'reexport-entry.ts',
    "import { EditIcon } from './reexport'\n" + 'console.log(EditIcon)\n'
  )

  const result = await build(entry)
  expect(result.success).toBe(false)
  expect(result.errors).toContain(GUARD_MESSAGE)
})
