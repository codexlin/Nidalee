import fs from 'node:fs'

const tag = process.argv[2] ?? ''

if (!/^v\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(tag)) {
  throw new Error(`Invalid release tag: ${tag || '<empty>'}`)
}

const version = tag.slice(1)

const updateJsonVersion = (path) => {
  const value = JSON.parse(fs.readFileSync(path, 'utf8'))
  value.version = version
  fs.writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`)
}

updateJsonVersion('package.json')
updateJsonVersion('src-tauri/tauri.conf.json')

const cargoPath = 'src-tauri/Cargo.toml'
const cargo = fs.readFileSync(cargoPath, 'utf8')
const updatedCargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`)

if (updatedCargo === cargo) {
  throw new Error('Cargo package version was not found')
}

fs.writeFileSync(cargoPath, updatedCargo)
console.log(`Applied release version ${version}`)
