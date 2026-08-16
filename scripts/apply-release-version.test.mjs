import assert from 'node:assert/strict'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { applyReleaseVersion } from './apply-release-version.mjs'

const createFixture = (version) => {
  const rootDirectory = fs.mkdtempSync(path.join(os.tmpdir(), 'nidalee-release-version-'))
  const tauriDirectory = path.join(rootDirectory, 'src-tauri')
  fs.mkdirSync(tauriDirectory)
  fs.writeFileSync(
    path.join(rootDirectory, 'package.json'),
    `${JSON.stringify({ name: 'nidalee', version }, null, 2)}\n`
  )
  fs.writeFileSync(
    path.join(tauriDirectory, 'tauri.conf.json'),
    `${JSON.stringify({ productName: 'Nidalee', version }, null, 2)}\n`
  )
  fs.writeFileSync(
    path.join(tauriDirectory, 'Cargo.toml'),
    `[package]\nname = "nidalee"\nversion = "${version}"\n\n[dependencies]\nexample = { version = "9.9.9" }\n`
  )
  return rootDirectory
}

const assertVersions = (rootDirectory, expectedVersion) => {
  const packageJson = JSON.parse(fs.readFileSync(path.join(rootDirectory, 'package.json'), 'utf8'))
  const tauriJson = JSON.parse(
    fs.readFileSync(path.join(rootDirectory, 'src-tauri', 'tauri.conf.json'), 'utf8')
  )
  const cargo = fs.readFileSync(path.join(rootDirectory, 'src-tauri', 'Cargo.toml'), 'utf8')

  assert.equal(packageJson.version, expectedVersion)
  assert.equal(tauriJson.version, expectedVersion)
  assert.match(cargo, new RegExp(`^version = "${expectedVersion.replaceAll('.', '\\.')}"$`, 'm'))
  assert.match(cargo, /example = \{ version = "9\.9\.9" \}/)
}

test('accepts an already-applied release version', (context) => {
  const rootDirectory = createFixture('1.0.0')
  context.after(() => fs.rmSync(rootDirectory, { recursive: true, force: true }))
  const paths = [
    path.join(rootDirectory, 'package.json'),
    path.join(rootDirectory, 'src-tauri', 'tauri.conf.json'),
    path.join(rootDirectory, 'src-tauri', 'Cargo.toml')
  ]
  const originalContents = paths.map((filePath) => fs.readFileSync(filePath, 'utf8'))

  assert.equal(applyReleaseVersion(rootDirectory, 'v1.0.0'), '1.0.0')
  assertVersions(rootDirectory, '1.0.0')
  assert.deepEqual(
    paths.map((filePath) => fs.readFileSync(filePath, 'utf8')),
    originalContents
  )
})

test('updates every release version source without touching dependency versions', (context) => {
  const rootDirectory = createFixture('1.0.0')
  context.after(() => fs.rmSync(rootDirectory, { recursive: true, force: true }))

  assert.equal(applyReleaseVersion(rootDirectory, 'v1.1.0'), '1.1.0')
  assertVersions(rootDirectory, '1.1.0')
})
