use std::collections::HashSet;

use promkit_widgets::{core::crossterm::event::Event, jsonstream, listbox, text_editor};
use serde::{Deserialize, Serialize};
use termcfg::crossterm_config::event_set_serde;
use tokio::time::Duration;

mod duration;
use duration::duration_serde;

#[derive(Serialize, Deserialize)]
pub struct EditorConfig {
    pub on_focus: text_editor::Config,
    pub on_defocus: text_editor::Config,
}

#[derive(Serialize, Deserialize)]
pub struct JsonConfig {
    pub max_streams: Option<usize>,
    pub stream: jsonstream::Config,
}

#[derive(Serialize, Deserialize)]
pub struct CompletionConfig {
    pub listbox: listbox::Config,
    pub search_result_chunk_size: usize,
    pub search_load_chunk_size: usize,
}

// TODO: remove Clone derive
#[derive(Clone, Serialize, Deserialize)]
pub struct Keybinds {
    #[serde(with = "event_set_serde")]
    pub exit: HashSet<Event>,
    #[serde(default, with = "event_set_serde")]
    pub cancel: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub copy_query: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub copy_result: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub switch_mode: HashSet<Event>,
    pub on_editor: EditorKeybinds,
    pub on_json_viewer: JsonViewerKeybinds,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EditorKeybinds {
    #[serde(with = "event_set_serde")]
    pub backward: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub forward: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub move_to_head: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub move_to_tail: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub move_to_previous_nearest: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub move_to_next_nearest: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub erase: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub erase_all: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub erase_to_previous_nearest: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub erase_to_next_nearest: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub completion: HashSet<Event>,
    pub on_completion: CompletionKeybinds,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CompletionKeybinds {
    #[serde(with = "event_set_serde")]
    pub up: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub down: HashSet<Event>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JsonViewerKeybinds {
    #[serde(with = "event_set_serde")]
    pub up: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub down: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub move_to_head: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub move_to_tail: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub toggle: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub expand: HashSet<Event>,
    #[serde(with = "event_set_serde")]
    pub collapse: HashSet<Event>,
}

#[derive(Serialize, Deserialize)]
pub struct ReactivityControl {
    #[serde(with = "duration_serde")]
    pub query_debounce_duration: Duration,

    #[serde(with = "duration_serde")]
    pub resize_debounce_duration: Duration,

    #[serde(with = "duration_serde")]
    pub spin_duration: Duration,
}

pub static DEFAULT_CONFIG: &str = include_str!("../default.toml");

/// Note that the config struct and the `.toml` configuration file are
/// managed separately because the current toml crate
/// does not readily support the following features:
///
/// - Preserve docstrings as comments in the `.toml` file
///   - https://github.com/toml-rs/toml/issues/376
/// - Output inline tables
///   - https://github.com/toml-rs/toml/issues/592
///
/// Also difficult to patch `Config` using only the items specified in the configuration file
/// (Premise: To address the complexity of configurations,
/// it assumes using a macro to avoid managing Option-wrapped structures on our side).s
///
/// The main challenge is that, for nested structs,
/// it is not able to wrap every leaf field with Option<>.
/// https://github.com/colin-kiegel/rust-derive-builder/issues/254
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub no_hint: bool,
    pub reactivity_control: ReactivityControl,
    pub editor: EditorConfig,
    pub json: JsonConfig,
    pub completion: CompletionConfig,
    pub keybinds: Keybinds,
}

impl Config {
    pub fn load_from(content: &str) -> anyhow::Result<Self> {
        toml::from_str(content).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use promkit_widgets::core::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_default_config_keybinds() {
        let config = Config::load_from(DEFAULT_CONFIG).expect("Failed to parse DEFAULT_CONFIG");

        let enter_event = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(
            config.keybinds.exit.contains(&enter_event),
            "exit keybind should contain Enter"
        );

        let ctrl_c_event = Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(
            config.keybinds.cancel.contains(&ctrl_c_event),
            "cancel keybind should contain Ctrl+C"
        );

        let space_event = Event::Key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(
            config.keybinds.on_json_viewer.toggle.contains(&space_event),
            "toggle keybind should contain Space"
        );
    }

    #[test]
    fn test_missing_cancel_in_config_defaults_to_empty() {
        let legacy_config = r#"
no_hint = false

[reactivity_control]
query_debounce_duration = "200ms"
resize_debounce_duration = "100ms"
spin_duration = "10ms"

[editor.on_focus]
[editor.on_defocus]

[json]
[json.stream]

[completion]
search_result_chunk_size = 10
search_load_chunk_size = 10
[completion.listbox]

[keybinds]
exit = ["Ctrl+C"]
copy_query = ["Ctrl+Q"]
copy_result = ["Ctrl+O"]
switch_mode = ["Shift+Down", "Shift+Up"]

[keybinds.on_editor]
backward = ["Left"]
forward = ["Right"]
move_to_head = ["Ctrl+A"]
move_to_tail = ["Ctrl+E"]
move_to_previous_nearest = ["Alt+B"]
move_to_next_nearest = ["Alt+F"]
erase = ["Backspace"]
erase_all = ["Ctrl+U"]
erase_to_previous_nearest = ["Ctrl+W"]
erase_to_next_nearest = ["Alt+D"]
completion = ["Tab"]
on_completion.up = ["Up"]
on_completion.down = ["Down"]

[keybinds.on_json_viewer]
up = ["Up"]
down = ["Down"]
move_to_head = ["Ctrl+L"]
move_to_tail = ["Ctrl+H"]
toggle = ["Enter"]
expand = []
collapse = []
"#;
        let config = Config::load_from(legacy_config).expect("Failed to parse config without cancel key");
        assert!(config.keybinds.cancel.is_empty());
    }
}
