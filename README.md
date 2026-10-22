# Timeline Arona

ブルーアーカイブのボス戦のタイムライン（いつ、誰の EX スキルを使うか）を作り、YAML ファイルに保存するデスクトップアプリです。
保存したファイルは [Timeline Plana](https://github.com/miyabisun/timeline-plana) で読み込むと、戦闘中に次に使うスキルを画面に案内します。
ファイルの形式は [docs/README.md](docs/README.md) にあります。

## 必要なもの

- Windows 10 / 11

## 導入

1. [Releases](https://github.com/miyabisun/timeline-arona/releases) の最新版から `timeline-arona-x86_64-pc-windows-msvc.exe` を取得します。
   同じ版の `.sha256` で、ファイルが壊れていないかを確かめられます（PowerShell の `Get-FileHash <exe>` の値と比べます）。
2. 好きなフォルダーに置いて起動します。インストーラーはなく、管理者権限も要りません。

更新するときは、新しい版の exe を同じ場所に置き換えます。

## 使い方

- **基本情報**: ボス名を入れ、難易度・装甲・地形を選びます。
- **編成**: STRIKER と SPECIAL の生徒を追加し、Lv・育成・スキル（`5M7M` のように EX/NS/PS/SS、10 は M）・装備（`T9/9/9`）・ワークブック（`25/25/0`）を入れます。
- **タイムライン**: 行を追加し、種類を選びます。
  - 時間で発動: 残り時間（`3:30` か `1:20.500`）になったら使うスキル
  - コストで発動: EX コストがその値（`7.1` など）に達したら使うスキル
  - 続けて発動: 前のスキルのすぐ後に使うスキル
  - 時間マーカー: その時刻に起きること（NS の発動、ボスのフェーズ変化など）のメモ
  - 生徒は編成から選び、対象は必要なときだけ入れます。行は上下に動かしたり消したりできます。
- 書き方に誤りがあると、その欄の下に理由が出て、上に件数が出ます。誤りがあっても保存はできます（Timeline Plana で読むには直しておきます）。
- **保存**（Ctrl+S）、メニューの **開く…**（Ctrl+O）、**新規**（Ctrl+N）、**名前を付けて保存…**（Ctrl+Shift+S）。保存していない変更があるときは、新規・開く・閉じる前に確認します。
- メニューの **テーマ設定** で、自動・ライト・ダークを選べます。

## 開発

```sh
npm ci
npm run tauri dev        # 開発用に起動
npm run verify           # build・型・単体・e2e・rustfmt・clippy・テスト
git config core.hooksPath .githooks   # clone ごとに一度: main とタグの push 前に verify を実行
```

画面の規則は [DESIGN.md](DESIGN.md) にあります。リリースは `vX.Y.Z` のタグを push すると GitHub Actions が exe と `.sha256` を作ります（`.github/workflows/release.yml`。タグは `package.json` の version と一致させます）。
