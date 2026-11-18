---
title: "Official Mods"
sidebar_position: 1
---

# Official Mods

MODs extend Desktop Homunculus with characters, animations, sound effects, UI panels, and integrations. The following official MODs are maintained as part of the project.

## MOD List

| MOD | Description | Install |
|---|---|---|
| [@hmcs/persona](./persona) | Persona management UI and default behavior service (identity, personality, appearance) | Recommended |
| [@hmcs/voicevox](./voicevox) | Text-to-speech integration using the VoiceVox engine | Optional |
| [@hmcs/stt](./stt) | Speech-to-text control panel with Whisper model management | Optional |

The right-click menu is built into the engine. MODs add their own items to it through the `homunculus.menus` field.

The [settings panel](../getting-started/quick-start#settings) is built into the engine too, and opens from **Settings** in the system tray.

:::info Migrating from `@hmcs/menu`
The right-click menu used to come from the `@hmcs/menu` MOD. The engine now ignores that MOD, so it is no longer needed and can be removed with `hmcs mod uninstall @hmcs/menu`.
:::

:::info Migrating from `@hmcs/settings`
The settings panel used to come from the `@hmcs/settings` MOD. The engine now ignores that MOD, so it is no longer needed and can be removed with `hmcs mod uninstall @hmcs/settings`.
:::

## Managing MODs

### List installed MODs

```shell
hmcs mod list
```

Example output:

```text
 NAME            VERSION  DESCRIPTION
 @hmcs/persona   1.0.0    Persona management
 @hmcs/stt       1.0.0    Speech-to-text panel
```

### Install a MOD

```shell
hmcs mod install <package>...
```

For example, to install the VoiceVox MOD:

```shell
hmcs mod install @hmcs/voicevox
```

### Uninstall a MOD

```shell
hmcs mod uninstall <package>...
```

For detailed CLI reference, see [`hmcs mod`](/reference/cli/mod).
