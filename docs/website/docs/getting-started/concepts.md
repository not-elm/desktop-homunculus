---
title: "Core Concepts"
sidebar_position: 3
---

# Core Concepts

Before diving into the Quick Start, here are three key concepts you'll encounter throughout Desktop Homunculus.

## Persona

A **persona** is a character that lives on your desktop. Each persona has:

- **Identity** — A name, age, gender, personality description, and profile text that define who the character is
- **VRM Model** — A 3D avatar rendered in a transparent window that floats above your other applications. You can use the built-in model or bring your own [VRM](https://vrm.dev/en/) file
- **Persistent Settings** — Appearance tweaks (bone scale adjustments) and preferences that are saved between sessions

You can create multiple personas, each with its own look and personality. Enable **Auto-Spawn** on a persona to have it appear automatically every time you launch the app.

Personas are managed through the **Persona Management** dashboard, accessible from the system tray.

## MOD

A **MOD** is a package that adds features to Desktop Homunculus. The right-click menu, the settings panel, and the default assets (character model, animations, and sound effects) are built into the engine. Persona management is provided by an official MOD that ships with the installer.

Official MODs use the `@hmcs/` prefix:

| MOD | What it does |
|---|---|
| `@hmcs/persona` | Persona management and default character behavior |

Additional MODs — such as text-to-speech (`@hmcs/voicevox`) — can be installed at any time using the `hmcs` CLI. Speech recognition is built into the engine ([Speech to Text](./quick-start#speech-to-text)). For AI-powered interaction, see [AI Integration](../ai-integration) (requires running [OpenClaw](https://docs.openclaw.ai) externally).

## Asset

An **asset** is a file built into the engine or bundled with a MOD — VRM models, animations (VRMA), sound effects, images, or HTML UIs. Assets are referenced by ID in the format **`mod-name:asset-id`**.

For example:
- `vrm:elmer` — the built-in VRM character model
- `vrma:idle-maid` — the default idle animation
- `se:open` — a UI open sound effect

These default assets are built into the engine and power the persona system out of the box.

## Next Steps

Ready to get started? Head over to the [Quick Start](./quick-start) guide to create your first persona.
