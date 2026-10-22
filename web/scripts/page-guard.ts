import fs from 'fs'
import path from 'path'
import ts from 'typescript'

// bundle.js is built from lib/**/*.ts, so every custom element registered
// anywhere under lib/ is already defined when a page bundle runs. A page that
// imports that module again throws NotSupportedError. Detect the definition
// itself rather than the directory: lib/web-components also holds utilities
// and base classes that are safe to import.
//
// Parse the module instead of grepping raw source: a comment or string that
// mentions the API is not a definition, and an aliased decorator or a
// destructured `define` still is. Reading the API is not defining either, so
// `typeof customElements.define` and `const define = customElements.define`
// stay safe.
const webRoot = path.resolve(import.meta.dir, '..')
const libDir = path.join(webRoot, 'lib')
const sourceExtensions = new Set(['.ts', '.tsx', '.js', '.jsx', '.mjs', '.cjs'])
const customElementsRoots = new Set(['window', 'globalThis', 'self'])
const forwardingMethods = new Set(['bind', 'call', 'apply'])

function isInside(dir: string, file: string): boolean {
  const rel = path.relative(dir, file)
  return rel !== '' && !rel.startsWith('..') && !path.isAbsolute(rel)
}

/** True for `customElements` and the global roots that expose it. */
function isCustomElements(node: ts.Expression): boolean {
  if (ts.isIdentifier(node)) return node.text === 'customElements'
  return (
    ts.isPropertyAccessExpression(node) &&
    node.name.text === 'customElements' &&
    ts.isIdentifier(node.expression) &&
    customElementsRoots.has(node.expression.text)
  )
}

/** True for `const { define } = customElements` and its renamed form. */
function destructuresDefine(node: ts.VariableDeclaration): boolean {
  if (!ts.isObjectBindingPattern(node.name) || !node.initializer) return false
  if (!isCustomElements(node.initializer)) return false

  return node.name.elements.some((element) => {
    const property = element.propertyName ?? element.name
    return (ts.isIdentifier(property) || ts.isStringLiteral(property)) && property.text === 'define'
  })
}

/** Decorator names come from `customElement` imports, including aliases. */
function customElementNames(source: ts.SourceFile): Set<string> {
  const names = new Set(['customElement'])
  const declarations: ts.VariableDeclaration[] = []

  function collect(node: ts.Node) {
    if (ts.isImportDeclaration(node) && node.importClause?.namedBindings) {
      const bindings = node.importClause.namedBindings
      if (ts.isNamedImports(bindings)) {
        for (const element of bindings.elements) {
          if ((element.propertyName ?? element.name).text === 'customElement') {
            names.add(element.name.text)
          }
        }
      }
    } else if (ts.isVariableDeclaration(node)) {
      declarations.push(node)
    }
    ts.forEachChild(node, collect)
  }
  collect(source)

  // `const ce = customElement` aliases can chain, so repeat until nothing new.
  let added = true
  while (added) {
    added = false
    for (const declaration of declarations) {
      if (!ts.isIdentifier(declaration.name) || !declaration.initializer) continue
      if (!ts.isIdentifier(declaration.initializer)) continue
      if (names.has(declaration.initializer.text) && !names.has(declaration.name.text)) {
        names.add(declaration.name.text)
        added = true
      }
    }
  }

  return names
}

function decoratorDefinesElement(expression: ts.Expression, names: Set<string>): boolean {
  const target = ts.isCallExpression(expression) ? expression.expression : expression
  if (ts.isIdentifier(target)) return names.has(target.text)
  return ts.isPropertyAccessExpression(target) && target.name.text === 'customElement'
}

/**
 * True for `customElements.define` and `customElements['define']` when the
 * access defines an element: called, assigned, or forwarded with `.bind`,
 * `.call`, or `.apply`. A plain read leaves the registry alone.
 */
function isDefineAccess(node: ts.Node): boolean {
  if (ts.isPropertyAccessExpression(node)) {
    return node.name.text === 'define' && isCustomElements(node.expression)
  }

  return (
    ts.isElementAccessExpression(node) &&
    ts.isStringLiteral(node.argumentExpression) &&
    node.argumentExpression.text === 'define' &&
    isCustomElements(node.expression)
  )
}

function accessDefinesElement(node: ts.Node): boolean {
  let expression = node
  while (
    ts.isParenthesizedExpression(expression.parent) ||
    ts.isAsExpression(expression.parent) ||
    ts.isNonNullExpression(expression.parent)
  ) {
    expression = expression.parent
  }

  const parent = expression.parent
  if (ts.isCallExpression(parent) && parent.expression === expression) return true
  if (
    ts.isBinaryExpression(parent) &&
    parent.left === expression &&
    parent.operatorToken.kind === ts.SyntaxKind.EqualsToken
  ) {
    return true
  }

  return (
    ts.isPropertyAccessExpression(parent) &&
    parent.expression === expression &&
    forwardingMethods.has(parent.name.text) &&
    ts.isCallExpression(parent.parent) &&
    parent.parent.expression === parent
  )
}

function sourceDefinesCustomElement(file: string, source: string): boolean {
  const parsed = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true)
  const names = customElementNames(parsed)
  let found = false

  function visit(node: ts.Node) {
    if (found) return

    if (ts.canHaveDecorators(node)) {
      for (const decorator of ts.getDecorators(node) ?? []) {
        if (decoratorDefinesElement(decorator.expression, names)) {
          found = true
          return
        }
      }
    }

    if (
      (isDefineAccess(node) && accessDefinesElement(node)) ||
      (ts.isVariableDeclaration(node) && destructuresDefine(node))
    ) {
      found = true
      return
    }

    ts.forEachChild(node, visit)
  }

  visit(parsed)
  return found
}

export function definesCustomElement(file: string, libRoot = libDir): boolean {
  // Declaration files emit no runtime code, so a page cannot re-register
  // anything by importing one. bundleLib() filters them for the same reason.
  const isSource = sourceExtensions.has(path.extname(file)) && !file.endsWith('.d.ts')
  if (!isInside(libRoot, file) || !isSource) {
    return false
  }

  try {
    return sourceDefinesCustomElement(file, fs.readFileSync(file, 'utf8'))
  } catch {
    return false
  }
}

function importError(importer: string, specifier: string, libRoot: string): string {
  const from = path.relative(webRoot, importer)
  const hint = isInside(libRoot, importer)
    ? 'Remove the re-export, or split the page-safe code into its own module.'
    : 'Use the component in the page markup instead.'

  return (
    `${from} imports ${specifier}, which defines a custom element. ` +
    'Page bundles must not import a component: bundle.js already registers it, and the second definition throws. ' +
    hint
  )
}

export function createRejectComponentImports(libRoot = libDir): Bun.BunPlugin {
  return {
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

        if (definesCustomElement(resolved, libRoot)) {
          throw new Error(importError(args.importer, args.path, libRoot))
        }
      })
    },
  }
}

export const rejectComponentImports = createRejectComponentImports()
