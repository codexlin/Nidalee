---
name: frontend-integration
description: Integration with frontend frameworks
metadata:
  tags: frontend, react, vue, svelte, integration
---

# Frontend Framework Integration

Tauri v2 seamlessly integrates with various frontend frameworks including React, Vue, Svelte, and more.

## Create Project

### Using create-tauri-app

```bash
# Interactive creation
pnpm create tauri-app@latest

# Use specific template
pnpm create tauri-app@latest -- --template vite-react-ts
pnpm create tauri-app@latest -- --template vite-vue-ts
pnpm create tauri-app@latest -- --template vite-svelte-ts
pnpm create tauri-app@latest -- --template nextjs-ts
```

### Available Templates

- `vanilla` / `vanilla-ts` - Plain JavaScript/TypeScript
- `vue` / `vue-ts` - Vue 3
- `react` / `react-ts` - React
- `svelte` / `svelte-ts` - Svelte
- `solid` / `solid-ts` - SolidJS
- `angular` - Angular
- `preact` / `preact-ts` - Preact
- `next` / `next-ts` - Next.js

## React Integration

### Project Structure

```
my-tauri-app/
├── src/                    # React source code
│   ├── App.tsx
│   ├── main.tsx
│   └── components/
├── src-tauri/             # Tauri backend
│   ├── src/
│   │   └── main.rs
│   └── Cargo.toml
├── package.json
├── vite.config.ts
└── tsconfig.json
```

### Call Tauri API

```tsx
// src/App.tsx
import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';

function App() {
    const [content, setContent] = useState('');

    const handleOpenFile = async () => {
        const filePath = await open({
            filters: [{ name: 'Text', extensions: ['txt'] }]
        });

        if (filePath) {
            const text = await readTextFile(filePath);
            setContent(text);
        }
    };

    const handleGreet = async () => {
        const response = await invoke('greet', { name: 'React' });
        console.log(response);
    };

    return (
        <div className="app">
            <h1>My Tauri App</h1>
            <button onClick={handleOpenFile}>Open File</button>
            <button onClick={handleGreet}>Greet</button>
            <pre>{content}</pre>
        </div>
    );
}

export default App;
```

### React Hooks Wrapper

```tsx
// src/hooks/useTauri.ts
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, Event } from '@tauri-apps/api/event';

export function useInvoke<T>(command: string, args?: Record<string, unknown>) {
    const [data, setData] = useState<T | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);

    const execute = useCallback(async () => {
        setLoading(true);
        setError(null);

        try {
            const result = await invoke<T>(command, args);
            setData(result);
            return result;
        } catch (err) {
            setError(err as Error);
            throw err;
        } finally {
            setLoading(false);
        }
    }, [command, args]);

    return { data, loading, error, execute };
}

export function useTauriEvent<T>(eventName: string, handler: (event: Event<T>) => void) {
    useEffect(() => {
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            unlisten = await listen<T>(eventName, handler);
        };

        setupListener();

        return () => {
            unlisten?.();
        };
    }, [eventName, handler]);
}
```

## Vue Integration

### Project Structure

```
my-tauri-app/
├── src/
│   ├── App.vue
│   ├── main.ts
│   ├── components/
│   └── composables/
├── src-tauri/
├── package.json
└── vite.config.ts
```

### Call Tauri API

```vue
<!-- src/App.vue -->
<template>
    <div class="app">
        <h1>My Tauri App</h1>
        <button @click="openFile">Open File</button>
        <button @click="greet">Greet</button>
        <pre>{{ content }}</pre>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';

const content = ref('');

const openFile = async () => {
    const filePath = await open({
        filters: [{ name: 'Text', extensions: ['txt'] }]
    });

    if (filePath) {
        content.value = await readTextFile(filePath);
    }
};

const greet = async () => {
    const response = await invoke('greet', { name: 'Vue' });
    console.log(response);
};
</script>
```

### Vue Composables

```typescript
// src/composables/useTauri.ts
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, Event } from '@tauri-apps/api/event';

export function useInvoke<T>(command: string, args?: Record<string, unknown>) {
    const data = ref<T | null>(null);
    const loading = ref(false);
    const error = ref<Error | null>(null);

    const execute = async () => {
        loading.value = true;
        error.value = null;

        try {
            const result = await invoke<T>(command, args);
            data.value = result;
            return result;
        } catch (err) {
            error.value = err as Error;
            throw err;
        } finally {
            loading.value = false;
        }
    };

    return { data, loading, error, execute };
}

export function useTauriEvent<T>(eventName: string, handler: (event: Event<T>) => void) {
    let unlisten: (() => void) | undefined;

    onMounted(async () => {
        unlisten = await listen<T>(eventName, handler);
    });

    onUnmounted(() => {
        unlisten?.();
    });
}
```

## Svelte Integration

### Project Structure

```
my-tauri-app/
├── src/
│   ├── App.svelte
│   ├── main.ts
│   ├── components/
│   └── stores/
├── src-tauri/
├── package.json
└── vite.config.ts
```

