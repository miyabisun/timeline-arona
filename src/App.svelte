<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { ask, open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  import Header from "./lib/Header.svelte";
  import PartyTable from "./lib/PartyTable.svelte";
  import TimelineTable from "./lib/TimelineTable.svelte";
  import {
    ARMORS,
    DIFFICULTIES,
    emptyTimeline,
    fileName,
    messagesFor,
    TERRAINS,
    type Problem,
    type Timeline,
  } from "./lib/timeline";

  const FILTERS = [{ name: "タイムライン", extensions: ["yaml", "yml"] }];

  let doc = $state<Timeline>(emptyTimeline());
  let path = $state<string | null>(null);
  let saved = $state(JSON.stringify(emptyTimeline()));
  let problems = $state<Problem[]>([]);
  let failure = $state<string | null>(null);

  const dirty = $derived(JSON.stringify(doc) !== saved);
  const students = $derived(
    [...doc.party.striker, ...doc.party.special].map((s) => s.name).filter((n) => n.trim() !== ""),
  );
  const title = $derived(`${dirty ? "● " : ""}${fileName(path)} - Timeline Arona`);

  // Every change is checked by the backend, the one place the rules live.
  $effect(() => {
    const timeline = $state.snapshot(doc);
    let current = true;
    invoke<Problem[]>("check_timeline", { timeline }).then((found) => {
      if (current) problems = found;
    });
    return () => {
      current = false;
    };
  });

  $effect(() => {
    getCurrentWindow().setTitle(title);
  });

  onMount(() => {
    const unlisten = getCurrentWindow().onCloseRequested(async (event) => {
      if (!(await mayDiscard())) event.preventDefault();
    });
    return () => {
      unlisten.then((stop) => stop());
    };
  });

  async function mayDiscard(): Promise<boolean> {
    if (!dirty) return true;
    return ask("保存していない変更があります。破棄しますか？", {
      title: "Timeline Arona",
      kind: "warning",
      okLabel: "破棄する",
      cancelLabel: "戻る",
    });
  }

  function load(timeline: Timeline, at: string | null) {
    doc = timeline;
    path = at;
    saved = JSON.stringify(timeline);
    failure = null;
  }

  async function newFile() {
    if (await mayDiscard()) load(emptyTimeline(), null);
  }

  async function openFile() {
    if (!(await mayDiscard())) return;
    const chosen = await openDialog({ filters: FILTERS });
    if (typeof chosen !== "string") return;
    try {
      const opened = await invoke<{ timeline: Timeline; problems: Problem[] }>("open_timeline", {
        path: chosen,
      });
      load(opened.timeline, chosen);
      problems = opened.problems;
    } catch (error) {
      failure = `開けませんでした: ${error}`;
    }
  }

  async function saveFile(asNew = false) {
    let target = path;
    if (asNew || target === null) {
      target = await saveDialog({ filters: FILTERS, defaultPath: path ?? "timeline.yaml" });
      if (!target) return;
    }
    const timeline = $state.snapshot(doc);
    try {
      await invoke("save_timeline", { path: target, timeline });
      path = target;
      saved = JSON.stringify(timeline);
      failure = null;
    } catch (error) {
      failure = `保存できませんでした: ${error}`;
    }
  }

  function onkeydown(event: KeyboardEvent) {
    if (!event.ctrlKey) return;
    const key = event.key.toLowerCase();
    const action =
      key === "s" ? () => saveFile(event.shiftKey) : key === "o" ? openFile : key === "n" ? newFile : null;
    if (action) {
      event.preventDefault();
      action();
    }
  }
</script>

<svelte:window {onkeydown} />

<Header onnew={newFile} onopen={openFile} onsaveas={() => saveFile(true)} />

<main class="content content-wide">
  <div class="doc-bar">
    <h1>{fileName(path)}</h1>
    {#if dirty}
      <span class="unsaved">未保存</span>
    {/if}
    {#if problems.length > 0}
      <span class="badge" role="status">{problems.length} 件の問題</span>
    {/if}
    <button class="btn btn-primary" type="button" onclick={() => saveFile()}>保存</button>
  </div>

  {#if failure}
    <p class="error-banner" role="alert">{failure}</p>
  {/if}

  <section aria-labelledby="basics">
    <h2 id="basics">基本情報</h2>
    <div class="basics">
      <div>
        <label class="label" for="boss">ボス</label>
        <input
          id="boss"
          class="input"
          placeholder="ホド"
          aria-invalid={messagesFor(problems, "boss").length > 0}
          aria-describedby="boss-errors"
          bind:value={doc.boss}
        />
        <div id="boss-errors">
          {#each messagesFor(problems, "boss") as message (message)}<p class="field-error">{message}</p>{/each}
        </div>
      </div>
      {#each [{ key: "difficulty", label: "難易度", choices: DIFFICULTIES }, { key: "armor", label: "装甲", choices: ARMORS }, { key: "terrain", label: "地形", choices: TERRAINS }] as const as item (item.key)}
        <div>
          <label class="label" for={item.key}>{item.label}</label>
          <select
            id={item.key}
            class="input"
            aria-invalid={messagesFor(problems, item.key).length > 0}
            aria-describedby="{item.key}-errors"
            bind:value={doc[item.key]}
          >
            <option value="" disabled>選ぶ</option>
            {#if doc[item.key] && !item.choices.includes(doc[item.key])}
              <option value={doc[item.key]}>{doc[item.key]}</option>
            {/if}
            {#each item.choices as choice (choice)}
              <option value={choice}>{choice}</option>
            {/each}
          </select>
          <div id="{item.key}-errors">
            {#each messagesFor(problems, item.key) as message (message)}<p class="field-error">{message}</p>{/each}
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section aria-labelledby="party">
    <h2 id="party">編成</h2>
    {#each messagesFor(problems, "party") as message (message)}<p class="field-error">{message}</p>{/each}
    <h3>STRIKER</h3>
    <PartyTable group="striker" label="STRIKER" bind:students={doc.party.striker} {problems} />
    <h3>SPECIAL</h3>
    <PartyTable group="special" label="SPECIAL" bind:students={doc.party.special} {problems} />
  </section>

  <section aria-labelledby="rows">
    <h2 id="rows">タイムライン</h2>
    <TimelineTable bind:rows={doc.timeline} {students} {problems} />
  </section>
</main>

<style lang="sass">
  .doc-bar
    display: flex
    align-items: center
    gap: var(--sp-3)
    margin-bottom: var(--sp-4)

    h1
      min-width: 0
      margin: 0
      overflow: hidden
      font-size: var(--fs-xl)
      font-weight: 600
      line-height: 1.3
      white-space: nowrap
      text-overflow: ellipsis

  .unsaved
    color: var(--c-muted)
    font-size: var(--fs-xs)

  .badge
    margin-left: auto
    padding: 2px var(--sp-2)
    border: 1px solid var(--c-danger)
    border-radius: var(--radius-full)
    color: var(--c-danger)
    font-size: var(--fs-xs)

  section
    margin-bottom: var(--sp-5)

  h2
    margin: 0 0 var(--sp-2)
    font-size: var(--fs-lg)
    font-weight: 600

  h3
    margin: var(--sp-3) 0 var(--sp-1)
    color: var(--c-muted)
    font-size: var(--fs-sm)
    font-weight: 500

  .basics
    display: grid
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr))
    gap: var(--sp-3)

  .label
    display: block
    margin-bottom: 2px
    color: var(--c-muted)
    font-size: var(--fs-xs)
</style>
