//! The timeline file (docs/README.md): reading it, checking it and writing it back.
//!
//! The editor keeps whatever the Sensei typed, mistakes included, so the model has
//! optional numbers and plain strings; `check` says what is wrong where.

use serde::{Deserialize, Serialize};
use serde_norway::{Mapping, Value};

pub const DIFFICULTIES: [&str; 7] = [
    "NORMAL", "HARD", "VERYHARD", "HARDCORE", "EXTREME", "INSANE", "TORMENT",
];
pub const ARMORS: [&str; 5] = ["軽装備", "重装甲", "特殊装甲", "弾力装甲", "複合装甲"];
pub const TERRAINS: [&str; 3] = ["市街地", "屋外", "屋内"];
pub const STARS: [&str; 6] = ["星3", "星4", "固有1", "固有2", "固有3", "固有4"];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Student {
    pub name: String,
    pub lv: Option<u32>,
    pub star: String,
    pub skill: String,
    pub equip: String,
    pub wb: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Party {
    pub striker: Vec<Student>,
    pub special: Vec<Student>,
}

/// One row of the timeline, by what starts it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Row {
    /// At a remaining time.
    At {
        when_at: String,
        skill: String,
        target: String,
        note: String,
    },
    /// When the EX cost reaches a value.
    Cost {
        when_cost: Option<f64>,
        skill: String,
        target: String,
        note: String,
    },
    /// Right after the previous skill.
    Chain {
        skill: String,
        target: String,
        note: String,
    },
    /// Something to know at a remaining time; nothing to press.
    Mark { time: String, note: String },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub boss: String,
    pub difficulty: String,
    pub armor: String,
    pub terrain: String,
    pub party: Party,
    pub timeline: Vec<Row>,
}

/// A mistake and the field it is in (`boss`, `party.striker.0.lv`, `timeline.3.when_at`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Problem {
    pub field: String,
    pub message: String,
}

fn problem(field: impl Into<String>, message: impl Into<String>) -> Problem {
    Problem {
        field: field.into(),
        message: message.into(),
    }
}

/// Reads a timeline file. A file that is no YAML mapping is an error; anything else
/// is read as far as it goes, the gaps left for `check` to report.
pub fn read(yaml: &str) -> Result<Timeline, String> {
    let value: Value = serde_norway::from_str(yaml).map_err(|e| format!("YAML: {e}"))?;
    let root = value
        .as_mapping()
        .ok_or("YAML の一番外側がキーと値の組ではありません")?;
    let party = root.get("party").and_then(Value::as_mapping);
    let students = |group: &str| {
        party
            .and_then(|p| p.get(group))
            .and_then(Value::as_sequence)
            .map(|list| list.iter().map(read_student).collect())
            .unwrap_or_default()
    };
    Ok(Timeline {
        boss: text(root, "boss"),
        difficulty: text(root, "difficulty"),
        armor: text(root, "armor"),
        terrain: text(root, "terrain"),
        party: Party {
            striker: students("striker"),
            special: students("special"),
        },
        timeline: root
            .get("timeline")
            .and_then(Value::as_sequence)
            .map(|rows| rows.iter().map(read_row).collect())
            .unwrap_or_default(),
    })
}

