---
name: notifications
description: System notifications
metadata:
  tags: notification, notify, toast, system
---

# System Notifications

Tauri v2 provides system notification functionality through the `@tauri-apps/plugin-notification` plugin.

## Install Plugin

```bash
# Add plugin (Rust)
cargo add tauri-plugin-notification

# Frontend install
pnpm add @tauri-apps/plugin-notification
```

### Register in Rust

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Request Permissions

Before displaying notifications, user permission must be requested:

```typescript
import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';

// Check permission
let permissionGranted = await isPermissionGranted();

// Request permission if not granted
if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === 'granted';
}
```

## Send Notifications

### Basic Notification

```typescript
import { sendNotification } from '@tauri-apps/plugin-notification';

// Simple notification
sendNotification('Hello from Tauri!');

// Notification with title
sendNotification({
    title: 'My App',
    body: 'This is a notification message'
});
```

### Advanced Notification

```typescript
import { sendNotification } from '@tauri-apps/plugin-notification';

sendNotification({
    title: 'Download Complete',
    body: 'Your file has been downloaded successfully',
    icon: 'icons/icon.png',  // Icon path
    sound: 'default'  // Play default sound
});
```

## Notification Options

```typescript
interface NotificationOptions {
    title?: string;           // Notification title
    body?: string;            // Notification content
    icon?: string;            // Icon path
    sound?: string;           // Sound
    subtitle?: string;        // Subtitle (macOS)
    group?: string;           // Group identifier (macOS)
    requireInteraction?: boolean;  // Requires user interaction
    silent?: boolean;         // Silent notification
}
```

## Notification Actions

### Click Notification

```typescript
import { onAction, sendNotification } from '@tauri-apps/plugin-notification';

// Send notification with identifier
sendNotification({
    title: 'New Message',
    body: 'You have a new message from John'
});

// Listen for notification click
await onAction((event) => {
    console.log('Notification clicked:', event);
    // Can open specific window or perform action here
});
```

### Notification with Buttons

```typescript
import { sendNotification, registerActionTypes } from '@tauri-apps/plugin-notification';

// Register action types
await registerActionTypes([{
    id: 'message',
    actions: [
        {
            id: 'reply',
            title: 'Reply'
        },
        {
            id: 'dismiss',
            title: 'Dismiss'
        }
    ]
}]);

// Send notification with actions
sendNotification({
    title: 'New Message',
    body: 'Hello! How are you?',
    actionTypeId: 'message'
});
```

## Progress Notifications

```typescript
import { sendNotification, onAction } from '@tauri-apps/plugin-notification';

class DownloadNotifier {
    private notificationId?: string;

    async startDownload(filename: string) {
        this.notificationId = `download-${Date.now()}`;

        sendNotification({
            title: 'Download Started',
            body: `Downloading ${filename}...`,
            requireInteraction: true
        });
    }

    async updateProgress(filename: string, progress: number) {
        // Some platforms support progress notifications
        sendNotification({
            title: 'Downloading...',
            body: `${filename}: ${progress}%`,
            requireInteraction: true
        });
    }

    async completeDownload(filename: string) {
        sendNotification({
            title: 'Download Complete',
            body: `${filename} has been downloaded`,
            sound: 'default'
        });
    }

    async errorDownload(filename: string, error: string) {
        sendNotification({
            title: 'Download Failed',
            body: `Failed to download ${filename}: ${error}`
        });
    }
}
```

## Batch Notification Management

```typescript
import { sendNotification, getActive, close, onAction } from '@tauri-apps/plugin-notification';

class NotificationManager {
    private notifications: Map<string, any> = new Map();

    async show(id: string, options: NotificationOptions) {
        // Close old notification with same ID
        await this.close(id);

        sendNotification({
            ...options,
            requireInteraction: true
        });

        this.notifications.set(id, { ...options, timestamp: Date.now() });
    }

    async close(id: string) {
        const notification = this.notifications.get(id);
        if (notification) {
            await close(id);
            this.notifications.delete(id);
        }
    }

    async closeAll() {
        const active = await getActive();
        for (const notification of active) {
            await close(notification.id);
        }
        this.notifications.clear();
    }

    async getPending() {
        return Array.from(this.notifications.entries());
    }
}
```

## Permission Configuration

Configure notification permissions in `capabilities/default.json`:

```json
{
  "permissions": [
    {
      "identifier": "notification:default"
    },
    {
      "identifier": "notification:allow-notify"
    },
    {
      "identifier": "notification:allow-request-permission"
    }
  ]
}
```

## Complete Example

```typescript
import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
    onAction
} from '@tauri-apps/plugin-notification';

class NotificationService {
    private initialized = false;

    async initialize() {
        if (this.initialized) return;

        let permissionGranted = await isPermissionGranted();

        if (!permissionGranted) {
            const permission = await requestPermission();
            permissionGranted = permission === 'granted';
        }

        if (permissionGranted) {
            // Listen for notification clicks
            await onAction((event) => {
                this.handleNotificationAction(event);
            });
        }

        this.initialized = true;
    }

    async notify(options: {
        title: string;
        body: string;
        icon?: string;
        action?: () => void;
    }) {
        await this.initialize();

        sendNotification({
            title: options.title,
            body: options.body,
            icon: options.icon,
            sound: 'default'
        });

        // Store action for click invocation
        if (options.action) {
            // Implement storage logic
        }
    }

    private handleNotificationAction(event: any) {
        console.log('Notification action:', event);
        // Handle notification click
    }

    // Preset notification types
    async success(message: string) {
        await this.notify({
            title: 'Success',
            body: message
        });
    }

    async error(message: string) {
        await this.notify({
            title: 'Error',
            body: message
        });
    }

    async info(message: string) {
        await this.notify({
            title: 'Info',
            body: message
        });
    }
}

// Usage example
const notifications = new NotificationService();
await notifications.success('File saved successfully!');
await notifications.error('Failed to connect to server');
```

## Platform Differences

### macOS
- Supports notification grouping
- Supports subtitles
- Supports custom action buttons

### Windows
- Supports progress bar notifications
- Supports input reply
- Supports quick actions

### Linux
- Features depend on desktop environment
- GNOME and KDE support well

## Best Practices

1. **Request Permissions**: Request notification permissions on first use
2. **Moderate Use**: Avoid sending too many notifications to disturb users
3. **Meaningful Content**: Ensure notification content is valuable to user
4. **Error Handling**: Handle cases where permission is denied
5. **Action Feedback**: Add executable actions to notifications