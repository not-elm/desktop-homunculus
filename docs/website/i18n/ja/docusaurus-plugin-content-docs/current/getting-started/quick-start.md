---
title: "クイックスタート"
sidebar_position: 4
---

# クイックスタート

:::tip[はじめる前に]
[インストール](/getting-started/installation)ガイドが完了していることを確認してください。Desktop Homunculus と公式 MOD がインストールされ、アプリが起動している必要があります。
:::

## 最初の Persona を作成する

:::tip[Desktop Homunculus が初めての方へ]
Persona、MOD、Asset について[基本コンセプト](./concepts)ページで確認できます。
:::

Desktop Homunculus を初めて起動すると、画面にキャラクターは表示されません — まず **Persona** を作成する必要があります。

1. **システムトレイ**（通知領域）の **Desktop Homunculus** アイコンをクリック
2. **"Persona"** を選択して Persona Management ダッシュボードを開く
3. **ID**（例：`elmer`）と **Name** を入力し、**Create** をクリック
4. VRM モデルを選択 — 組み込みの `vrm:elmer` をすぐに使えます
5. **Auto-Spawn** を有効にして、次回以降の起動時にキャラクターが自動的に表示されるようにし、**Save** をクリック

デスクトップにキャラクターが表示されるはずです。待機状態でアイドルし、つかむと反応し、カーソルを追従します — 組み込みのアセットと `@hmcs/persona` MOD によって動作しています。

## キャラクターとのインタラクション

### ドラッグ & 移動

キャラクターをクリック＆ドラッグして、デスクトップ上の好きな場所に移動できます。ドラッグ中、キャラクターは「drag」状態に切り替わります。離すと新しい位置にドロップされます。

### キャラクターの状態

キャラクターはさまざまな状態に応じて、それぞれ異なる動作をします：

| 状態 | トリガー |
|---|---|
| **`idle`** | デフォルト状態 |
| **`drag`** | キャラクターをクリック＆ドラッグ |
| **`sitting`** | ウィンドウの端にキャラクターをドロップ |

### 右クリックメニュー {#right-click-menu}

キャラクターを右クリックすると、エンジンに組み込まれたコンテキストメニューオーバーレイが開きます。ここからインストール済み MOD が提供する、キャラクターごとのアクションにアクセスできます。

### 設定 {#settings}

設定パネルはエンジンに組み込まれています。**システムトレイ** → **"Settings"** で開きます。パネルが開いているときに Settings を選んでも、何も起きません。

| 設定 | 説明 |
|---|---|
| Frame Rate | 描画のフレームレート（1〜120 fps）。下げると CPU/GPU の使用量が減ります。 |
| Shadow Opacity | キャラクターの背後に出るシャドウパネルの透明度（0〜100%）。 |

変更は自動で保存され、再起動後も保たれます。

### Persona 設定

キャラクターを右クリックして **"Persona"** を選択すると、Persona ごとの設定パネルが開きます。ここでキャラクターのアイデンティティ（名前、年齢、性別、性格）や外見（ボーンスケール調整）を設定できます。

すべての Persona を管理する（新規作成、削除など）には、**システムトレイ** → **"Persona"** で Persona Management ダッシュボードにアクセスしてください。

### Speech to Text

`@hmcs/stt` MOD がインストールされている場合、**システムトレイ** → **"Speech to Text"** で STT コントロールパネルにアクセスできます。Whisper モデルをダウンロードし、言語を設定して、AI Agent との音声入力に使用します。

### アプリの終了

1. **システムトレイ**（通知領域）の **Desktop Homunculus** アイコンを見つける
2. トレイアイコンをクリックしてトレイメニューを開く
3. **Exit** を選択

:::tip
Exit はトレイメニューに標準で入っており、MOD が1つも入っていなくても常に使えます。Windows ではアプリがタスクバーや Alt+Tab に表示されないため、終了にはトレイを使ってください。
:::

## 公式 MOD の紹介

Desktop Homunculus には `@hmcs` スコープの公式 MOD セットが同梱されています：

| MOD | 説明 |
|---|---|
| `@hmcs/persona` | Persona 管理 UI とデフォルト動作サービス — アイデンティティ、性格、外見を設定 |

## 追加 MOD

オプションの MOD で Desktop Homunculus を拡張できます。CLI を使っていつでもインストールできます：

| MOD | 説明 |
|---|---|
| `@hmcs/voicevox` | [VoiceVox](https://voicevox.hiroshiba.jp/) エンジンによるテキスト読み上げ連携 |
| `@hmcs/stt` | Whisper ベースの音声認識による Speech-to-Text（[詳細](../mods/stt)） |

AI 連携については、外部で [OpenClaw](https://docs.openclaw.ai) を起動し、`@hmcs/openclaw-plugin` をインストールしてください。詳細は [AI 連携](../ai-integration) を参照してください。

:::info[VoiceVox のセットアップ]
`@hmcs/voicevox` MOD を使用するには、VoiceVox エンジンを別途インストールして起動しておく必要があります。インストール方法は [VoiceVox の Web サイト](https://voicevox.hiroshiba.jp/)をご覧ください。
:::

```shell
hmcs mod install @hmcs/voicevox @hmcs/stt
```

## 次のステップ

- **[MOD 開発](/mod-development)** — TypeScript SDK で独自の MOD を開発
- **[API リファレンス](/reference/api/homunculus-api)** — HTTP API ドキュメントの全体を確認
