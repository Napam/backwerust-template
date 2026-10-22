import fs from 'fs'
import path from 'path'

import { createRejectComponentImports, rejectComponentImports } from './page-guard'

// build.ts rejects page bundles that import a module defining a custom
// element. These fixtures pin both halves: utilities under web-components stay
// importable, while components (directly or behind a re-export) fail. They
// also pin the source analysis: comments, strings, and plain reads of the API
// do not count, while a bound, destructured, or aliased definition does.
const webRoot = path.resolve(import.meta.dir, '..')
const fixtureDir = path.join(webRoot, 'tmp/page-guard')
const fixtureLibDir = path.join(fixtureDir, 'lib')
const outdir = path.join(fixtureDir, 'out')
const fixtureGuard = createRejectComponentImports(fixtureLibDir)

type BuildResult = { success: boolean; errors: string }

async function runBuild(entry: string, plugin: Bun.BunPlugin = rejectComponentImports): Promise<BuildResult> {
  const result = await Bun.build({
    entrypoints: [entry],
    outdir,
    target: 'browser',
    format: 'iife',
    throw: false,
    plugins: [plugin],
  })

  return {
    success: result.success,
    errors: result.logs.map((log) => log.message).join('\n'),
  }
}

function writeFixture(name: string, source: string): string {
  const file = path.join(fixtureDir, name)
  fs.mkdirSync(path.dirname(file), { recursive: true })
  fs.writeFileSync(file, source)
  return file
}

const failures: string[] = []

function expect(name: string, result: BuildResult, wantSuccess: boolean) {
  if (result.success === wantSuccess) return

  const outcome = wantSuccess ? 'failed to build' : 'built but should have failed'
  failures.push(`${name} ${outcome}${result.errors ? `:\n${result.errors}` : ''}`)
}

fs.rmSync(fixtureDir, { recursive: true, force: true })

const utility = await runBuild(
  writeFixture(
    'utility.ts',
    "import { focusClass } from '../../lib/web-components/common'\n" +
      "import { getTailwindCSSHREF } from '../../lib/web-components/utils/css'\n" +
      'console.log(focusClass, getTailwindCSSHREF())\n'
  )
)
expect('utility import', utility, true)

const theme = await runBuild(writeFixture('theme.ts', "import '../../lib/theme'\n"))
expect('lib/theme import', theme, true)

const component = await runBuild(
  writeFixture(
    'component.ts',
    "import { EditIcon } from '../../lib/web-components/icons/edit-icon'\n" + 'console.log(EditIcon)\n'
  )
)
expect('component import', component, false)
if (!component.success && !component.errors.includes('defines a custom element')) {
  failures.push(`component import failed without the guard message:\n${component.errors}`)
}

writeFixture('reexport.ts', "export { EditIcon } from '../../lib/web-components/icons/edit-icon'\n")
const reexport = await runBuild(
  writeFixture('reexport-entry.ts', "import { EditIcon } from './reexport'\n" + 'console.log(EditIcon)\n')
)
expect('re-exported component import', reexport, false)
if (!reexport.success && !reexport.errors.includes('defines a custom element')) {
  failures.push(`re-exported component failed without the guard message:\n${reexport.errors}`)
}

// Comments and strings are not definitions: a utility that merely mentions
// the API must stay importable.
writeFixture(
  'lib/safe-utility.ts',
  '// This utility must never call customElements.define() itself.\n' +
    '// Nor apply @customElement("fake").\n' +
    "export const warning = 'customElements.define('\n" +
    'export const markup = `@customElement("fake")`\n' +
    'export function message() {\n' +
    '  return warning + markup\n' +
    '}\n'
)
const safeUtility = await runBuild(
  writeFixture(
    'safe-utility.ts',
    "import { message } from './lib/safe-utility'\n" + 'console.log(message())\n'
  ),
  fixtureGuard
)
expect('utility mentioning define in comments and strings', safeUtility, true)

