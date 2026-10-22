<script lang="ts">
  import Face from "./Face.svelte";
  import Icon from "./Icon.svelte";
  import type { Roster } from "./students";
  import {
    changeKind,
    messagesFor,
    moveRow,
    newRow,
    ROW_KINDS,
    type Problem,
    type Row,
    type RowKind,
  } from "./timeline";

  let {
    rows = $bindable(),
    students,
    problems,
    roster,
  }: { rows: Row[]; students: string[]; problems: Problem[]; roster: Roster } = $props();

  const messages = (i: number, key: string) => messagesFor(problems, `timeline.${i}.${key}`);
  const name = (i: number, title: string) => `${i + 1} 行目の${title}`;

  function setCost(row: Row, value: string) {
    if (row.kind !== "cost") return;
    const cost = Number.parseFloat(value);
    row.when_cost = Number.isNaN(cost) ? null : cost;
  }

  function setKind(i: number, kind: string) {
    rows[i] = changeKind(rows[i], kind as RowKind);
  }

  function add() {
    rows.push(newRow(rows.length === 0 ? "at" : "chain"));
  }
</script>

<datalist id="students">
  {#each students as student (student)}
    <option value={student}></option>
  {/each}
</datalist>

<div class="table-scroll">
  <table aria-label="タイムライン">
    <thead>
      <tr>
        <th class="num">#</th>
        <th class="kind">種類</th>
        <th class="when">発動</th>
        <th class="student">生徒</th>
        <th class="who">対象</th>
        <th>メモ</th>
        <th class="actions"></th>
      </tr>
    </thead>
    <tbody>
      {#each rows as row, i (i)}
        <tr class:mark={row.kind === "mark"}>
          <td class="num">{i + 1}</td>
          <td>
            <select
              class="input"
              aria-label={name(i, "種類")}
              value={row.kind}
              onchange={(e) => setKind(i, e.currentTarget.value)}
            >
              {#each ROW_KINDS as option (option.kind)}
                <option value={option.kind}>{option.label}</option>
              {/each}
            </select>
          </td>
          <td>
            {#if row.kind === "at"}
              <input
                class="input"
                aria-label={name(i, "残り時間")}
                aria-invalid={messages(i, "when_at").length > 0}
                placeholder="3:30"
                bind:value={row.when_at}
              />
              {#each messages(i, "when_at") as message (message)}<p class="field-error">{message}</p>{/each}
            {:else if row.kind === "cost"}
              <input
                class="input"
                inputmode="decimal"
                aria-label={name(i, "コスト")}
                aria-invalid={messages(i, "when_cost").length > 0}
                placeholder="7.1"
                value={row.when_cost ?? ""}
                oninput={(e) => setCost(row, e.currentTarget.value)}
              />
              {#each messages(i, "when_cost") as message (message)}<p class="field-error">{message}</p>{/each}
            {:else if row.kind === "mark"}
              <input
                class="input"
                aria-label={name(i, "時刻")}
                aria-invalid={messages(i, "time").length > 0}
                placeholder="3:25"
                bind:value={row.time}
              />
              {#each messages(i, "time") as message (message)}<p class="field-error">{message}</p>{/each}
            {:else}
              <span class="follow">前のスキルの直後</span>
            {/if}
          </td>
          {#if row.kind === "mark"}
            <td colspan="2" class="follow">押すスキルなし</td>
          {:else}
            <td>
              <div class="pick">
                <Face {roster} name={row.skill} />
                <select
                  class="input"
                  aria-label={name(i, "生徒")}
                  aria-invalid={messages(i, "skill").length > 0}
                  bind:value={row.skill}
                >
                  <option value="" disabled>選ぶ</option>
                  {#if row.skill && !students.includes(row.skill)}
                    <option value={row.skill}>{row.skill}</option>
                  {/if}
                  {#each students as student (student)}
                    <option value={student}>{student}</option>
                  {/each}
                </select>
              </div>
              {#each messages(i, "skill") as message (message)}<p class="field-error">{message}</p>{/each}
            </td>
            <td>
              <input class="input" list="students" aria-label={name(i, "対象")} bind:value={row.target} />
            </td>
          {/if}
          <td>
            <input
              class="input"
              aria-label={name(i, "メモ")}
              aria-invalid={messages(i, "note").length > 0}
              bind:value={row.note}
            />
            {#each messages(i, "note") as message (message)}<p class="field-error">{message}</p>{/each}
          </td>
          <td class="actions">
            <button
              class="icon-btn"
              type="button"
              aria-label="{i + 1} 行目を上へ"
              disabled={i === 0}
              onclick={() => (rows = moveRow(rows, i, -1))}><Icon name="arrow-up" /></button
            >
            <button
              class="icon-btn"
              type="button"
              aria-label="{i + 1} 行目を下へ"
              disabled={i === rows.length - 1}
              onclick={() => (rows = moveRow(rows, i, 1))}><Icon name="arrow-down" /></button
            >
            <button
              class="icon-btn"
              type="button"
              aria-label="{i + 1} 行目を消す"
              onclick={() => rows.splice(i, 1)}><Icon name="trash" /></button
            >
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
<button class="btn add" type="button" onclick={add}><Icon name="plus" />行を追加</button>

<style lang="sass">
  .num
    width: 32px
    color: var(--c-muted)
    text-align: right

  .kind
    width: 140px

  .when
    width: 112px

  .student
    width: 164px

  .who
    width: 128px

  .pick
    display: flex
    align-items: center
    gap: var(--sp-2)

    .input
      flex: 1
      min-width: 7em

  .actions
    width: 124px
    white-space: nowrap

    .icon-btn
      display: inline-flex

  .follow
    color: var(--c-muted)
    font-size: var(--fs-xs)

  tr.mark td
    background: var(--c-wash-raised)

  .add
    display: inline-flex
    align-items: center
    gap: var(--sp-1)
    margin-top: var(--sp-2)
</style>
