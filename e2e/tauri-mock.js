// A stand-in for the Tauri backend in a plain browser: commands answer from
// `window.__mock.answers` (changeable with `answer`), and every call is recorded
// in `window.__mock.calls`.

/** Installs the mock before the page's own scripts run. */
export async function mockTauri(page, answers = {}) {
  await page.addInitScript((answers) => {
    const callbacks = new Map();
    let nextId = 1;
    window.__mock = { answers, calls: [] };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    window.__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: "main" },
        currentWebview: { windowLabel: "main", label: "main" },
      },
      transformCallback(callback) {
        const id = nextId++;
        callbacks.set(id, callback);
        return id;
      },
      unregisterCallback(id) {
        callbacks.delete(id);
      },
      async invoke(cmd, args) {
        if (cmd === "plugin:event|listen") return nextId++;
        if (cmd === "plugin:event|unlisten") return null;
        window.__mock.calls.push({ cmd, args: JSON.parse(JSON.stringify(args ?? {})) });
        const answer = window.__mock.answers[cmd];
        if (answer && "error" in answer) throw answer.error;
        return answer ? answer.ok : null;
      },
    };
  }, answers);
}

/** Changes what `cmd` answers from now on. */
export async function answer(page, cmd, value) {
  await page.evaluate(([cmd, value]) => (window.__mock.answers[cmd] = value), [cmd, value]);
}

/** The calls of `cmd` so far. */
export async function callsOf(page, cmd) {
  const all = await page.evaluate(() => window.__mock.calls);
  return all.filter((c) => c.cmd === cmd);
}