/// A scalar as the text the Sensei would have typed (numbers as written).
fn text(map: &Mapping, key: &str) -> String {
    match map.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

fn read_student(value: &Value) -> Student {
    let empty = Mapping::new();
    let map = value.as_mapping().unwrap_or(&empty);
    Student {
        name: text(map, "name"),
        lv: map
            .get("lv")
            .and_then(Value::as_u64)
            .and_then(|lv| u32::try_from(lv).ok()),
        star: text(map, "star"),
        skill: text(map, "skill"),
        equip: text(map, "equip"),
        wb: text(map, "wb"),
    }
}

fn read_row(value: &Value) -> Row {
    let empty = Mapping::new();
    let map = value.as_mapping().unwrap_or(&empty);
    let (skill, target, note) = (text(map, "skill"), text(map, "target"), text(map, "note"));
    if map.contains_key("time") {
        Row::Mark {
            time: text(map, "time"),
            note,
        }
    } else if map.contains_key("when_at") {
        Row::At {
            when_at: text(map, "when_at"),
            skill,
            target,
            note,
        }
    } else if map.contains_key("when_cost") {
        Row::Cost {
            when_cost: map.get("when_cost").and_then(Value::as_f64),
            skill,
            target,
            note,
        }
    } else {
        Row::Chain {
            skill,
            target,
            note,
        }
    }
}

/// Whether `text` is a remaining time as the docs write it: `M:SS` or `M:SS.mmm`.
pub fn is_time(text: &str) -> bool {
    let Some((minutes, rest)) = text.split_once(':') else {
        return false;
    };
    let (seconds, millis) = rest.split_once('.').unwrap_or((rest, "000"));
    let digits = |s: &str, len: Option<usize>| {
        !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) && len.is_none_or(|n| s.len() == n)
    };
    digits(minutes, None) && digits(seconds, Some(2)) && digits(millis, Some(3)) && seconds < "60"
}

/// Skill levels written EX/NS/PS/SS, one character each, M for 10 (`5M7M`); EX goes
/// up to 5.
fn is_skill_levels(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    chars.len() == 4
        && ('1'..='5').contains(&chars[0])
        && chars[1..]
            .iter()
            .all(|c| ('1'..='9').contains(c) || *c == 'M')
}

/// Three numbers split by `/`, each at most `max`.
fn three_numbers(text: &str, max: u32) -> bool {
    let parts: Vec<&str> = text.split('/').collect();
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.bytes().all(|b| b.is_ascii_digit())
                && p.parse::<u32>().is_ok_and(|n| n <= max)
        })
}

/// Every mistake in `timeline`, in the order of the file.
pub fn check(timeline: &Timeline) -> Vec<Problem> {
    let mut problems = Vec::new();
    if timeline.boss.trim().is_empty() {
        problems.push(problem("boss", "ボス名を入れてください"));
    }
    for (field, value, choices) in [
        ("difficulty", &timeline.difficulty, &DIFFICULTIES[..]),
        ("armor", &timeline.armor, &ARMORS[..]),
        ("terrain", &timeline.terrain, &TERRAINS[..]),
    ] {
        if !choices.contains(&value.as_str()) {
            problems.push(problem(
                field,
                format!("{} から選んでください", choices.join("・")),
            ));
        }
    }
    let mut names: Vec<&str> = Vec::new();
    for (group, list) in [
        ("striker", &timeline.party.striker),
        ("special", &timeline.party.special),
    ] {
        for (i, student) in list.iter().enumerate() {
            let at = |key: &str| format!("party.{group}.{i}.{key}");
            if student.name.trim().is_empty() {
                problems.push(problem(at("name"), "生徒名を入れてください"));
            } else if names.contains(&student.name.as_str()) {
                problems.push(problem(at("name"), "同じ生徒が編成に二人います"));
            } else {
                names.push(&student.name);
            }
            check_student(&mut problems, student, at);
        }
    }
    if timeline.party.striker.is_empty() && timeline.party.special.is_empty() {
        problems.push(problem("party", "編成に生徒を入れてください"));
    }
    for (i, row) in timeline.timeline.iter().enumerate() {
        check_row(&mut problems, row, &names, |key| {
            format!("timeline.{i}.{key}")
        });
    }
    problems
}

