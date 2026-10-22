import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";
import { answer, callsOf, emit, mockTauri } from "./tauri-mock.js";

const example = JSON.parse(
  readFileSync(new URL("./fixtures/example-timeline.json", import.meta.url), "utf8"),
);
const PATH = "C:\\Users\\sensei\\hod-insane.yaml";

/** The page with a backend that finds no problem unless told otherwise. */
async function start(page, answers = {}) {
  await mockTauri(page, { check_timeline: { ok: [] }, ...answers });
  await page.goto("/");
}

async function openExample(page) {
  await answer(page, "plugin:dialog|open", { ok: PATH });
  await answer(page, "open_timeline", { ok: { timeline: example, problems: [] } });
  await page.keyboard.press("Control+o");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("hod-insane.yaml");
}

const lastSaved = async (page) => (await callsOf(page, "save_timeline")).at(-1)?.args;

test("a timeline made from nothing is saved in the docs' shape", async ({ page }) => {
  await start(page, { "plugin:dialog|save": { ok: PATH }, save_timeline: { ok: null } });
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("無題");

  await page.getByLabel("ボス").fill("ホド");
  await page.getByLabel("難易度").selectOption("INSANE");
  await page.getByLabel("装甲").selectOption("重装甲");
  await page.getByLabel("地形").selectOption("屋外");
  await expect(page.getByText("未保存")).toBeVisible();

  await page.getByRole("button", { name: "STRIKERを追加" }).click();
  await page.getByLabel("STRIKER 1 人目の名前").fill("シロコ");
  await page.getByLabel("STRIKER 1 人目のレベル").fill("90");
  await page.getByLabel("STRIKER 1 人目の育成").selectOption("固有3");
  await page.getByLabel("STRIKER 1 人目のスキル").fill("5M7M");
  await page.getByLabel("STRIKER 1 人目の装備").fill("T9/9/9");
  await page.getByLabel("STRIKER 1 人目のWB").fill("25/25/0");
  await page.getByRole("button", { name: "SPECIALを追加" }).click();
  await page.getByLabel("SPECIAL 1 人目の名前").fill("コタマ");

  // The first row waits for a time; the next follows it.
  await page.getByRole("button", { name: "行を追加" }).click();
  await expect(page.getByLabel("1 行目の種類")).toHaveValue("at");
  await page.getByLabel("1 行目の残り時間").fill("3:30");
  await page.getByLabel("1 行目の生徒").selectOption("コタマ");
  await page.getByLabel("1 行目のメモ").fill("開幕バフ");
  await page.getByRole("button", { name: "行を追加" }).click();
  await expect(page.getByLabel("2 行目の種類")).toHaveValue("chain");
  await page.getByLabel("2 行目の生徒").selectOption("シロコ");
  await page.getByRole("button", { name: "行を追加" }).click();
  await page.getByLabel("3 行目の種類").selectOption("cost");
  await page.getByLabel("3 行目のコスト").fill("7.1");
  await page.getByLabel("3 行目の生徒").selectOption("コタマ");
  await page.getByLabel("3 行目の対象").fill("シロコ");
  await page.getByRole("button", { name: "行を追加" }).click();
  await page.getByLabel("4 行目の種類").selectOption("mark");
  await page.getByLabel("4 行目の時刻").fill("3:25");
  await page.getByLabel("4 行目のメモ").fill("NS 発動");

  await page.keyboard.press("Control+s");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("hod-insane.yaml");
  await expect(page.getByText("未保存")).toHaveCount(0);
  expect(await lastSaved(page)).toEqual({
    path: PATH,
    timeline: {
      boss: "ホド",
      difficulty: "INSANE",
      armor: "重装甲",
      terrain: "屋外",
      party: {
        striker: [
          { name: "シロコ", lv: 90, star: "固有3", skill: "5M7M", equip: "T9/9/9", wb: "25/25/0" },
        ],
        special: [{ name: "コタマ", lv: null, star: "", skill: "", equip: "", wb: "" }],
      },
      timeline: [
        { kind: "at", when_at: "3:30", skill: "コタマ", target: "", note: "開幕バフ" },
        { kind: "chain", skill: "シロコ", target: "", note: "" },
        { kind: "cost", when_cost: 7.1, skill: "コタマ", target: "シロコ", note: "" },
        { kind: "mark", time: "3:25", note: "NS 発動" },
      ],
    },
  });
  const titles = (await callsOf(page, "plugin:window|set_title")).map((c) => c.args.value);
  expect(titles.at(-1)).toBe("hod-insane.yaml - Timeline Arona");
  expect(titles).toContain("● 無題 - Timeline Arona");
});

