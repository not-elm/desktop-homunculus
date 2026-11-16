---
title: "Asset IDs"
sidebar_position: 3
---

# Asset IDs

Asset IDs are unique identifiers that reference files across the entire MOD system. Whenever you attach a VRM model to a persona, play an animation, or open a WebView, you use an asset ID to tell the engine which file to load. The only hard requirement is that every asset ID is **globally unique** — duplicate IDs are logged as warnings and skipped.

## Recommended Convention

The recommended convention is:

```
<mod-name>:<asset-name>
```

The engine treats asset IDs as opaque strings — it does not validate or parse this format. However, prefixing with the mod name prevents collisions between MODs and makes IDs self-documenting.

| Part | Source | Example |
|---|---|---|
| `mod-name` | Derived from the `name` field in `package.json` | `@hmcs/settings` becomes `settings` |
| `asset-name` | The key in the `homunculus.assets` object | `vrm`, `open`, `ui` |

The **mod name** is extracted from the package name by stripping the scope prefix. For example:

- `@hmcs/settings` -- mod name is `settings`
- `@hmcs/persona` -- mod name is `persona`
- `my-character` -- mod name is `my-character` (no scope to strip)

The **asset name** is whatever you chose as the key when declaring the asset in `package.json`.

### Example

Given this `package.json`:

```json
{
  "name": "my-character",
  "homunculus": {
    "assets": {
      "my-character:vrm": {
        "path": "assets/MyModel.vrm",
        "type": "vrm",
        "description": "Custom VRM character model"
      },
      "my-character:open": {
        "path": "assets/open.mp3",
        "type": "sound",
        "description": "Sound effect for opening action"
      }
    }
  }
}
```

The asset IDs are `my-character:vrm` and `my-character:open`. You use these strings anywhere the SDK or API expects an asset reference.

## Built-in Assets

The engine ships a default character model, animations, and sound effects that all MODs can use. They are always available; there is nothing to install:

| Asset ID | Type | Description |
|---|---|---|
| `vrm:elmer` | `vrm` | VRM model named Elmer |
| `vrma:idle-maid` | `vrma` | Maid-style standing idle with hands clasped in front |
| `vrma:grabbed` | `vrma` | Reactive pose while being dragged by the user |
| `vrma:idle-sitting` | `vrma` | Seated idle loop with legs together |
| `se:open` | `sound` | Sound effect for opening a HUD panel |
| `se:close` | `sound` | Sound effect for closing a HUD panel |

Built-in assets are listed by `GET /assets` (and `assets.list()` in the SDK) with `mod` set to `"builtin"`, so `assets.list({ mod: "builtin" })` returns exactly these. Imported files are listed with `mod` set to `"local"`.

Built-in asset IDs are reserved:

- A MOD that declares one of these IDs in its `package.json` is ignored for that ID, with a warning in the log.
- Importing a file under one of these IDs (`POST /assets/import`) fails with `409 Conflict`.
- MOD packages named `builtin` or `local` are not loaded.

:::info Migrating from `@hmcs/assets`
These assets used to come from the `@hmcs/assets` MOD. They keep the same IDs, so existing code keeps working. Filtering by the old MOD name (`assets.list({ mod: "@hmcs/assets" })`) no longer returns them — filter by `"builtin"` instead. The old MOD is no longer needed and can be removed with `hmcs mod uninstall @hmcs/assets`.
:::

:::tip
You do not need to create your own idle animations unless you want custom ones.
:::

## Using Asset IDs in Code

The `@hmcs/sdk` accepts asset IDs as strings wherever an asset is needed.

### Attaching a VRM character to a persona

```typescript
import { persona } from "@hmcs/sdk";

const p = await persona.create({ id: "elmer" });
const vrm = await p.attachVrm("elmer:vrm");
```

### Playing a VRMA animation

```typescript
import { repeat } from "@hmcs/sdk";

await vrm.playVrma({
  asset: "vrma:idle-maid",
  repeat: repeat.forever(),
  transitionSecs: 0.5,
});
```

### Opening a WebView

```typescript
import { Webview, webviewSource } from "@hmcs/sdk";

await Webview.open({
  source: webviewSource.local("settings:ui"),
  size: [1, 0.9],
  viewportSize: [900, 700],
});
```

The `webviewSource.local("settings:ui")` source tells the engine to load the HTML file registered under the `settings:ui` asset ID.

## Using Asset IDs in the HTTP API

Asset IDs appear in HTTP API request bodies as well. For example, to attach a VRM model to a persona via the API:

```bash
curl -X POST http://localhost:3100/personas/elmer/vrm \
  -H "Content-Type: application/json" \
  -d '{"assetId": "elmer:vrm"}'
```

The same asset ID string is used consistently across the SDK and the HTTP API.

## Referencing Assets from Other MODs

A MOD can reference built-in assets and assets from any other installed MOD. For example, a MOD that creates a character commonly uses the built-in animations:

```typescript
// This MOD uses a built-in animation
await vrm.playVrma({
  asset: "vrma:idle-maid", // Built into the engine, not declared by this MOD
});
```

:::warning
If the MOD that owns the asset is not installed, the engine will return an error when you try to use its asset ID. Make sure the MOD that provides the asset is installed (`hmcs mod install <mod-name>`).
:::
