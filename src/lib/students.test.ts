import { describe, expect, test } from "vitest";
import { faceOf, namesOf, type Roster } from "./students";

const roster: Roster = [
  { id: 10005, name: "ホシノ", squad: "striker", released: true },
  { id: 10098, name: "ホシノ（臨戦）", squad: "striker", released: true },
  { id: 10099, name: "ホシノ（臨戦）", squad: "striker", released: true },
  { id: 20022, name: "ヒマリ", squad: "special", released: true },
  { id: 26000, name: "ミカ（未実装）", squad: "special", released: false },
];

describe("faceOf", () => {
  test("a party's name gives the face of that student", () => {
    expect(faceOf(roster, "ヒマリ")).toBe("/schaledb/students/20022.webp");
    expect(faceOf(roster, " ホシノ ")).toBe("/schaledb/students/10005.webp");
  });

  test("a name two forms share gives the first form's face", () => {
    expect(faceOf(roster, "ホシノ（臨戦）")).toBe("/schaledb/students/10098.webp");
  });

  test("an unknown or empty name gives no face", () => {
    expect(faceOf(roster, "カジコ")).toBeNull();
    expect(faceOf(roster, "")).toBeNull();
    expect(faceOf([], "ヒマリ")).toBeNull();
  });
});

describe("namesOf", () => {
  test("the names a group can take, once each, released students only", () => {
    expect(namesOf(roster, "striker")).toEqual(["ホシノ", "ホシノ（臨戦）"]);
    expect(namesOf(roster, "special")).toEqual(["ヒマリ"]);
  });
});
