import fs from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const generatedDir = path.resolve(scriptDir, '../src/types/generated')

await fs.rm(generatedDir, { recursive: true, force: true })
await fs.mkdir(generatedDir, { recursive: true })

console.log(`已清理 TypeScript 生成目录：${generatedDir}`)