/// A student's level and the forms of the training fields; `at` names a field.
fn check_student(problems: &mut Vec<Problem>, student: &Student, at: impl Fn(&str) -> String) {
    if !student.lv.is_some_and(|lv| (1..=90).contains(&lv)) {
        problems.push(problem(at("lv"), "レベルは 1〜90 です"));
    }
    if !STARS.contains(&student.star.as_str()) {
        problems.push(problem(
            at("star"),
            format!("{} から選んでください", STARS.join("・")),
        ));
    }
    if !is_skill_levels(&student.skill) {
        problems.push(problem(
            at("skill"),
            "EX・NS・PS・SS の順に 4 文字（10 は M、EX は 5 まで）。例: 5M7M",
        ));
    }
    if !student
        .equip
        .strip_prefix('T')
        .is_some_and(|rest| three_numbers(rest, 10))
    {
        problems.push(problem(
            at("equip"),
            "T と 3 つの段階を / で区切ります。例: T9/9/9",
        ));
    }
    if !three_numbers(&student.wb, 25) {
        problems.push(problem(
            at("wb"),
            "HP・攻撃力・治癒力を / で区切ります（各 25 まで）。例: 25/25/0",
        ));
    }
}

/// A row's trigger and its student, who must be in the party (`names`).
fn check_row(problems: &mut Vec<Problem>, row: &Row, names: &[&str], at: impl Fn(&str) -> String) {
    let skill = match row {
        Row::At { skill, .. } | Row::Cost { skill, .. } | Row::Chain { skill, .. } => Some(skill),
        Row::Mark { .. } => None,
    };
    if let Some(skill) = skill {
        if skill.trim().is_empty() {
            problems.push(problem(at("skill"), "生徒を選んでください"));
        } else if !names.contains(&skill.as_str()) {
            problems.push(problem(at("skill"), "編成にいない生徒です"));
        }
    }
    match row {
        Row::At { when_at, .. } if !is_time(when_at) => problems.push(problem(
            at("when_at"),
            "残り時間は M:SS か M:SS.mmm です。例: 3:30",
        )),
        Row::Cost { when_cost, .. } if !when_cost.is_some_and(|c| (0.0..=10.0).contains(&c)) => {
            problems.push(problem(at("when_cost"), "コストは 0〜10 です。例: 7.1"));
        }
        Row::Mark { time, note } => {
            if !is_time(time) {
                problems.push(problem(
                    at("time"),
                    "残り時間は M:SS か M:SS.mmm です。例: 3:25",
                ));
            }
            if note.trim().is_empty() {
                problems.push(problem(at("note"), "何が起きるかを書いてください"));
            }
        }
        _ => {}
    }
}

/// A YAML double-quoted string (a JSON string is one).
fn quoted(s: &str) -> String {
    serde_json::to_string(s).expect("a string always serializes to JSON")
}

/// A skill row's lines; an empty target or note is left out.
fn skill_lines(skill: &str, target: &str, note: &str) -> Vec<String> {
    let mut lines = vec![format!("skill: {}", quoted(skill))];
    if !target.is_empty() {
        lines.push(format!("target: {}", quoted(target)));
    }
    if !note.is_empty() {
        lines.push(format!("note: {}", quoted(note)));
    }
    lines
}

/// The file in the docs' layout: comments, key order, every string quoted.
pub fn write(timeline: &Timeline) -> String {
    let mut lines = vec!["# メタデータ".to_owned()];
    for (key, value) in [
        ("boss", &timeline.boss),
        ("difficulty", &timeline.difficulty),
        ("armor", &timeline.armor),
        ("terrain", &timeline.terrain),
    ] {
        lines.push(format!("{key}: {}", quoted(value)));
    }
    lines.push("party:".to_owned());
    for (group, list) in [
        ("striker", &timeline.party.striker),
        ("special", &timeline.party.special),
    ] {
        if list.is_empty() {
            lines.push(format!("  {group}: []"));
        } else {
            lines.push(format!("  {group}:"));
            lines.extend(list.iter().flat_map(student_lines));
        }
    }
    lines.push(String::new());
    lines.push("# タイムライン".to_owned());
    if timeline.timeline.is_empty() {
        lines.push("timeline: []".to_owned());
    } else {
        lines.push("timeline:".to_owned());
        for row in &timeline.timeline {
            for (i, line) in row_lines(row).into_iter().enumerate() {
                lines.push(format!("{}{line}", if i == 0 { "  - " } else { "    " }));
            }
        }
    }
    lines.join("\n") + "\n"
}

