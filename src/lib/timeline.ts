// The timeline as the backend (src-tauri/src/timeline.rs) sends and receives it.

export interface Student {
  name: string;
  lv: number | null;
  star: string;
  skill: string;
  equip: string;
  wb: string;
}

export type Row =
  | { kind: "at"; when_at: string; skill: string; target: string; note: string }
  | { kind: "cost"; when_cost: number | null; skill: string; target: string; note: string }
  | { kind: "chain"; skill: string; target: string; note: string }
  | { kind: "mark"; time: string; note: string };

export type RowKind = Row["kind"];

export interface Timeline {
  boss: string;
  difficulty: string;
  armor: string;
  terrain: string;
  party: { striker: Student[]; special: Student[] };
  timeline: Row[];
}

/** A mistake and the field it is in (`boss`, `party.striker.0.lv`, `timeline.3.when_at`). */
export interface Problem {
  field: string;
  message: string;
}

export const DIFFICULTIES = ["NORMAL", "HARD", "VERYHARD", "HARDCORE", "EXTREME", "INSANE", "TORMENT"];
export const ARMORS = ["軽装備", "重装甲", "特殊装甲", "弾力装甲", "複合装甲"];
export const TERRAINS = ["市街地", "屋外", "屋内"];
export const STARS = ["星3", "星4", "固有1", "固有2", "固有3", "固有4"];

export const ROW_KINDS: { kind: RowKind; label: string }[] = [
  { kind: "at", label: "時間で発動" },
  { kind: "cost", label: "コストで発動" },
  { kind: "chain", label: "続けて発動" },
  { kind: "mark", label: "時間マーカー" },
];

export function emptyTimeline(): Timeline {
  return {
    boss: "",
    difficulty: "",
    armor: "",
    terrain: "",
    party: { striker: [], special: [] },
    timeline: [],
  };
}

export function emptyStudent(): Student {
  return { name: "", lv: null, star: "", skill: "", equip: "", wb: "" };
}

export function newRow(kind: RowKind): Row {
  return changeKind({ kind: "chain", skill: "", target: "", note: "" }, kind);
}

/** The row as another kind, keeping what both kinds have. */
export function changeKind(row: Row, kind: RowKind): Row {
  if (row.kind === kind) return row;
  const skill = row.kind === "mark" ? { skill: "", target: "", note: row.note } : {
    skill: row.skill,
    target: row.target,
    note: row.note,
  };
  const time = row.kind === "at" ? row.when_at : row.kind === "mark" ? row.time : "";
  switch (kind) {
    case "at":
      return { kind, when_at: time, ...skill };
    case "cost":
      return { kind, when_cost: null, ...skill };
    case "chain":
      return { kind, ...skill };
    case "mark":
      return { kind, time, note: skill.note };
  }
}

/** The rows with the one at `index` moved by `delta`; the same rows when it cannot move. */
export function moveRow(rows: Row[], index: number, delta: number): Row[] {
  const to = index + delta;
  if (to < 0 || to >= rows.length) return rows;
  const moved = [...rows];
  [moved[index], moved[to]] = [moved[to], moved[index]];
  return moved;
}

export function messagesFor(problems: Problem[], field: string): string[] {
  return problems.filter((p) => p.field === field).map((p) => p.message);
}

export function fileName(path: string | null): string {
  return path ? path.split(/[\\/]/).at(-1)! : "無題";
}
