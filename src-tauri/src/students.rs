//! The students of SchaleDB's JP data (`public/schaledb/students.json`, made by
//! `tools/schaledb/update.ps1`): the names a timeline uses and their face icons.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Student {
    pub id: u32,
    /// The JP name, an outfit in brackets: `ホシノ（臨戦）`.
    pub name: String,
    /// `striker` or `special`.
    pub squad: String,
    pub released: bool,
}

pub fn parse(json: &str) -> Result<Vec<Student>, String> {
    serde_json::from_str(json).map_err(|e| format!("students.json: {e}"))
}

/// The students `query` may mean: those whose name holds it, or else those whose
/// name without the outfit it holds (`臨戦ホシノ` finds every ホシノ). Hiragana and
/// katakana are one; spaces are ignored. An empty query means everyone.
pub fn search<'a>(students: &'a [Student], query: &str) -> Vec<&'a Student> {
    let query = fold(query);
    let named: Vec<&Student> = students
        .iter()
        .filter(|s| fold(&s.name).contains(&query))
        .collect();
    if !named.is_empty() {
        return named;
    }
    students
        .iter()
        .filter(|s| query.contains(fold(&s.name).split('（').next().unwrap_or_default()))
        .collect()
}

/// Hiragana as katakana, without spaces.
fn fold(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| match c {
            'ぁ'..='ゖ' => char::from_u32(c as u32 + 0x60).unwrap_or(c),
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn student(id: u32, name: &str) -> Student {
        Student {
            id,
            name: name.to_owned(),
            squad: "striker".to_owned(),
            released: true,
        }
    }

    fn roster() -> Vec<Student> {
        vec![
            student(10005, "ホシノ"),
            student(10045, "ホシノ（水着）"),
            student(10098, "ホシノ（臨戦）"),
            student(10099, "ホシノ（臨戦）"),
            student(10000, "アル"),
            student(10066, "カズサ"),
        ]
    }

    fn ids(found: &[&Student]) -> Vec<u32> {
        found.iter().map(|s| s.id).collect()
    }

    #[test]
    fn the_snapshot_reads() {
        let all = parse(include_str!("../../public/schaledb/students.json")).unwrap();
        assert!(all.len() > 250, "{} students", all.len());
        assert!(all.iter().any(|s| s.name == "ホシノ（臨戦）"));
        assert!(
            all.iter()
                .all(|s| s.squad == "striker" || s.squad == "special")
        );
    }

    #[test]
    fn a_name_finds_its_outfits_and_a_nickname_its_student() {
        let all = roster();
        assert_eq!(ids(&search(&all, "ホシノ（臨戦）")), [10098, 10099]);
        assert_eq!(
            ids(&search(&all, "臨戦ホシノ")),
            [10005, 10045, 10098, 10099]
        );
        assert_eq!(ids(&search(&all, "ほしの")), [10005, 10045, 10098, 10099]);
        assert_eq!(ids(&search(&all, " アル ")), [10000]);
        assert_eq!(ids(&search(&all, "カジコ")), Vec::<u32>::new());
        assert_eq!(search(&all, "").len(), all.len());
    }
}
