import { describe, expect, test } from "vitest";
import {
  changeKind,
  emptyTimeline,
  fileName,
  messagesFor,
  moveRow,
  newRow,
  type Row,
} from "./timeline";

const skillAt: Row = { kind: "at", when_at: "3:30", skill: "コタマ", target: "シロコ", note: "開幕バフ" };

describe("changeKind", () => {
  test("keeps the skill, target and note between skill rows", () => {
    expect(changeKind(skillAt, "cost")).toEqual({
      kind: "cost",
      when_cost: null,
      skill: "コタマ",
      target: "シロコ",
      note: "開幕バフ",
    });
    expect(changeKind(skillAt, "chain")).toEqual({
      kind: "chain",
      skill: "コタマ",
      target: "シロコ",
      note: "開幕バフ",
    });
  });

  test("keeps the time and the note between a timed skill and a marker", () => {
    const mark = changeKind(skillAt, "mark");
    expect(mark).toEqual({ kind: "mark", time: "3:30", note: "開幕バフ" });
    expect(changeKind(mark, "at")).toEqual({
      kind: "at",
      when_at: "3:30",
      skill: "",
      target: "",
      note: "開幕バフ",
    });
  });

  test("leaves a row of the same kind as it is", () => {
    expect(changeKind(skillAt, "at")).toBe(skillAt);
  });
});

test("new rows start empty", () => {
  expect(newRow("at")).toEqual({ kind: "at", when_at: "", skill: "", target: "", note: "" });
  expect(newRow("mark")).toEqual({ kind: "mark", time: "", note: "" });
});

test("a row moves up or down, never past either end", () => {
  const rows = [newRow("at"), newRow("chain"), newRow("mark")];
  expect(moveRow(rows, 2, -1).map((r) => r.kind)).toEqual(["at", "mark", "chain"]);
  expect(moveRow(rows, 0, 1).map((r) => r.kind)).toEqual(["chain", "at", "mark"]);
  expect(moveRow(rows, 0, -1)).toBe(rows);
  expect(moveRow(rows, 2, 1)).toBe(rows);
});

test("a field's messages come from the problems naming it", () => {
  const problems = [
    { field: "boss", message: "ボス名を入れてください" },
    { field: "timeline.3.when_at", message: "残り時間は M:SS か M:SS.mmm です。例: 3:30" },
    { field: "timeline.3.when_at", message: "two" },
  ];
  expect(messagesFor(problems, "timeline.3.when_at")).toEqual([
    "残り時間は M:SS か M:SS.mmm です。例: 3:30",
    "two",
  ]);
  expect(messagesFor(problems, "timeline.3")).toEqual([]);
});

test("a new timeline has every list empty", () => {
  expect(emptyTimeline()).toEqual({
    boss: "",
    difficulty: "",
    armor: "",
    terrain: "",
    party: { striker: [], special: [] },
    timeline: [],
  });
});

test("the file name is the last part of the path", () => {
  expect(fileName("C:\\Users\\sensei\\hod-insane.yaml")).toBe("hod-insane.yaml");
  expect(fileName("/home/sensei/hod.yaml")).toBe("hod.yaml");
  expect(fileName(null)).toBe("無題");
});
