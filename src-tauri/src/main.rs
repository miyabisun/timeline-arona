// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // `timeline-arona mcp` serves MCP on stdin and stdout instead of opening the editor.
    timeline_arona_lib::run(std::env::args().nth(1).as_deref() == Some("mcp"));
}