### Call Tauri API

```svelte
<!-- src/App.svelte -->
<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { open } from '@tauri-apps/plugin-dialog';
    import { readTextFile } from '@tauri-apps/plugin-fs';

    let content = '';

    async function handleOpenFile() {
        const filePath = await open({
            filters: [{ name: 'Text', extensions: ['txt'] }]
        });

        if (filePath) {
            content = await readTextFile(filePath);
        }
    }

    async function handleGreet() {
        const response = await invoke('greet', { name: 'Svelte' });
        console.log(response);
    }
</script>

<div class="app">
    <h1>My Tauri App</h1>
    <button on:click={handleOpenFile}>Open File</button>
    <button on:click={handleGreet}>Greet</button>
    <pre>{content}</pre>
</div>
```

### Svelte Stores

```typescript
// src/stores/tauri.ts
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen, Event } from '@tauri-apps/api/event';

export function createInvokeStore<T>(command: string, args?: Record<string, unknown>) {
    const { subscribe, set } = writable<T | null>(null);
    const loading = writable(false);
    const error = writable<Error | null>(null);

    async function execute() {
        loading.set(true);
        error.set(null);

        try {
            const result = await invoke<T>(command, args);
            set(result);
            return result;
        } catch (err) {
            error.set(err as Error);
            throw err;
        } finally {
            loading.set(false);
        }
    }

    return {
        subscribe,
        loading: { subscribe: loading.subscribe },
        error: { subscribe: error.subscribe },
        execute
    };
}

export function createEventStore<T>(eventName: string) {
    const { subscribe, set } = writable<T | null>(null);

    let unlisten: (() => void) | undefined;

    async function start() {
        unlisten = await listen<T>(eventName, (event) => {
            set(event.payload);
        });
    }

    function stop() {
        unlisten?.();
    }

    return {
        subscribe,
        start,
        stop
    };
}
```

## State Management

### Using Zustand (React)

```typescript
// src/store/appStore.ts
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface AppState {
    files: string[];
    currentFile: string | null;
    content: string;
    isLoading: boolean;
    openFile: (path: string) => Promise<void>;
    saveFile: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
    files: [],
    currentFile: null,
    content: '',
    isLoading: false,

    openFile: async (path: string) => {
        set({ isLoading: true });

        try {
            const content = await invoke<string>('read_file', { path });
            set({
                currentFile: path,
                content,
                files: [...get().files, path]
            });
        } finally {
            set({ isLoading: false });
        }
    },

    saveFile: async () => {
        const { currentFile, content } = get();
        if (!currentFile) return;

        set({ isLoading: true });

        try {
            await invoke('write_file', {
                path: currentFile,
                content
            });
        } finally {
            set({ isLoading: false });
        }
    }
}));
```

### Using Pinia (Vue)

```typescript
// src/stores/app.ts
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useAppStore = defineStore('app', () => {
    const files = ref<string[]>([]);
    const currentFile = ref<string | null>(null);
    const content = ref('');
    const isLoading = ref(false);

    async function openFile(path: string) {
        isLoading.value = true;

        try {
            const fileContent = await invoke<string>('read_file', { path });
            currentFile.value = path;
            content.value = fileContent;
            files.value.push(path);
        } finally {
            isLoading.value = false;
        }
    }

    async function saveFile() {
        if (!currentFile.value) return;

        isLoading.value = true;

        try {
            await invoke('write_file', {
                path: currentFile.value,
                content: content.value
            });
        } finally {
            isLoading.value = false;
        }
    }

    return {
        files,
        currentFile,
        content,
        isLoading,
        openFile,
        saveFile
    };
});
```

## Type Safety

### Shared Type Definitions

```typescript
// src/types/tauri.ts
export interface FileInfo {
    path: string;
    name: string;
    size: number;
    modified: number;
}

export interface AppConfig {
    theme: 'light' | 'dark' | 'system';
    language: string;
    recentFiles: string[];
}

// Define Command args and return types
export interface Commands {
    greet: {
        args: { name: string };
        return: string;
    };
    read_file: {
        args: { path: string };
        return: string;
    };
    write_file: {
        args: { path: string; content: string };
        return: void;
    };
}
```

### Type-Safe Invoke

```typescript
// src/utils/tauri.ts
import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { Commands } from '../types/tauri';

export async function invoke<K extends keyof Commands>(
    command: K,
    args: Commands[K]['args']
): Promise<Commands[K]['return']> {
    return tauriInvoke(command, args);
}
```

## Best Practices

1. **Encapsulate API Calls**: Wrap Tauri API calls in hooks/composables/stores
2. **Error Handling**: Handle Tauri API errors consistently
3. **Loading State**: Show loading state for operations
4. **Type Safety**: Use TypeScript for type safety
5. **Resource Cleanup**: Properly clean up event listeners
6. **State Management**: Use appropriate state management solutions