/// A student's lines, indented under `striker:` or `special:`.
fn student_lines(student: &Student) -> Vec<String> {
    let lv = student
        .lv
        .map_or_else(|| "null".to_owned(), |lv| lv.to_string());
    let mut lines = vec![
        format!("    - name: {}", quoted(&student.name)),
        format!("      lv: {lv}"),
    ];
    for (key, value) in [
        ("star", &student.star),
        ("skill", &student.skill),
        ("equip", &student.equip),
        ("wb", &student.wb),
    ] {
        lines.push(format!("      {key}: {}", quoted(value)));
    }
    lines
}

/// A row's lines, without the list's indent.
fn row_lines(row: &Row) -> Vec<String> {
    match row {
        Row::At {
            when_at,
            skill,
            target,
            note,
        } => [
            vec![format!("when_at: {}", quoted(when_at))],
            skill_lines(skill, target, note),
        ]
        .concat(),
        Row::Cost {
            when_cost,
            skill,
            target,
            note,
        } => {
            let cost = when_cost.map_or_else(|| "null".to_owned(), |cost| cost.to_string());
            [
                vec![format!("when_cost: {cost}")],
                skill_lines(skill, target, note),
            ]
            .concat()
        }
        Row::Chain {
            skill,
            target,
            note,
        } => skill_lines(skill, target, note),
        Row::Mark { time, note } => vec![
            format!("time: {}", quoted(time)),
            format!("note: {}", quoted(note)),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example of docs/README.md, as the editor writes it.
    const EXAMPLE: &str = include_str!("../../e2e/fixtures/example.yaml");

    fn example() -> Timeline {
        read(EXAMPLE).unwrap()
    }

    fn fields(problems: &[Problem]) -> Vec<&str> {
        problems.iter().map(|p| p.field.as_str()).collect()
    }

    #[test]
    fn reads_the_docs_example() {
        let t = example();
        assert_eq!(
            (
                t.boss.as_str(),
                t.difficulty.as_str(),
                t.armor.as_str(),
                t.terrain.as_str()
            ),
            ("ホド", "INSANE", "重装甲", "屋外")
        );
        assert_eq!(
            t.party.striker[0],
            Student {
                name: "シロコ".to_owned(),
                lv: Some(90),
                star: "固有3".to_owned(),
                skill: "5M7M".to_owned(),
                equip: "T9/9/9".to_owned(),
                wb: "25/25/0".to_owned(),
            }
        );
        assert_eq!(t.party.special.len(), 2);
        assert_eq!(t.timeline.len(), 9);
        assert!(
            matches!(&t.timeline[0], Row::At { when_at, skill, note, .. } if when_at == "3:30" && skill == "コタマ" && note == "開幕バフ")
        );
        assert!(matches!(&t.timeline[1], Row::Chain { skill, .. } if skill == "ホシノ"));
        assert!(matches!(&t.timeline[3], Row::Mark { time, .. } if time == "3:25"));
        assert!(
            matches!(&t.timeline[4], Row::Cost { when_cost: Some(c), .. } if (*c - 7.1).abs() < 1e-9)
        );
        assert!(matches!(&t.timeline[8], Row::At { target, .. } if target == "シロコ"));
        assert_eq!(check(&t), []);
    }

    #[test]
    fn writes_the_docs_example_back_as_it_was() {
        assert_eq!(write(&example()), EXAMPLE.replace("\r\n", "\n"));
    }

    #[test]
    fn what_it_writes_reads_back_the_same() {
        let mut t = example();
        t.boss = "\"ゴズ\" \\ 2".to_owned();
        t.party.special.clear();
        t.timeline.push(Row::Cost {
            when_cost: None,
            skill: "シロコ".to_owned(),
            target: String::new(),
            note: "改行\nを含む".to_owned(),
        });
        t.party.striker[1].lv = None;
        assert_eq!(read(&write(&t)).unwrap(), t);
        assert_eq!(
            read(&write(&Timeline::default())).unwrap(),
            Timeline::default()
        );
    }

    #[test]
    fn says_which_field_is_wrong_and_how() {
        let mut t = example();
        t.boss = " ".to_owned();
        t.difficulty = "LUNATIC".to_owned();
        t.party.striker[0].lv = Some(91);
        t.party.striker[0].star = "星5".to_owned();
        t.party.striker[0].skill = "6M7M".to_owned();
        t.party.striker[1].equip = "9/9/9".to_owned();
        t.party.striker[1].wb = "26/0/0".to_owned();
        t.party.special[1].name = "シロコ".to_owned();
        t.timeline[0] = Row::At {
            when_at: "3:3".to_owned(),
            skill: "ヒナ".to_owned(),
            target: String::new(),
            note: String::new(),
        };
        t.timeline[4] = Row::Cost {
            when_cost: Some(10.5),
            skill: String::new(),
            target: String::new(),
            note: String::new(),
        };
        t.timeline[3] = Row::Mark {
            time: "3:60".to_owned(),
            note: String::new(),
        };
        let problems = check(&t);
        assert_eq!(
            fields(&problems),
            [
                "boss",
                "difficulty",
                "party.striker.0.lv",
                "party.striker.0.star",
                "party.striker.0.skill",
                "party.striker.1.equip",
                "party.striker.1.wb",
                "party.special.1.name",
                "timeline.0.skill",
                "timeline.0.when_at",
                "timeline.3.time",
                "timeline.3.note",
                "timeline.4.skill",
                "timeline.4.when_cost",
                "timeline.8.skill",
            ]
        );
        assert_eq!(problems[0].message, "ボス名を入れてください");
        assert_eq!(problems[7].message, "同じ生徒が編成に二人います");
        assert_eq!(problems[8].message, "編成にいない生徒です");
        assert_eq!(
            problems[14].message, "編成にいない生徒です",
            "セリナ was renamed"
        );
    }

    #[test]
    fn a_new_timeline_asks_for_the_basics() {
        assert_eq!(
            fields(&check(&Timeline::default())),
            ["boss", "difficulty", "armor", "terrain", "party"]
        );
    }

    #[test]
    fn remaining_times_and_levels_follow_the_docs() {
        for ok in ["3:30", "0:05", "1:20.500", "12:00"] {
            assert!(is_time(ok), "{ok}");
        }
        for bad in ["3:3", "3:60", "3:30.5", ":30", "3:30.", "a:30", "330", ""] {
            assert!(!is_time(bad), "{bad}");
        }
        for ok in ["5M7M", "1111", "5MMM", "3777"] {
            assert!(is_skill_levels(ok), "{ok}");
        }
        for bad in ["6M7M", "5M7", "5M7MM", "0M7M", "5m7m", "5A7M"] {
            assert!(!is_skill_levels(bad), "{bad}");
        }
    }

    /// The example as the frontend receives it; the e2e tests send this same file,
    /// so the frontend types and the backend cannot drift apart.
    #[test]
    fn the_frontend_gets_the_shape_the_e2e_tests_use() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../e2e/fixtures/example-timeline.json"
        );
        let expected = serde_json::to_string_pretty(&example()).unwrap() + "\n";
        if std::env::var_os("UPDATE_FIXTURES").is_some() {
            std::fs::write(path, &expected).unwrap();
        }
        let written = std::fs::read_to_string(path).unwrap_or_default();
        assert_eq!(
            written.replace("\r\n", "\n"),
            expected,
            "example-timeline.json is stale: run with UPDATE_FIXTURES=1"
        );
    }

    #[test]
    fn a_file_that_is_no_timeline_is_refused() {
        assert!(read("boss: [").unwrap_err().starts_with("YAML: "));
        assert_eq!(
            read("- 1\n- 2").unwrap_err(),
            "YAML の一番外側がキーと値の組ではありません"
        );
        let partial = read("boss: ホド\ntimeline:\n  - skill: シロコ\n").unwrap();
        assert_eq!(partial.boss, "ホド");
        assert_eq!(
            fields(&check(&partial)),
            [
                "difficulty",
                "armor",
                "terrain",
                "party",
                "timeline.0.skill"
            ]
        );
    }
}
