//! The MCP server's protocol (JSON-RPC 2.0, one message a line on stdin and
//! stdout): the handshake, the list of tools and the calls, which `Tools` carries
//! out. `server.rs` connects it to the app.

use serde_json::{Value, json};

/// The protocol versions this server speaks, newest first.
pub const VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// What carries out a tool call: the result as JSON, or what went wrong.
pub trait Tools {
    fn call(&mut self, name: &str, args: &Value) -> Result<Value, String>;
}

/// The tools, as `tools/list` describes them.
pub fn tool_list() -> Value {
    let crop = json!({
        "type": "object",
        "description": "The part of the picture to keep, as fractions of its width and height (0 to 1). The whole picture when left out.",
        "properties": {
            "x": {"type": "number"}, "y": {"type": "number"},
            "w": {"type": "number"}, "h": {"type": "number"}
        },
        "required": ["x", "y", "w", "h"]
    });
    json!([
        {
            "name": "video_open",
            "description": "Opens a YouTube video in Timeline Arona's video window and waits past any ad until the video can be read. Returns its title, channel, length in seconds and description (chapters included).",
            "inputSchema": {
                "type": "object",
                "properties": {"url": {"type": "string", "description": "youtube.com/watch?v=… or youtu.be/…"}},
                "required": ["url"]
            }
        },
        {
            "name": "video_frames",
            "description": "Seeks the open video to the times from `from` to `to` every `every` seconds and saves each picture as a PNG (the video itself: no player controls or mouse pointer). Returns the folder and each frame's time and path. With `changes_only`, a frame that looks like the last one kept (in the cropped part) is dropped, which keeps one frame per state of an overlaid text.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from": {"type": "number", "description": "seconds into the video"},
                    "to": {"type": "number", "description": "seconds into the video, included"},
                    "every": {"type": "number", "description": "seconds between frames"},
                    "crop": crop,
                    "changes_only": {"type": "boolean"}
                },
                "required": ["from", "to", "every"]
            }
        },
        {
            "name": "students",
            "description": "The students of SchaleDB's JP data whose name holds the query, or whose name without the outfit the query holds (\"臨戦ホシノ\" finds every ホシノ). Each comes with id, name (outfits in brackets: ホシノ（臨戦）), squad (striker or special) and the path of the face icon. Timelines use these names.",
            "inputSchema": {
                "type": "object",
                "properties": {"query": {"type": "string", "description": "a name or part of one; everyone when left out"}}
            }
        },
        {
            "name": "timeline_check",
            "description": "Checks a timeline YAML (the format of timeline-arona's docs/README.md, which Timeline Plana reads). Returns the problems, each with its field, and the YAML in the editor's layout.",
            "inputSchema": {
                "type": "object",
                "properties": {"yaml": {"type": "string"}},
                "required": ["yaml"]
            }
        },
        {
            "name": "timeline_open",
            "description": "Opens a timeline YAML file in Timeline Arona's editor window for the Sensei to look over.",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string", "description": "absolute path of the file"}},
                "required": ["path"]
            }
        }
    ])
}

fn answer(id: &Value, result: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "result": result}).to_string()
}

fn failure(id: &Value, code: i64, message: &str) -> String {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}}).to_string()
}

