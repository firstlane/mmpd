use crate::keyboard_control::{self, KeyboardControlAdapter};
use std::process::Command;

/// Action run in response to a MIDI event
/// Any Action value can be run through ActionRunner::run.
pub enum Action<'a> {
    /// Sends a key sequence 0 or more times
    /// Use this one for key combinations.
    /// The str argument specifies the key sequence, according to X Keysym notation.
    /// Per example "ctrl+shift+t": emulates pressing the "Ctrl", "Shift" and "t" keys at
    /// the same time.
    /// The number is how many times this key sequence should be entered.
    KeySequence(&'a str, usize),

    /// Enters text as if you typed it on a keyboard
    /// Use this one for text exactly as in the string provided.
    /// The number is how many times this same string should be entered.
    EnterText(&'a str, usize),

    /// Runs a program using the shell, allows running arbitrary programs.
    Shell {
        /// Absolute path to the program to run, without any arguments or options
        command: &'a str,

        /// A list of arguments provided to the command. These end up space-separated.
        /// If one item includes spaces, that item will be surrounded by quotes so it's treated as
        /// one argument.
        args: Option<Vec<&'a str>>,

        /// A list of key/value pairs with environment variables to be provided to the program
        env_vars: Option<Vec<(&'a str, &'a str)>>
    },

    /// A list of actions to be run in the order specified, to allow executing several different
    /// ones for more complex things.
    Combination(Vec<Action<'a>>),

    // This can be expanded upon
}

const DELAY_BETWEEN_KEYS_US: u32 = 100;

/// Struct to give access to running Actions
pub struct ActionRunner {
    kb_adapter: Box<dyn KeyboardControlAdapter>
}

impl ActionRunner {
    /// Set up a new ActionRunner, relying on getting an adapter from keyboard_control.
    /// If no keyboard_control adapter can be obtained, returns None.
    pub fn new() -> Option<ActionRunner> {
        Some(ActionRunner {
            kb_adapter: keyboard_control::get_adapter()?
        })
    }

    /// Executes a given action based on action type
    pub fn run(&self, action: &Action) {
        match action {
            Action::KeySequence(sequence, count) => {
                self.run_key_sequence(sequence, *count);
            },

            Action::EnterText(text, count) => {
                self.run_enter_text(text, *count);
            },

            Action::Shell { command, args, env_vars } => {
                self.run_shell(command, args.clone(), env_vars.clone());
            },

            Action::Combination(actions) => {
                for action in actions {
                    self.run(action);
                }
            },
        }
    }

    fn run_key_sequence(&self, sequence: &str, count: usize) {
        for _ in 0..count {
            self.kb_adapter.send_keysequence(sequence, DELAY_BETWEEN_KEYS_US);
        }
    }

    fn run_enter_text(&self, text: &str, count: usize) {
        for _ in 0..count {
            self.kb_adapter.send_text(text, DELAY_BETWEEN_KEYS_US);
        }
    }

    fn run_shell(
        &self,
        command: &str,
        args: Option<Vec<&str>>,
        env_vars: Option<Vec<(&str, &str)>>
    ) {
        let mut cmd = Command::new(command);

        // TODO: it would be good to be able to substitute certain patterns in any of the strings
        // used in these commands. Substitutable values would essentially include any parameter that
        // was involved in leading to this action being run. That is, any parameters of the
        // MidiMessage, and perhaps access to the whole of the Midi state being stored in memory.
        // This needs further working out to get sensible var names.

        // Attach any arguments
        if let Some(args) = args {
            for arg in args.iter() {
                cmd.arg(arg);
            }
        }

        // Attach any environment variables
        if let Some(env_vars) = env_vars {
            for (env_key, env_val) in env_vars {
                cmd.env(env_key, env_val);
            }
        }

        // Run
        let _ = cmd.status();
    }
}

#[cfg(test)]
mod tests {
    // TODO: add a mocking library to test actions
}