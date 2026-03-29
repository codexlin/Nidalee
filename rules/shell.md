---
name: shell
description: Execute shell commands and open external programs
metadata:
  tags: shell, command, execute, spawn, open
---

# Shell Operations

Tauri v2 provides shell command execution and external program opening through the `@tauri-apps/plugin-shell` plugin.

## Install Plugin

```bash
# Add plugin (Rust)
cargo add tauri-plugin-shell

# Frontend install
pnpm add @tauri-apps/plugin-shell
```

### Register in Rust

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Open External Programs

### Open URL

```typescript
import { open } from '@tauri-apps/plugin-shell';

// Open URL in default browser
await open('https://tauri.app');

// Open mail client
await open('mailto:support@example.com');

// Open file (using default program)
await open('/path/to/document.pdf');
```

### Open with Specific Program

```typescript
import { open } from '@tauri-apps/plugin-shell';

// Windows: Open with specific program
await open('/path/to/file.txt', 'notepad.exe');

// macOS: Open with specific program
await open('/path/to/file.txt', 'TextEdit');

// Linux: Open with specific program
await open('/path/to/file.txt', 'gedit');
```

## Execute Shell Commands

### Simple Command Execution

```typescript
import { Command } from '@tauri-apps/plugin-shell';

// Execute command
const result = await Command.create('echo', ['Hello, World!']).execute();

console.log(result.stdout);  // "Hello, World!"
console.log(result.stderr);  // ""
console.log(result.code);    // 0
```

### Cross-Platform Commands

```typescript
import { Command } from '@tauri-apps/plugin-shell';
import { platform } from '@tauri-apps/plugin-os';

async function getSystemInfo() {
    const currentPlatform = await platform();

    let command: Command;

    switch (currentPlatform) {
        case 'windows':
            command = Command.create('systeminfo');
            break;
        case 'macos':
            command = Command.create('system_profiler', ['SPHardwareDataType']);
            break;
        case 'linux':
            command = Command.create('uname', ['-a']);
            break;
        default:
            throw new Error('Unsupported platform');
    }

    const result = await command.execute();
    return result.stdout;
}
```

### Pipe Commands

```typescript
import { Command } from '@tauri-apps/plugin-shell';

// Create pipe command
const cmd = new Command('ps', ['aux']);
const grep = new Command('grep', ['node']);

// Connect pipe
const result = await cmd.pipe(grep).execute();

console.log(result.stdout);
```

## Long-Running Commands

### Real-Time Output

```typescript
import { Command } from '@tauri-apps/plugin-shell';

const command = Command.create('ping', ['google.com']);

// Listen to stdout
command.on('close', (data) => {
    console.log('Command finished with code:', data.code);
});

command.on('error', (error) => {
    console.error('Error:', error);
});

command.stdout.on('data', (line) => {
    console.log('stdout:', line);
});

command.stderr.on('data', (line) => {
    console.error('stderr:', line);
});

// Execute command
const child = await command.spawn();

// Terminate command later
await child.kill();
```

### Process Management

```typescript
import { Command } from '@tauri-apps/plugin-shell';

class ProcessManager {
    private processes: Map<string, any> = new Map();

    async startProcess(id: string, command: string, args: string[]) {
        const cmd = Command.create(command, args);

        cmd.on('close', () => {
            this.processes.delete(id);
        });

        const child = await cmd.spawn();
        this.processes.set(id, child);

        return child;
    }

    async stopProcess(id: string) {
        const process = this.processes.get(id);
        if (process) {
            await process.kill();
            this.processes.delete(id);
        }
    }

    async stopAll() {
        for (const [id, process] of this.processes) {
            await process.kill();
        }
        this.processes.clear();
    }
}
```

## Environment Variables

```typescript
import { Command } from '@tauri-apps/plugin-shell';

// Set environment variables
const command = Command.create('node', ['script.js']);

// Set environment variables in Rust
#[tauri::command]
fn run_with_env(command: String, env_vars: HashMap<String, String>) -> Result<String, String> {
    let output = std::process::Command::new(command)
        .envs(env_vars)
        .output()
        .map_err(|e| e.to_string())?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

## Working Directory

```typescript
import { Command } from '@tauri-apps/plugin-shell';

// Execute command in specific directory
const command = Command.create('git', ['status']);
command.cwd = '/path/to/repo';

const result = await command.execute();
```

## Permission Configuration

Configure shell permissions in `capabilities/default.json`:

```json
{
  "permissions": [
    {
      "identifier": "shell:default"
    },
    {
      "identifier": "shell:allow-open"
    },
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "echo",
          "cmd": "echo",
          "args": true
        },
        {
          "name": "git",
          "cmd": "git",
          "args": ["status", "log", "pull", "push"]
        }
      ]
    }
  ]
}
```

## Complete Example

```typescript
import { Command, open } from '@tauri-apps/plugin-shell';
import { BaseDirectory, writeTextFile } from '@tauri-apps/plugin-fs';

class GitManager {
    private repoPath: string;

    constructor(repoPath: string) {
        this.repoPath = repoPath;
    }

    async status(): Promise<string> {
        const result = await this.runGitCommand(['status']);
        return result.stdout;
    }

    async log(limit: number = 10): Promise<string> {
        const result = await this.runGitCommand(['log', `--max-count=${limit}`, '--oneline']);
        return result.stdout;
    }

    async pull(): Promise<string> {
        const result = await this.runGitCommand(['pull']);
        return result.stdout;
    }

    async push(): Promise<string> {
        const result = await this.runGitCommand(['push']);
        return result.stdout;
    }

    private async runGitCommand(args: string[]) {
        const command = Command.create('git', args);
        command.cwd = this.repoPath;

        const result = await command.execute();

        if (result.code !== 0) {
            throw new Error(`Git command failed: ${result.stderr}`);
        }

        return result;
    }

    async openInEditor(): Promise<void> {
        await open(this.repoPath);
    }
}

// Usage example
const git = new GitManager('/path/to/repo');
const status = await git.status();
console.log(status);
```

## Security Considerations

1. **Command Injection**: Never directly concatenate user input into commands
2. **Permission Control**: Explicitly allow commands in capabilities
3. **Argument Validation**: Validate all command arguments
4. **Timeout Settings**: Set timeouts for long-running commands
5. **Resource Cleanup**: Ensure processes are properly terminated

## Best Practices

1. **Use Command Class**: Instead of directly concatenating strings
2. **Error Handling**: Check command return codes and error output
3. **Async Execution**: Avoid blocking the main thread
4. **Logging**: Log executed commands and output
5. **Resource Limits**: Limit command execution time and resources