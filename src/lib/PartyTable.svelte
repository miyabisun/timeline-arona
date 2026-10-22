<script lang="ts">
  import Icon from "./Icon.svelte";
  import { emptyStudent, messagesFor, STARS, type Problem, type Student } from "./timeline";

  let {
    group,
    label,
    students = $bindable(),
    problems,
  }: {
    group: "striker" | "special";
    label: string;
    students: Student[];
    problems: Problem[];
  } = $props();

  type TextKey = "name" | "skill" | "equip" | "wb";
  const texts: { key: TextKey; title: string; placeholder: string }[] = [
    { key: "skill", title: "スキル", placeholder: "5M7M" },
    { key: "equip", title: "装備", placeholder: "T9/9/9" },
    { key: "wb", title: "WB", placeholder: "25/25/0" },
  ];

  const messages = (i: number, key: keyof Student) => messagesFor(problems, `party.${group}.${i}.${key}`);
  const name = (i: number, title: string) => `${label} ${i + 1} 人目の${title}`;

  function setLevel(student: Student, value: string) {
    const lv = Number.parseInt(value, 10);
    student.lv = Number.isNaN(lv) ? null : lv;
  }
</script>

<div class="table-scroll">
  <table aria-label={label}>
    <thead>
      <tr>
        <th>生徒</th>
        <th class="narrow">Lv</th>
        <th class="mid">育成</th>
        <th class="mid">スキル</th>
        <th class="mid">装備</th>
        <th class="mid" title="ワークブック（HP/攻撃力/治癒力）">WB</th>
        <th class="action"></th>
      </tr>
    </thead>
    <tbody>
      {#each students as student, i (i)}
        <tr>
          <td>
            <input
              class="input"
              aria-label={name(i, "名前")}
              aria-invalid={messages(i, "name").length > 0}
              placeholder="シロコ"
              bind:value={student.name}
            />
            {#each messages(i, "name") as message (message)}<p class="field-error">{message}</p>{/each}
          </td>
          <td>
            <input
              class="input"
              inputmode="numeric"
              aria-label={name(i, "レベル")}
              aria-invalid={messages(i, "lv").length > 0}
              placeholder="90"
              value={student.lv ?? ""}
              oninput={(e) => setLevel(student, e.currentTarget.value)}
            />
            {#each messages(i, "lv") as message (message)}<p class="field-error">{message}</p>{/each}
          </td>
          <td>
            <select
              class="input"
              aria-label={name(i, "育成")}
              aria-invalid={messages(i, "star").length > 0}
              bind:value={student.star}
            >
              <option value="" disabled>選ぶ</option>
              {#each STARS as star (star)}
                <option value={star}>{star}</option>
              {/each}
            </select>
            {#each messages(i, "star") as message (message)}<p class="field-error">{message}</p>{/each}
          </td>
          {#each texts as text (text.key)}
            <td>
              <input
                class="input"
                aria-label={name(i, text.title)}
                aria-invalid={messages(i, text.key).length > 0}
                placeholder={text.placeholder}
                bind:value={student[text.key]}
              />
              {#each messages(i, text.key) as message (message)}<p class="field-error">{message}</p>{/each}
            </td>
          {/each}
          <td>
            <button
              class="icon-btn"
              type="button"
              aria-label="{label} {i + 1} 人目を外す"
              onclick={() => students.splice(i, 1)}
            >
              <Icon name="trash" />
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
<button class="btn add" type="button" onclick={() => students.push(emptyStudent())}>
  <Icon name="plus" />{label}を追加
</button>

<style lang="sass">
  .narrow
    width: 64px

  .mid
    width: 104px

  .action
    width: 44px

  .add
    display: inline-flex
    align-items: center
    gap: var(--sp-1)
    margin-top: var(--sp-2)
</style>