/// The answer to one line from the client; `None` for notifications and for
/// the client's own answers.
pub fn respond(line: &str, tools: &mut dyn Tools) -> Option<String> {
    if line.trim().is_empty() {
        return None;
    }
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return Some(failure(&Value::Null, -32700, "the line is no JSON"));
    };
    let (Some(method), Some(id)) = (message["method"].as_str(), message.get("id")) else {
        return None;
    };
    let params = &message["params"];
    Some(match method {
        "initialize" => {
            let asked = params["protocolVersion"].as_str().unwrap_or_default();
            let version = VERSIONS
                .into_iter()
                .find(|v| *v == asked)
                .unwrap_or(VERSIONS[0]);
            answer(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "timeline-arona", "version": env!("CARGO_PKG_VERSION")}
                }),
            )
        }
        "ping" => answer(id, json!({})),
        "tools/list" => answer(id, json!({"tools": tool_list()})),
        "tools/call" => {
            let name = params["name"].as_str().unwrap_or_default();
            if !tool_list()
                .as_array()
                .is_some_and(|list| list.iter().any(|t| t["name"] == name))
            {
                return Some(failure(id, -32602, &format!("no tool named {name:?}")));
            }
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let (text, is_error) = match tools.call(name, &args) {
                Ok(result) => (
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                    false,
                ),
                Err(message) => (message, true),
            };
            answer(
                id,
                json!({"content": [{"type": "text", "text": text}], "isError": is_error}),
            )
        }
        _ => failure(id, -32601, &format!("no method {method:?}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tools that echo their call, or fail when asked to.
    #[derive(Default)]
    struct Echo {
        calls: Vec<(String, Value)>,
    }

    impl Tools for Echo {
        fn call(&mut self, name: &str, args: &Value) -> Result<Value, String> {
            self.calls.push((name.to_owned(), args.clone()));
            if args.get("fail").is_some() {
                Err("動画を開けません".to_owned())
            } else {
                Ok(json!({"tool": name}))
            }
        }
    }

    fn ask(line: &str, tools: &mut Echo) -> Value {
        serde_json::from_str(&respond(line, tools).expect("an answer")).unwrap()
    }

    #[test]
    fn the_handshake_names_the_server_and_agrees_on_a_version() {
        let mut tools = Echo::default();
        let answer = ask(
            r#"{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"claude-code","version":"2"}}}"#,
            &mut tools,
        );
        assert_eq!(answer["id"], 0);
        assert_eq!(answer["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(answer["result"]["serverInfo"]["name"], "timeline-arona");
        assert!(answer["result"]["capabilities"]["tools"].is_object());

        let newer = ask(
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2099-01-01"}}"#,
            &mut tools,
        );
        assert_eq!(newer["result"]["protocolVersion"], VERSIONS[0]);
    }

    #[test]
    fn notifications_and_answers_get_no_reply() {
        let mut tools = Echo::default();
        for line in [
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            r#"{"jsonrpc":"2.0","id":5,"result":{}}"#,
            "",
            "   ",
        ] {
            assert_eq!(respond(line, &mut tools), None, "{line}");
        }
    }

    #[test]
    fn the_tools_are_listed_with_their_inputs() {
        let mut tools = Echo::default();
        let answer = ask(
            r#"{"jsonrpc":"2.0","id":"a","method":"tools/list"}"#,
            &mut tools,
        );
        assert_eq!(answer["id"], "a");
        let listed = answer["result"]["tools"].as_array().unwrap();
        let names: Vec<&str> = listed.iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert_eq!(
            names,
            [
                "video_open",
                "video_frames",
                "students",
                "timeline_check",
                "timeline_open"
            ]
        );
        for tool in listed {
            assert!(!tool["description"].as_str().unwrap().is_empty());
            assert_eq!(tool["inputSchema"]["type"], "object");
        }
    }

    #[test]
    fn a_call_reaches_the_tool_and_its_failure_is_a_tool_error() {
        let mut tools = Echo::default();
        let ok = ask(
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"students","arguments":{"query":"アル"}}}"#,
            &mut tools,
        );
        assert_eq!(ok["result"]["isError"], false);
        let text = ok["result"]["content"][0]["text"].as_str().unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(text).unwrap(),
            json!({"tool": "students"})
        );
        assert_eq!(
            tools.calls,
            [("students".to_owned(), json!({"query": "アル"}))]
        );

        let failed = ask(
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"video_open","arguments":{"fail":true}}}"#,
            &mut tools,
        );
        assert_eq!(failed["result"]["isError"], true);
        assert_eq!(failed["result"]["content"][0]["text"], "動画を開けません");

        let bare = ask(
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"students"}}"#,
            &mut tools,
        );
        assert_eq!(bare["result"]["isError"], false);
        assert_eq!(tools.calls[2].1, json!({}));
    }

    #[test]
    fn unknown_tools_methods_and_broken_lines_are_protocol_errors() {
        let mut tools = Echo::default();
        let unknown_tool = ask(
            r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"rm","arguments":{}}}"#,
            &mut tools,
        );
        assert_eq!(unknown_tool["error"]["code"], -32602);
        let unknown_method = ask(
            r#"{"jsonrpc":"2.0","id":7,"method":"resources/list"}"#,
            &mut tools,
        );
        assert_eq!(unknown_method["error"]["code"], -32601);
        let broken = ask("{not json", &mut tools);
        assert_eq!(broken["error"]["code"], -32700);
        assert_eq!(broken["id"], Value::Null);
        assert!(tools.calls.is_empty());

        let ping = ask(r#"{"jsonrpc":"2.0","id":8,"method":"ping"}"#, &mut tools);
        assert_eq!(ping["result"], json!({}));
    }
}
