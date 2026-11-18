---
title: "Quick Start"
sidebar_position: 4
---

# Quick Start

:::tip[Before You Begin]
Make sure you have completed the [Installation](/getting-started/installation) guide. Desktop Homunculus and the official MODs should be installed and the app should be running.
:::

## Create Your First Persona

:::tip[New to Desktop Homunculus?]
Learn what personas, MODs, and assets are in the [Core Concepts](./concepts) page.
:::

When you first launch Desktop Homunculus, no character appears on screen — you need to create a **persona** first.

1. Click the **Desktop Homunculus** icon in the **system tray** (notification area)
2. Select **"Persona"** to open the Persona Management dashboard
3. Enter an **ID** (e.g. `elmer`) and a **Name**, then click **Create**
4. Select a VRM model — the built-in `vrm:elmer` model is ready to use
5. Enable **Auto-Spawn** so the character appears automatically on future launches, then click **Save**

Your character should now appear on the desktop. It will idle, react when grabbed, and follow your cursor — powered by the built-in assets and the `@hmcs/persona` MOD.

## Interact with Your Character

### Drag and Move

Click and drag the character to reposition it anywhere on your desktop. While being dragged, the character switches to a "drag" state. Release to drop the character at the new position.

### Character States

The character responds to different states, each with its own behavior:

| State | Trigger |
|---|---|
| **`idle`** | Default state |
| **`drag`** | Click and drag the character |
| **`sitting`** | Drop the character on a window edge |

### Right-Click Menu {#right-click-menu}

Right-click the character to open the context menu overlay, which is built into the engine. From here you can access the per-character actions provided by installed MODs.

### Settings {#settings}

The settings panel is built into the engine. Open the **system tray** → **"Settings"** to show it; picking Settings again while the panel is open does nothing.

| Setting | Description |
|---|---|
| Frame Rate | The rendering frame rate (1–120 fps). Lower values reduce CPU/GPU usage. |
| Shadow Opacity | The transparency of the shadow panel overlay behind the character (0–100%). |

Changes are saved automatically and kept across restarts.

### Persona Settings

Right-click the character and select **"Persona"** to open the per-persona settings panel. Here you can configure the character's identity (name, age, gender, personality) and appearance (bone scale adjustments).

To manage all your personas (create new ones, delete old ones), open the **system tray** → **"Persona"** to access the Persona Management dashboard.

### Speech to Text {#speech-to-text}

The Speech to Text panel is built into the engine. Open the **system tray** → **"Speech to Text"** to show it; picking Speech to Text again while the panel is open does nothing.

Speech recognition runs locally with Whisper models. The panel lists the models and downloads them, showing the progress; a download can be cancelled with **✕**.

| Model | Size |
|---|---|
| Tiny | 32.2 MB |
| Base | 59.7 MB |
| Small | 190 MB |
| Medium | 539 MB |
| Large v3 Turbo | 574 MB |
| Large v3 | 1.08 GB |

Models are stored in `~/.homunculus/models/`. The OS may ask for microphone access the first time the microphone is used.

Recognition and Push-to-Talk are used from the SDK's `stt` namespace or the HTTP API: [recognize](/reference/api/recognize), [start Push-to-Talk](/reference/api/ptt-start), [stop Push-to-Talk](/reference/api/ptt-stop), [list models](/reference/api/list-models), and [download a model](/reference/api/download-model-stream).

### Exiting the App

1. Find the **Desktop Homunculus** icon in the **system tray** (notification area)
2. Click the tray icon to open the tray menu
3. Select **Exit**

:::tip
Exit is built into the tray menu and is always available, even when no MODs are installed. On Windows, the app is hidden from the taskbar and Alt+Tab, so use the tray to quit.
:::

## Explore Official MODs

Desktop Homunculus ships with a set of official MODs under the `@hmcs` scope:

| MOD | Description |
|---|---|
| `@hmcs/persona` | Persona management UI and default behavior service — configure identity, personality, and appearance |

## Additional MODs

You can extend Desktop Homunculus with optional MODs. Install any of them at any time using the CLI:

| MOD | Description |
|---|---|
| `@hmcs/voicevox` | Text-to-speech integration using the [VoiceVox](https://voicevox.hiroshiba.jp/) engine |

For AI-powered interaction, install [OpenClaw](https://docs.openclaw.ai) externally and set up the `@hmcs/openclaw-plugin` bridge. See [AI Integration](../ai-integration) for details.

:::info[VoiceVox Setup]
The `@hmcs/voicevox` MOD requires the VoiceVox engine to be installed and running separately. Visit the [VoiceVox website](https://voicevox.hiroshiba.jp/) for installation instructions.
:::

```shell
hmcs mod install @hmcs/voicevox
```

## What's Next?

- **[MOD Development](/mod-development)** — Build your own MODs with the TypeScript SDK
- **[API Reference](/reference/api/homunculus-api)** — Explore the full HTTP API documentation
