---
name: commands
description: Frontend-backend communication (Commands and Events)
metadata:
  tags: commands, events, ipc, communication, invoke, emit
---

# Commands and Events

Commands are the primary way frontend calls Rust functions. Events are used for backend to push messages to frontend.

## Basic Command

### Rust Side Definition

In `src-tauri/src/lib.rs` or `src-tauri/src/main.rs`:

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend Call

```typescript
import { invoke } from '@tauri-apps/api/core';

// Call command
const response = await invoke('greet', { name: 'World' });
console.log(response); // "Hello, World!"
```

## Async Command

### Rust Side

```rust
use tauri::State;
use std::time::Duration;
use tokio::time::sleep;

#[tauri::command]
async fn fetch_data(url: String) -> Result<String, String> {
    sleep(Duration::from_secs(1)).await;

    // Simulate network request
    Ok(format!("Data from {}", url))
}
```

### Frontend Call

```typescript
const data = await invoke('fetch_data', { url: 'https://api.example.com' });
```

## Command with State

### Define State on Rust Side

```rust
use std::sync::Mutex;

struct AppState {
    counter: Mutex<i32>,
}

#[tauri::command]
fn increment_counter(state: State<AppState>) -> i32 {
    let mut counter = state.counter.lock().unwrap();
    *counter += 1;
    *counter
}
```

### Initialize State

```rust
fn run() {
    tauri::Builder::default()
        .manage(AppState {
            counter: Mutex::new(0),
        })
        .invoke_handler(tauri::generate_handler![increment_counter])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Error Handling

### Return Result on Rust Side

```rust
use serde::Serialize;

#[derive(Serialize)]
struct ApiError {
    message: String,
    code: u16,
}

#[tauri::command]
fn risky_operation() -> Result<String, ApiError> {
    if some_condition() {
        Ok("Success".to_string())
    } else {
        Err(ApiError {
            message: "Something went wrong".to_string(),
            code: 500,
        })
    }
}
```

### Frontend Error Handling

```typescript
try {
    const result = await invoke('risky_operation');
} catch (error) {
    console.error('Command failed:', error);
}
```

## Events

### Send Event from Backend to Frontend

```rust
use tauri::Emitter;

#[tauri::command]
fn start_process(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        for i in 0..100 {
            app.emit("progress", i).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}
```

### Listen to Events on Frontend

```typescript
import { listen } from '@tauri-apps/api/event';

// Listen to event
const unlisten = await listen('progress', (event) => {
    console.log('Progress:', event.payload);
});

// Cancel listener
unlisten();
```

### One-Time Event Listener

```typescript
import { once } from '@tauri-apps/api/event';

// Listen only once
await once('completed', (event) => {
    console.log('Completed:', event.payload);
});
```

## Send Event from Frontend to Backend

```typescript
import { emit } from '@tauri-apps/api/event';

// Send event to backend
await emit('frontend-event', { data: 'some data' });
```

### Listen on Rust Side

```rust
use tauri::Listener;

fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.listen("frontend-event", |event| {
                println!("Received: {:?}", event.payload());
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Window-Specific Events

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

const window = getCurrentWindow();

// Send event to specific window
await window.emit('window-specific-event', { data: 'hello' });

// Listen to events on specific window
await window.listen('event-name', (event) => {
    console.log(event.payload);
});
```