// Reading the API is not defining an element. A feature-detection utility, an
// exported reference, and an unrelated object with a `customElements` property
// must all stay importable.
writeFixture(
  'lib/reads-define.ts',
  'export const canDefine = typeof customElements.define === "function"\n' +
    'export const define = customElements.define\n' +
    'export const readDefine = customElements.define.bind\n'
)
const readsDefine = await runBuild(
  writeFixture(
    'reads-define-page.ts',
    "import { canDefine, define, readDefine } from './lib/reads-define'\n" +
      'console.log(canDefine, define, readDefine)\n'
  ),
  fixtureGuard
)
expect('utility reading customElements.define', readsDefine, true)

writeFixture(
  'lib/unrelated-custom-elements.ts',
  'const registry = { customElements: { define() {} } }\n' +
    'export function register(name: string) {\n' +
    '  registry.customElements.define(name)\n' +
    '}\n'
)
const unrelatedCustomElements = await runBuild(
  writeFixture(
    'unrelated-custom-elements-page.ts',
    "import { register } from './lib/unrelated-custom-elements'\n" + "register('not-an-element')\n"
  ),
  fixtureGuard
)
expect('unrelated customElements receiver', unrelatedCustomElements, true)

writeFixture(
  'lib/window-define.ts',
  "window.customElements.define('window-el', class extends HTMLElement {})\n"
)
const windowDefine = await runBuild(
  writeFixture('window-define-page.ts', "import './lib/window-define'\n"),
  fixtureGuard
)
expect('window.customElements define', windowDefine, false)
if (!windowDefine.success && !windowDefine.errors.includes('defines a custom element')) {
  failures.push(`window define failed without the guard message:\n${windowDefine.errors}`)
}

// The definition can hide behind a binding, so the guard follows the common
// indirect forms instead of only matching customElements.define(.
writeFixture(
  'lib/bound-define.ts',
  'const define = customElements.define.bind(customElements)\n' +
    "define('bound-el', class extends HTMLElement {})\n"
)
const boundDefine = await runBuild(
  writeFixture('bound-define-page.ts', "import './lib/bound-define'\n"),
  fixtureGuard
)
expect('bound define', boundDefine, false)
if (!boundDefine.success && !boundDefine.errors.includes('defines a custom element')) {
  failures.push(`bound define failed without the guard message:\n${boundDefine.errors}`)
}

writeFixture(
  'lib/destructured-define.ts',
  'const { define } = customElements\n' +
    'const { define: register } = customElements\n' +
    "define('destructured-el', class extends HTMLElement {})\n" +
    "register('renamed-el', class extends HTMLElement {})\n"
)
const destructuredDefine = await runBuild(
  writeFixture('destructured-define-page.ts', "import './lib/destructured-define'\n"),
  fixtureGuard
)
expect('destructured define', destructuredDefine, false)
if (!destructuredDefine.success && !destructuredDefine.errors.includes('defines a custom element')) {
  failures.push(`destructured define failed without the guard message:\n${destructuredDefine.errors}`)
}

writeFixture(
  'lib/aliased-decorator.ts',
  "import { customElement as ce } from 'lit/decorators.js'\n" +
    "@ce('aliased-el')\n" +
    'class AliasedElement extends HTMLElement {}\n'
)
const aliasedDecorator = await runBuild(
  writeFixture('aliased-decorator-page.ts', "import './lib/aliased-decorator'\n"),
  fixtureGuard
)
expect('aliased decorator', aliasedDecorator, false)
if (!aliasedDecorator.success && !aliasedDecorator.errors.includes('defines a custom element')) {
  failures.push(`aliased decorator failed without the guard message:\n${aliasedDecorator.errors}`)
}

if (failures.length > 0) {
  console.error(`page guard: ${failures.length} fixture(s) behaved unexpectedly:`)
  for (const failure of failures) {
    console.error(`  ${failure}`)
  }
  process.exit(1)
}

console.log('page guard: fixtures behave as expected')
