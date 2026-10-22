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
- 生徒名の欄は SchaleDB の生徒名を候補に出し、名前の左に顔アイコンを出します。

## 攻略動画からタイムラインを作る（MCP サーバー）

`timeline-arona-x86_64-pc-windows-msvc.exe mcp` で起動すると、標準入出力の MCP サーバーとして動きます。
Claude Code などのエージェントがこの道具を使い、YouTube の攻略動画（画面にタイムラインの文字を重ねたもの）から
タイムラインの YAML を作ります。文字を読んで YAML に直すのはエージェントで、Timeline Arona は API キーを持ちません。

Claude Code への登録の例:

```sh
claude mcp add --scope user timeline-arona -- "C:\Tools\timeline-arona-x86_64-pc-windows-msvc.exe" mcp
```

| 道具 | すること |
|---|---|
| `video_open` | YouTube の動画を動画用の窓で開き、広告を過ぎて読めるようになるまで待つ。題名・投稿者・長さ・説明文を返す |
| `video_frames` | 指定の秒数ごとに動画の絵（再生ボタンやマウスは写らない）を PNG に保存する。範囲の切り抜きと、前と同じ絵を捨てる指定ができる |
| `students` | SchaleDB の生徒（名前・STRIKER/SPECIAL・顔アイコンのファイル）を名前で探す |
| `timeline_check` | YAML の誤りと、編集画面と同じ書式に整えた YAML を返す |
| `timeline_open` | YAML のファイルを編集画面で開く |

窓は道具が必要とするときだけ開きます。動画用の窓を隠したり最小化したりせず、ほかの窓の後ろに置いてください。

## 開発

```sh
npm ci
npm run tauri dev        # 開発用に起動
npm run verify           # build・型・単体・e2e・rustfmt・clippy・テスト
git config core.hooksPath .githooks   # clone ごとに一度: main とタグの push 前に verify を実行
powershell -File tools/schaledb/update.ps1   # SchaleDB の生徒と顔アイコンを取り直す（public/schaledb）
```

画面の規則は [DESIGN.md](DESIGN.md) にあります。リリースは `vX.Y.Z` のタグを push すると GitHub Actions が exe と `.sha256` を作ります（`.github/workflows/release.yml`。タグは `package.json` の version と一致させます）。
