import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const TYPE_LABELS = new Map([
  ['feat', '新增功能'],
  ['fix', '问题修复'],
  ['perf', '性能优化'],
  ['refactor', '体验优化'],
  ['docs', '文档更新']
])

export function parseReleaseCommits(subjects) {
  const groups = new Map()

  for (const subject of subjects) {
    if (/^Merge\b/i.test(subject)) continue

    const match = /^(feat|fix|perf|refactor|docs)(?:\([^)]+\))?!?:\s*(.+)$/i.exec(subject)
    if (!match) continue

    const label = TYPE_LABELS.get(match[1].toLowerCase())
    const rawDescription = match[2].trim()
    const description = /\s/.test(rawDescription) ? rawDescription : rawDescription.replace(/[-_]+/g, ' ')
    if (!label || !description) continue

    const entries = groups.get(label) ?? []
    if (!entries.includes(description)) entries.push(description)
    groups.set(label, entries)
  }

  return groups
}

export function renderReleaseNotes({ tag, previousTag, subjects, repositoryUrl }) {
  const groups = parseReleaseCommits(subjects)
  const lines = ['## 更新内容', '']

  for (const label of TYPE_LABELS.values()) {
    const entries = groups.get(label)
    if (!entries?.length) continue

    lines.push(`### ${label}`, '', ...entries.map((entry) => `- ${entry}`), '')
  }

  if (groups.size === 0) {
    lines.push('- 包含稳定性改进与问题修复。', '')
  }

  if (previousTag) {
    lines.push(`[查看完整变更](${repositoryUrl}/compare/${previousTag}...${tag})`)
  } else {
    lines.push(`[查看完整提交](${repositoryUrl}/commits/${tag})`)
  }

  return `${lines.join('\n').trim()}\n`
}

export function updateUpdaterMetadata(updaterPath, notes, tag, repositoryUrl, releaseAssets = []) {
  const updater = JSON.parse(fs.readFileSync(updaterPath, 'utf8'))
  const assetsBySourceUrl = new Map()

  for (const asset of releaseAssets) {
    if (asset.apiUrl) assetsBySourceUrl.set(asset.apiUrl, asset)
    if (asset.url) assetsBySourceUrl.set(asset.url, asset)
  }

  for (const platform of Object.values(updater.platforms ?? {})) {
    const sourceUrl = platform.url
    const sourceName = sourceUrl ? decodeURIComponent(new URL(sourceUrl).pathname.split('/').at(-1) ?? '') : ''
    const asset = assetsBySourceUrl.get(sourceUrl) ?? releaseAssets.find((candidate) => candidate.name === sourceName)

    if (asset?.name) {
      platform.url = `${repositoryUrl}/releases/download/${encodeURIComponent(tag)}/${encodeURIComponent(asset.name)}`
    }

    if (platform.url?.startsWith('https://api.github.com/repos/') || platform.url?.includes('/download/untagged-')) {
      throw new Error(`Updater platform still uses a non-public GitHub asset URL: ${platform.url}`)
    }
  }

  updater.notes = notes.trim()
  fs.writeFileSync(updaterPath, `${JSON.stringify(updater, null, 2)}\n`)
}

function git(rootDirectory, args) {
  return execFileSync('git', args, { cwd: rootDirectory, encoding: 'utf8' }).trim()
}

export function generateReleaseNotes(rootDirectory, tag) {
  let previousTag = null
  try {
    previousTag = git(rootDirectory, ['describe', '--tags', '--abbrev=0', `${tag}^`]) || null
  } catch {
    // The first release has no preceding tag.
  }
  const range = previousTag ? `${previousTag}..${tag}` : tag
  const subjects = git(rootDirectory, ['log', '--format=%s', range]).split(/\r?\n/).filter(Boolean)
  const repository = process.env.GITHUB_REPOSITORY ?? 'codexlin/Nidalee'
  const serverUrl = process.env.GITHUB_SERVER_URL ?? 'https://github.com'

  return renderReleaseNotes({
    tag,
    previousTag,
    subjects,
    repositoryUrl: `${serverUrl}/${repository}`
  })
}

const isDirectRun = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)

if (isDirectRun) {
  const [, , tag, notesPath, updaterPath, releaseAssetsPath] = process.argv
  if (!tag || !notesPath) {
    throw new Error(
      'Usage: node scripts/generate-release-notes.mjs <tag> <notes-path> [latest-json-path] [release-assets-json-path]'
    )
  }

  const rootDirectory = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
  const repository = process.env.GITHUB_REPOSITORY ?? 'codexlin/Nidalee'
  const serverUrl = process.env.GITHUB_SERVER_URL ?? 'https://github.com'
  const repositoryUrl = `${serverUrl}/${repository}`
  const notes = generateReleaseNotes(rootDirectory, tag)
  fs.writeFileSync(path.resolve(notesPath), notes)
  if (updaterPath) {
    const releaseAssets = releaseAssetsPath
      ? JSON.parse(fs.readFileSync(path.resolve(releaseAssetsPath), 'utf8')).assets
      : []
    updateUpdaterMetadata(path.resolve(updaterPath), notes, tag, repositoryUrl, releaseAssets)
  }
}