test("an opened timeline shows every part and saves back unchanged", async ({ page }) => {
  await start(page, { save_timeline: { ok: null } });
  await openExample(page);
  await expect(page.getByLabel("ボス")).toHaveValue("ホド");
  await expect(page.getByLabel("地形")).toHaveValue("屋外");
  await expect(page.getByLabel("STRIKER 2 人目の名前")).toHaveValue("ホシノ");
  await expect(page.getByLabel("SPECIAL 2 人目の育成")).toHaveValue("星4");
  const rows = page.getByRole("table", { name: "タイムライン" }).locator("tbody tr");
  await expect(rows).toHaveCount(9);
  await expect(page.getByLabel("4 行目の時刻")).toHaveValue("3:25");
  await expect(page.getByLabel("5 行目のコスト")).toHaveValue("7.1");
  await expect(page.getByLabel("9 行目の対象")).toHaveValue("シロコ");
  await expect(rows.nth(1)).toContainText("前のスキルの直後");

  // Saving goes to the same file without asking.
  await page.getByRole("button", { name: "保存", exact: true }).click();
  expect(await lastSaved(page)).toEqual({ path: PATH, timeline: example });
  expect(await callsOf(page, "plugin:dialog|save")).toHaveLength(0);
});

test("rows move, change kind and go, keeping what they had", async ({ page }) => {
  await start(page);
  await openExample(page);
  await page.getByRole("button", { name: "1 行目を下へ" }).click();
  await expect(page.getByLabel("1 行目の種類")).toHaveValue("chain");
  await expect(page.getByLabel("2 行目の残り時間")).toHaveValue("3:30");
  await expect(page.getByRole("button", { name: "1 行目を上へ" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "9 行目を下へ" })).toBeDisabled();

  await page.getByLabel("2 行目の種類").selectOption("cost");
  await expect(page.getByLabel("2 行目の生徒")).toHaveValue("コタマ");
  await expect(page.getByLabel("2 行目のメモ")).toHaveValue("開幕バフ");
  await expect(page.getByLabel("2 行目のコスト")).toHaveValue("");

  await page.getByRole("button", { name: "4 行目を消す" }).click();
  const rows = page.getByRole("table", { name: "タイムライン" }).locator("tbody tr");
  await expect(rows).toHaveCount(8);
  await expect(page.getByLabel("4 行目のコスト")).toHaveValue("7.1");
});

test("each problem shows at its field", async ({ page }) => {
  const problems = [
    { field: "boss", message: "ボス名を入れてください" },
    { field: "party.striker.0.lv", message: "レベルは 1〜90 です" },
    { field: "timeline.0.when_at", message: "残り時間は M:SS か M:SS.mmm です。例: 3:30" },
    { field: "timeline.8.skill", message: "編成にいない生徒です" },
  ];
  await start(page);
  await openExample(page);
  await answer(page, "check_timeline", { ok: problems });
  await page.getByLabel("ボス").fill("");
  await expect(page.getByRole("status")).toHaveText("4 件の問題");
  await expect(page.getByLabel("ボス")).toHaveAttribute("aria-invalid", "true");
  await expect(page.getByText("ボス名を入れてください")).toBeVisible();
  await expect(page.getByLabel("STRIKER 1 人目のレベル")).toHaveAttribute("aria-invalid", "true");
  await expect(page.getByLabel("1 行目の残り時間")).toHaveAttribute("aria-invalid", "true");
  await expect(page.getByLabel("9 行目の生徒")).toHaveAttribute("aria-invalid", "true");
  await expect(page.getByLabel("2 行目の生徒")).toHaveAttribute("aria-invalid", "false");
  // The backend checked what is on screen.
  expect((await callsOf(page, "check_timeline")).at(-1).args.timeline.boss).toBe("");
});

test("unsaved work is not thrown away without asking", async ({ page }) => {
  await start(page);
  await openExample(page);
  await page.getByLabel("ボス").fill("ゴズ");

  await answer(page, "plugin:dialog|message", { ok: "戻る" });
  await page.keyboard.press("Control+n");
  await expect(page.getByLabel("ボス")).toHaveValue("ゴズ");
  const asked = await callsOf(page, "plugin:dialog|message");
  expect(asked.at(-1).args.message).toBe("保存していない変更があります。破棄しますか？");

  await answer(page, "plugin:dialog|message", { ok: "破棄する" });
  await page.keyboard.press("Control+n");
  await expect(page.getByLabel("ボス")).toHaveValue("");
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("無題");

  // Nothing to lose: no question.
  await page.keyboard.press("Control+n");
  expect(await callsOf(page, "plugin:dialog|message")).toHaveLength(2);
});

