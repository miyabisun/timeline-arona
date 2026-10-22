// A stand-in for the Tauri backend in a plain browser: commands answer from
// `window.__mock.answers` (changeable with `answer`), every call is recorded
// in `window.__mock.calls`, and `emit` sends the page an event as the backend would.

/** Installs the mock before the page's own scripts run. */
export async function mockTauri(page, answers = {}) {
  await page.addInitScript((answers) => {
    const callbacks = new Map();
    let nextId = 1;
    const listeners = [];
    window.__mock = {
      answers,
      calls: [],
      emit(event, payload) {
        for (const l of listeners.filter((l) => l.event === event)) callbacks.get(l.handler)?.({ event, id: 0, payload });
      },
    };
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
        if (cmd === "plugin:event|listen") {
          listeners.push({ event: args.event, handler: args.handler });
          return nextId++;
        }
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

/** Sends the page `event` from the backend. */
export async function emit(page, event, payload) {
  await page.evaluate(([event, payload]) => window.__mock.emit(event, payload), [event, payload]);
}

/** The calls of `cmd` so far. */
export async function callsOf(page, cmd) {
  const all = await page.evaluate(() => window.__mock.calls);
  return all.filter((c) => c.cmd === cmd);
}
