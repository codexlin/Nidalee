import assert from 'node:assert/strict'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'

import { parseReleaseCommits, renderReleaseNotes, updateUpdaterMetadata } from './generate-release-notes.mjs'

test('groups user-facing conventional commits and ignores merge or maintenance commits', () => {
  const groups = parseReleaseCommits([
    'feat(ui): add update details',
    'fix: restore matchmaking state',
    'refactor(ui):streamline-update-notifications',
    'chore: bump dependencies',
    "Merge branch 'feature/example'"
  ])

  assert.deepEqual(groups.get('新增功能'), ['add update details'])
  assert.deepEqual(groups.get('问题修复'), ['restore matchmaking state'])
  assert.deepEqual(groups.get('体验优化'), ['streamline update notifications'])
  assert.equal(groups.has('chore'), false)
})

test('renders readable notes with a comparison link', () => {
  const notes = renderReleaseNotes({
    tag: 'v1.0.2',
    previousTag: 'v1.0.1',
    subjects: ['fix(updater): include release notes'],
    repositoryUrl: 'https://github.com/codexlin/Nidalee'
  })

  assert.match(notes, /## 更新内容/)
  assert.match(notes, /### 问题修复\n\n- include release notes/)
  assert.match(notes, /compare\/v1\.0\.1\.\.\.v1\.0\.2/)
})

test('writes release notes and public asset URLs into updater metadata', (context) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'nidalee-updater-notes-'))
  context.after(() => fs.rmSync(directory, { recursive: true, force: true }))
  const updaterPath = path.join(directory, 'latest.json')
  const apiUrl = 'https://api.github.com/repos/codexlin/Nidalee/releases/assets/123'
  const publicUrl = 'https://github.com/codexlin/Nidalee/releases/download/v1.0.2/Nidalee.msi'
  fs.writeFileSync(
    updaterPath,
    JSON.stringify({ version: '1.0.2', notes: '', platforms: { 'windows-x86_64': { url: apiUrl } } })
  )

  updateUpdaterMetadata(updaterPath, '## 更新内容\n\n- 修复更新说明', [{ apiUrl, url: publicUrl }])

  const updater = JSON.parse(fs.readFileSync(updaterPath, 'utf8'))
  assert.equal(updater.notes, '## 更新内容\n\n- 修复更新说明')
  assert.equal(updater.platforms['windows-x86_64'].url, publicUrl)
})

test('rejects updater metadata that still contains a GitHub API asset URL', (context) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'nidalee-updater-url-'))
  context.after(() => fs.rmSync(directory, { recursive: true, force: true }))
  const updaterPath = path.join(directory, 'latest.json')
  fs.writeFileSync(
    updaterPath,
    JSON.stringify({
      version: '1.0.2',
      platforms: {
        'windows-x86_64': {
          url: 'https://api.github.com/repos/codexlin/Nidalee/releases/assets/123'
        }
      }
    })
  )

  assert.throws(() => updateUpdaterMetadata(updaterPath, 'notes'), /Updater platform still uses a GitHub API asset URL/)
})