test("a file that cannot be read or written says so and keeps the work", async ({ page }) => {
  await start(page);
  await answer(page, "plugin:dialog|open", { ok: PATH });
  await answer(page, "open_timeline", { error: "YAML: did not find expected node content" });
  await page.keyboard.press("Control+o");
  await expect(page.getByRole("alert")).toHaveText("開けませんでした: YAML: did not find expected node content");

  await page.getByLabel("ボス").fill("ホド");
  await answer(page, "plugin:dialog|save", { ok: PATH });
  await answer(page, "save_timeline", { error: `${PATH}: Access is denied.` });
  await page.getByRole("button", { name: "保存", exact: true }).click();
  await expect(page.getByRole("alert")).toHaveText(`保存できませんでした: ${PATH}: Access is denied.`);
  await expect(page.getByText("未保存")).toBeVisible();
  await expect(page.getByLabel("ボス")).toHaveValue("ホド");
});

test("students get their SchaleDB face and names to choose from", async ({ page }) => {
  await start(page);
  await page.getByRole("button", { name: "STRIKERを追加" }).click();
  const striker = page.getByRole("table", { name: "STRIKER" }).locator("tbody tr").first();
  const choices = page.locator("#names-striker option");
  await expect(choices.first()).toBeAttached();
  const names = await choices.evaluateAll((options) => options.map((o) => o.value));
  expect(names).toContain("ホシノ（臨戦）");
  expect(names).not.toContain("ヒマリ"); // a SPECIAL
  expect(await page.locator("#names-special option").evaluateAll((o) => o.map((x) => x.value))).toContain("ヒマリ");

  await expect(striker.locator(".face img")).toHaveCount(0);
  await page.getByLabel("STRIKER 1 人目の名前").fill("ホシノ（臨戦）");
  const face = striker.locator(".face img");
  await expect(face).toHaveAttribute("src", "/schaledb/students/10098.webp");
  await expect.poll(() => face.evaluate((img) => img.naturalWidth)).toBeGreaterThan(0);
  await page.getByLabel("STRIKER 1 人目の名前").fill("カジコ");
  await expect(face).toHaveCount(0);

  await page.getByLabel("STRIKER 1 人目の名前").fill("ホシノ（臨戦）");
  await page.getByRole("button", { name: "行を追加" }).click();
  await page.getByLabel("1 行目の生徒").selectOption("ホシノ（臨戦）");
  const row = page.getByRole("table", { name: "タイムライン" }).locator("tbody tr").first();
  await expect(row.locator(".face img")).toHaveAttribute("src", "/schaledb/students/10098.webp");
});

test("a file the MCP server sends opens without a dialog", async ({ page }) => {
  // Sent before the window was up.
  await start(page, { take_open: { ok: PATH }, open_timeline: { ok: { timeline: example, problems: [] } } });
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("hod-insane.yaml");
  expect((await callsOf(page, "open_timeline")).at(-1).args).toEqual({ path: PATH });

  // Sent while it is up, with unsaved work: asked first.
  await page.getByLabel("ボス").fill("ゴズ");
  await answer(page, "plugin:dialog|message", { ok: "破棄する" });
  const other = "C:/Users/sensei/geburah.yaml";
  await emit(page, "open", other);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("geburah.yaml");
  expect(await callsOf(page, "plugin:dialog|message")).toHaveLength(1);
  expect(await callsOf(page, "plugin:dialog|open")).toHaveLength(0);
});

test("the menu offers the file actions and the theme", async ({ page }) => {
  await start(page, { "plugin:dialog|save": { ok: null } });
  const menu = page.getByRole("button", { name: "メニュー", exact: true });
  await menu.click();
  await expect(menu).toHaveAttribute("aria-expanded", "true");
  await page.getByRole("button", { name: /名前を付けて保存/ }).click();
  expect(await callsOf(page, "plugin:dialog|save")).toHaveLength(1);
  await expect(menu).toHaveAttribute("aria-expanded", "false");

  await menu.click();
  await page.getByRole("button", { name: "テーマ設定" }).click();
  await page.getByRole("radio", { name: "ライト" }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await expect(page.locator("body")).toHaveCSS("background-color", "rgb(250, 246, 239)");
  await page.getByRole("radio", { name: "ダーク" }).click();
  await expect(page.locator("body")).toHaveCSS("background-color", "rgb(25, 25, 25)");
});
