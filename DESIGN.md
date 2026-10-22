---
version: alpha
name: Sumi / Timeline Arona
description: タイムラインを編集するデスクトップアプリの意匠。rust-svelte-template の Sumi family 原本（2026-10-22 参照）を取り込んで仕上げた。
colors:
  primary: "#1d5fa6"
  accent: "#1d5fa6"
  accent-subtle: "rgba(29, 95, 166, 0.10)"
  surface: "#faf6ef"
  surface-raised: "#fffdf8"
  on-surface: "#3a2f28"
  muted: "#6f6257"
  border: "#e3d9c9"
  scrim: "rgba(58, 47, 40, 0.4)"
  link: "#14506e"
  danger: "#9c2b1d"
  danger-subtle: "#f9e9e4"
  wash-base: "#f6efe0"
  wash-raised: "#faf4ea"
  hover-1: "rgba(29, 95, 166, 0.10)"
  hover-2: "rgba(29, 95, 166, 0.16)"
typography:
  title:
    fontFamily: system-ui
    fontSize: 17px
    fontWeight: 600
    lineHeight: 1.3
  body:
    fontFamily: system-ui
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.6
  body-sm:
    fontFamily: system-ui
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: system-ui
    fontSize: 15px
    fontWeight: 500
    lineHeight: 1.2
  caption:
    fontFamily: system-ui
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.4
rounded:
  sm: 6px
  md: 8px
  lg: 12px
  full: 9999px
spacing:
  sp-1: 4px
  sp-2: 8px
  sp-3: 12px
  sp-4: 16px
  sp-5: 24px
components:
  app-header:
    backgroundColor: "{colors.wash-base}"
    textColor: "{colors.on-surface}"
    height: 48px
  button:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.on-surface}"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    padding: 8px
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.surface-raised}"
    typography: "{typography.label}"
    rounded: "{rounded.sm}"
    padding: 8px
  icon-button:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.sm}"
    size: 36px
  input:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body}"
    rounded: "{rounded.sm}"
    padding: 8px
  modal:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.lg}"
    padding: 16px
  modal-scrim:
    backgroundColor: "{colors.scrim}"
  error-banner:
    backgroundColor: "{colors.danger-subtle}"
    textColor: "{colors.danger}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.sm}"
    padding: 8px
  badge:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.danger}"
    typography: "{typography.caption}"
    rounded: "{rounded.full}"
    padding: 4px
---

# Timeline Arona

## Overview

ボス戦のタイムラインを作り、YAML に保存するデスクトップアプリ（Tauri）。主役は編集中のタイムラインで、
操作はファイルの保存・開く・新規と、行と生徒の追加・並べ替え・削除に限る。
原本の規則のうち、ここに書いたものがこの製品の正である。原本の更新は差分を見て明示的に取り込む。

## Colors

暗色は Sumi、明色は Kinari。値は `src/global.sass` の CSS 変数が持ち、frontmatter は Kinari の値を持つ。
アクセントはアロナの青とし、Kinari は `#1d5fa6`、Sumi は `#6cb6ff` を使う。
主ボタンの文字（surface-raised）との対比は Kinari 6.4:1、Sumi 7.3:1 で、面の上の対比も 4.5:1 を超える。
アクセントは主操作・フォーカス・選択中のテーマにだけ使う。塗りつぶしの主ボタンは画面に1つとする。
誤りは danger 色の文字と枠で示し、件数は文字でも示す（色だけに頼らない）。
テーマの保存キーは `timeline-arona:theme`。明示指定・OS 委任・初回描画前の適用は原本の規則に従う。

## Typography

書体は system-ui。サイズ・太さ・行高は frontmatter の 5 役割だけを使う。
欄の見出しと誤りの文は注記（12px）、表は補助文（14px）、ファイル名はタイトル（17px）とする。

## Layout

- app header は全幅・48px・sticky。左にアプリ名（リンクなし）、右に 36px のメニューボタン。
- 本文は文書フローで縦に続き、内側の縦スクロール領域を作らない。表は `.table-scroll` の中だけで横にはみ出せる。
  編集の表は列が多いので、本文は幅いっぱい（`.content.content-wide`）を使う。
- 本文の先頭の1行に、開いているファイルとその状態（未保存・問題の件数）と主操作をまとめる。
  節は見出しと表で区切り、説明の帯を足さない。
- 誤りは、その欄の直下に置き、欄を danger の枠にする（`aria-invalid`）。通常時に誤りの場所を予約しない。

## Components

- メニュー: 原本のドロップダウン。先頭が「テーマ設定」、続いてファイルの操作とそのショートカット。
- テーマ設定: 原本のモーダル（自動・ライト・ダーク）。
- アイコン: `src/lib/Icon.svelte` の辞書。原本の辞書に plus・arrow-up・arrow-down を足した（汎用なので原本へ戻す候補）。
  絵文字や文字記号をアイコンや状態の印に使わない。状態は文字で示す。
- 入力欄: surface・1px 枠・6px 角丸。表の中では上下 4px・左右 8px の余白で詰める。
- 行の操作（上へ・下へ・消す）はアイコンボタンを行末に並べ、端の行では動けない方向を無効にする。
- 失敗（開けない・保存できない）は本文先頭の error-banner で、何が起きたかを文で示す。編集中の内容は消さない。
