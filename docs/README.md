# Timeline Arona - YAMLフォーマット仕様

## 概要

Timeline Aronaは、ブルーアーカイブのボス戦攻略用タイムラインをYAMLファイルとして作成するプロジェクトです。
作成したYAMLファイルは [Timeline Plana](https://github.com/miyabisun/timeline-plana) で読み込み、リアルタイムの戦闘支援に使用されます。

## ファイルフォーマット

### 全体構造

```yaml
# メタデータ
boss: "ホド"
difficulty: "INSANE"
armor: "重装甲"
terrain: "屋外"
party:
  striker:
    - name: "シロコ"
      lv: 90
      star: "固有3"
      skill: "5M7M"
      equip: "T9/9/9"
      wb: "25/25/0"
    - name: "ホシノ"
      lv: 90
      star: "固有2"
      skill: "5777"
      equip: "T7/7/7"
      wb: "25/25/0"
  special:
    - name: "コタマ"
      lv: 80
      star: "固有1"
      skill: "2MMM"
      equip: "T5/5/5"
      wb: "0/0/0"
    - name: "セリナ"
      lv: 70
      star: "星4"
      skill: "3777"
      equip: "T5/5/5"
      wb: "0/0/0"

# タイムライン
timeline:
  - when_at: "3:30"
    skill: "コタマ"
    note: "開幕バフ"
  - skill: "ホシノ"
    note: "バフ重ねがけ"
  - skill: "シロコ"
    note: "EXで攻撃"
  - time: "3:25"
    note: "コタマNS ボス攻撃力デバフ発動"
  - when_cost: 7.1
    skill: "コタマ"
    note: "2回目バフ"
  - skill: "ホシノ"
  - skill: "シロコ"
  - time: "1:30"
    note: "ボス防御力ダウンフェーズ"
  - when_at: "1:20.500"
    skill: "セリナ"
    target: "シロコ"
    note: "回復"
```

### メタデータ

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `boss` | string | Yes | ボス名 |
| `difficulty` | string | Yes | 難易度（NORMAL, HARD, VERYHARD, HARDCORE, EXTREME, INSANE, TORMENT） |
| `armor` | string | Yes | ボスの防御属性（軽装備, 重装甲, 特殊装甲, 弾力装甲, 複合装甲） |
| `terrain` | string | Yes | 地形（市街地, 屋外, 屋内） |
| `party` | object | Yes | 編成 |
| `party.striker` | array | Yes | STRIKERの生徒リスト |
| `party.special` | array | Yes | SPECIALの生徒リスト |
| `party.striker[].name` | string | Yes | 生徒名（SchaleDB の日本版の名前。衣装違いは `ホシノ（臨戦）` のように括弧で書く） |
| `party.striker[].lv` | number | No | レベル（最大90） |
| `party.striker[].star` | string | No | 育成段階（星3, 星4, 固有1, 固有2, 固有3, 固有4） |
| `party.striker[].skill` | string | No | スキルレベル（EX/NS/PS/SSの順、各1桁。10はMと表記。例: `"5M7M"`） |
| `party.striker[].equip` | string | No | 装備Tier（攻撃力/防御力/その他のスラッシュ区切り。例: `"T9/9/9"`） |
| `party.striker[].wb` | string | No | ワークブック（HP/攻撃力/治癒力のスラッシュ区切り。各最大25。例: `"25/25/0"`） |
| `party.special[].name` | string | Yes | 生徒名（`party.striker[].name` と同じ） |
| `party.special[].lv` | number | No | レベル（最大90） |
| `party.special[].star` | string | No | 育成段階（星3, 星4, 固有1, 固有2, 固有3, 固有4） |
| `party.special[].skill` | string | No | スキルレベル（EX/NS/PS/SSの順、各1桁。10はMと表記。例: `"2MMM"`） |
| `party.special[].equip` | string | No | 装備Tier（攻撃力/防御力/その他のスラッシュ区切り。例: `"T5/5/5"`） |
| `party.special[].wb` | string | No | ワークブック（HP/攻撃力/治癒力のスラッシュ区切り。各最大25。例: `"25/25/0"`） |

育成（`lv`・`star`・`skill`・`equip`・`wb`）は、分かる場合だけ書く。攻略動画から作ったタイムラインのように
分からないときは省く。書いた欄は上の形で検査する。Timeline Plana は生徒名だけを読む。

### タイムライン

`timeline` は、スキル発動ルールの順序付き配列です。上から順に評価されます。

`when_at` または `when_cost` が指定された行は、条件を満たすまで待機します。
どちらも指定されていない行は、前のスキルに続けて即座に連続発動します。

タイムラインの各行は以下の3種類のいずれかです。

#### 1. スキル発動（トリガー待機あり）

`when_at` または `when_cost` で条件を指定し、条件を満たすまで待機してからスキルを発動します。

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `when_at` | string | いずれか1つ | 時間ベーストリガー。カウントダウンタイマーの値（`"M:SS"` または `"M:SS.mmm"` 形式） |
| `when_cost` | number | いずれか1つ | コストベーストリガー。EXスキルコストが指定値以上になったら発動（小数対応）。「6〜8」のような幅は下限を書き、幅や「〜したら」の条件は `note` に書く |
| `skill` | string | Yes | 発動するEXスキルの生徒名 |
| `target` | string | No | スキルの対象（自明でない場合のみ指定） |
| `note` | string | No | メモ・補足説明 |

#### 2. スキル発動（即座に連続発動）

トリガー条件なし。前のスキルに続けて即座に発動します。

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `skill` | string | Yes | 発動するEXスキルの生徒名 |
| `target` | string | No | スキルの対象（自明でない場合のみ指定） |
| `note` | string | No | メモ・補足説明 |

#### 3. 時間マーカー（情報表示のみ）

スキル操作は不要だが、タイミングを把握すべきイベント（NSの自動発動、ボスのフェーズ変化など）を記録します。

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `time` | string | Yes | イベントの時刻（`"M:SS"` または `"M:SS.mmm"` 形式） |
| `note` | string | Yes | イベントの説明 |
