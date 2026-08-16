import fs from 'node:fs'
import path from 'node:path'
import { pathToFileURL } from 'node:url'

const TAG_PATTERN = /^v\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/
const CARGO_VERSION_PATTERN = /^version\s*=\s*"[^"]+"/m
const JSON_VERSION_PATTERN = /^(\s*"version"\s*:\s*)"[^"]+"/m

const replaceJsonVersion = (source, version, label) => {
  const value = JSON.parse(source)
  if (typeof value.version !== 'string' || !JSON_VERSION_PATTERN.test(source)) {
    throw new Error(`${label} version was not found`)
  }
  return source.replace(JSON_VERSION_PATTERN, `$1"${version}"`)
}

export const applyReleaseVersion = (rootDirectory, tag) => {
  if (!TAG_PATTERN.test(tag)) {
    throw new Error(`Invalid release tag: ${tag || '<empty>'}`)
  }

  const version = tag.slice(1)
  const packagePath = path.join(rootDirectory, 'package.json')
  const tauriPath = path.join(rootDirectory, 'src-tauri', 'tauri.conf.json')
  const cargoPath = path.join(rootDirectory, 'src-tauri', 'Cargo.toml')

  const packageJson = fs.readFileSync(packagePath, 'utf8')
  const tauriJson = fs.readFileSync(tauriPath, 'utf8')
  const cargo = fs.readFileSync(cargoPath, 'utf8')

  if (!CARGO_VERSION_PATTERN.test(cargo)) {
    throw new Error('Cargo package version was not found')
  }

  const updatedPackageJson = replaceJsonVersion(packageJson, version, 'Package')
  const updatedTauriJson = replaceJsonVersion(tauriJson, version, 'Tauri config')
  const updatedCargo = cargo.replace(CARGO_VERSION_PATTERN, `version = "${version}"`)

  if (updatedPackageJson !== packageJson) fs.writeFileSync(packagePath, updatedPackageJson)
  if (updatedTauriJson !== tauriJson) fs.writeFileSync(tauriPath, updatedTauriJson)
  if (updatedCargo !== cargo) fs.writeFileSync(cargoPath, updatedCargo)

  return version
}

const invokedAsScript = process.argv[1]
  ? import.meta.url === pathToFileURL(fs.realpathSync(process.argv[1])).href
  : false

if (invokedAsScript) {
  const version = applyReleaseVersion(process.cwd(), process.argv[2] ?? '')
  console.log(`Applied release version ${version}`)
}
