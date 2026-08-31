use action::{CAction, Interface};

use strum::EnumMessage;
use zellij_shell::action;
use zellij_tile::prelude::*;

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    mem::take,
};

#[derive(Debug, Default)]
struct DisplaySize {
    rows: usize,
    columns: usize,
}

#[derive(Debug, Default)]
struct ZellijState {
    auto_execute_when_ready: VecDeque<PipeMessage>,
    permission_granted: bool,
    // current_session: SessionInfo,
    // mode_info: ModeInfo,
    // current_tab: TabInfo,
    // last_pane: PaneManifest,
    client_list: Vec<ClientInfo>,
    current_client_id: Option<ClientId>,
}

#[derive(Debug, Default)]
struct State {
    action: CAction,
    should_open_floating: bool,
    search_filter: EnvironmentFrom,
    display: DisplaySize,
    zellij_state: ZellijState,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ChangeApplicationState,
            PermissionType::MessageAndLaunchOtherPlugins,
            PermissionType::OpenFiles,
            PermissionType::OpenTerminalsOrPlugins,
            PermissionType::ReadApplicationState,
            PermissionType::ReadCliPipes,
            PermissionType::RunCommands,
        ]);

        subscribe(&[
            EventType::Key,
            EventType::ListClients,
            // EventType::ModeUpdate,
            // EventType::PaneUpdate,
            EventType::PermissionRequestResult,
            // EventType::SessionUpdate,
            // EventType::TabUpdate,
        ]);

        eprintln!("Just loaded");
        // TODO: This may change as I’m not convinced the `configuration`’s API is good for this
        // self.action
        //     .set(configuration.get("command").unwrap_or(&Default::default()));
    }

    fn update(&mut self, event: Event) -> bool {
        let mut should_render;

        eprintln!("update {event:?}");

        match event {
            Event::Key(key) => {
                self.handle_key(key);
                should_render = true;
            }
            Event::ListClients(clients) => {
                should_render = true;
                self.zellij_state.client_list = clients;
                self.zellij_state.current_client_id =
                    self.zellij_state.client_list.iter().find_map(|c| {
                        if c.is_current_client {
                            Some(c.client_id)
                        } else {
                            None
                        }
                    });
                eprintln!("New current ID {:?}", self.zellij_state.current_client_id);

                if self.is_ready() {
                    if let Some(message) = self.zellij_state.auto_execute_when_ready.pop_front() {
                        should_render = self.handle_pipe(&message);
                    }
                } else if self.zellij_state.current_client_id.is_none() {
                    // BUG: This is because zellij does not give the list until we un-select and re-select the plugin… Might be a more user-friendly workaround to do that instead of loop requesting hopping to get the value we need. See: https://github.com/zellij-org/zellij/issues/5413
                    list_clients();
                }
            }
            // Event::ModeUpdate(mode_info) => {
            //     self.mode_info = mode_info;
            //     should_render = true;
            // }
            // Event::PaneUpdate(pane_info) => {
            //     self.last_pane = pane_info;
            //     // should_render = true;
            // }
            Event::PermissionRequestResult(PermissionStatus::Granted) => {
                self.zellij_state.permission_granted = true;
                should_render = true;

                list_clients();
            }
            Event::PermissionRequestResult(status) => {
                todo!("Handle receiving permission: {status:?}, aborting for now");
            }
            // Event::SessionUpdate(sessions_info, _resurrectable_sessions) => {
            //     sessions_info.iter().for_each(|s| {
            //         eprintln!(
            //             "Session infos: name: {}, current: {}, clients: {}",
            //             s.name, s.is_current_session, s.connected_clients
            //         )
            //     });
            //     self.zellij_state.current_session = sessions_info
            //         .iter()
            //         .find(|s| s.is_current_session)
            //         .unwrap()
            //         .clone();
            // }
            // Event::TabUpdate(tab_info) => {
            //     self.current_tab = tab_info.iter().find(|t| t.active).unwrap().clone();
            //     // eprintln!("Event::TabUpdate: {tab_info:#?}\n");
            //     // eprintln!("Current Tab: {:#?}\n", self.current_tab);
            //     // should_render = true;
            // }
            _ => unimplemented!("{:?}", event),
        };

        dbg!(should_render)
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        self.handle_pipe(&pipe_message)
    }

    fn render(&mut self, rows: usize, cols: usize) {
        self.change_size(rows, cols);
        // let a = take(&mut self.action);
        let r = self.action.raw().to_owned();

        match self.action.build_action() {
            Ok(b) => print!("{:?}", b),
            // TODO@Visuals: Use `Text` to format the error message
            // TODO@Errors: This string construction feels strange here: we know which message we are about to get
            Err(e) => print!("ERR: `{r:?}` {e:?}"),
        }
    }
}

impl State {
    fn handle_key(&mut self, key: KeyWithModifier) {
        match key {
            // KeyWithModifier {
            //     bare_key: BareKey::Down,
            //     key_modifiers: _,
            // } => self.action.selection_down(),
            // KeyWithModifier {
            //     bare_key: BareKey::Up,
            //     key_modifiers: _,
            // } => self.action.selection_up(),
            KeyWithModifier {
                bare_key: BareKey::Enter,
                key_modifiers: _,
            } => {
                let a = take(&mut self.action);
                match a.build_action() {
                    Ok(b) => zellij_shell::execute_action(b),

                    Err(_) => todo!(),
                }
            } // start_action(None),
            // Key::BackTab => self.open_search_result_in_terminal(),
            KeyWithModifier {
                bare_key: BareKey::Char('f'),
                key_modifiers: modifiers,
            } if modifiers == BTreeSet::from([KeyModifier::Ctrl]) => {
                self.should_open_floating = !self.should_open_floating
            }
            KeyWithModifier {
                bare_key: BareKey::Char('e'),
                key_modifiers: modifiers,
            } if modifiers == BTreeSet::from([KeyModifier::Ctrl]) => self.search_filter.progress(), // TODO: should also be a toggelable bool
            // Key::Esc | Key::Ctrl('c') => {
            //     if !self.search_term.is_empty() {
            //         self.clear_state();
            //     } else {
            //         hide_self();
            //     }
            // }
            _ => self.append_to_search_term(key),
        }
    }

    fn handle_pipe(&mut self, pipe_message: &PipeMessage) -> bool {
        todo!()
        //     let should_render = false; // TODO: needed sometimes? apparently not since changing the action already update the interface… but check it… (if needed add when `set`ting the action?)
        //     const INTERFACE_PIPE: Interface = Interface::Pipe;

        //     let my_client_id = match self.zellij_state.current_client_id {
        //         Some(id) => id,
        //         None => {
        //             self.zellij_state
        //                 .auto_execute_when_ready
        //                 .push_back(pipe_message.to_owned());
        //             return should_render;
        //         }
        //     };

        //     // Sent by us, ignore
        //     if let Some(pipe_client_id) = pipe_message.args.iter().find(|a| a.0 == "client_id") {
        //         let pipe_client_id: ClientId = pipe_client_id.1.parse().unwrap_or_default();
        //         eprintln!(
        //             "{pipe_client_id} ?= {:?}",
        //             self.zellij_state.current_client_id
        //         );
        //         if pipe_client_id == my_client_id {
        //             eprintln!("Ignore message from pipe: same client");
        //             return should_render;
        //         }
        //     }

        //     eprintln!(
        //         "received message from pipe (not ignored) {:#?}",
        //         pipe_message
        //     );

        //     // TODO: add an arg to choose between
        //     //     1) <Default> Don’t replace the text in the openned plugin + run immediatly
        //     //     2) Show in a new plugin pane                              + wait to run
        //     //     3) Show in currently opened plugin pane                   + wait to run
        //     // Option 2 let peoples start to write their command but have the visual help if they are unsure or got an error on previous try
        //     // Option 3 could override already half typed command. Even currently typed one on other user’s screen already half typed command. Even currently typed one on other user’s screen. -> Not a fan of this one
        //     // But not allowing Option 3 means with Option 2 we may spawn quite a lot of panes on unwatched screen
        //     // NOTE: This is kind of already implemented by zellij itself:
        //     //     1) If the configuration is not the same (Ex: no config and `zellij pipe --plugin file:/mnt/Data/Code/Target/Cargo/wasm32-wasi/debug/zellij-console.wasm --args "command=<COMMAND>"`) // TODO: use plugin alias
        //     //     2) Not sure if this is possible
        //     //     3) If the configuration is the same (Ex: no config and `echo <COMMAND> | zellij pipe --plugin file:/mnt/Data/Code/Target/Cargo/wasm32-wasi/debug/zellij-console.wasm`) // TODO: use plugin alias
        //     // Some caveats:
        //     //     * Not sure if the commands run or wait in those scenarios.
        //     //     * The `-c, --plugin-configuration <PLUGIN_CONFIGURATION>` can be used (currently with any config, since it is not used) to always create a new plugin and found yourself in the first scenario.

        //     if let Some(command) = &pipe_message.payload {
        //         self.action.set(command, &INTERFACE_PIPE)
        //     } else if let Some(command) = pipe_message.args.get("command") {
        //         self.action.set(command, &INTERFACE_PIPE)
        //     };

        //     eprintln!("Received action: {}", self.action.command());

        //     let force = pipe_message.args.get("force_available").is_some(); // TODO: just "force"?
        //     let res = format!("{}", self.action.action());
        //     let res = if force {
        //         // Cannot merge those 2 if with `&&` because of: eRFC 2497, "if- and while-let-chains, take 2" see tracking issue https://github.com/rust-lang/rust/issues/53667
        //         // This is annoying for the elses
        //         if let ActionList::Unavailable {
        //             action,
        //             calling_interface,
        //         } = self.action.action()
        //         {
        //             eprintln!("Just force: {action}");
        //             let res = format!("{}", action);
        //             self.start_action(Some((**action).clone()));
        //             res
        //         } else {
        //             eprintln!("Just available");
        //             self.start_action(None);
        //             res
        //         }
        //     } else {
        //         eprintln!("Just execute");
        //         self.start_action(None);
        //         res
        //     };

        //     match &pipe_message.source {
        //         PipeSource::Cli(pipe_name) => {
        //             cli_pipe_output(&pipe_name, &format!("{res}\n"));

        //             // FIXME: Auto-force on only 1 user connected attempt.
        //             //        See: https://github.com/zellij-org/zellij/issues/3580
        //             // TODO: This could also be done for Keybind? but not the text writting
        //             //
        //             // if let ActionList::Unavailable {
        //             //     action,
        //             //     calling_interface,
        //             // } = self.action.action()
        //             // {
        //             //     cli_pipe_output(
        //             //         &pipe_name,
        //             //         &format!("{}\n", self.zellij_state.current_session.connected_clients),
        //             //     );
        //             //     // Cannot merge those 2 if with `&&` because of: eRFC 2497, "if- and while-let-chains, take 2" see tracking issue https://github.com/rust-lang/rust/issues/53667
        //             //     if self.zellij_state.current_session.connected_clients == 0 {
        //             //         // TODO: implement Display for interface?
        //             //         cli_pipe_output(&pipe_name, &format!("Note: forcing this command to run because only 1 user is connected, so it should makes sense even if it is technically usually don’t from `{calling_interface:?}` interface \n"));
        //             //         self.start_action(Some((**action).clone()));
        //             //     }
        //             // }
        //         }
        //         PipeSource::Plugin(_) | PipeSource::Keybind => {
        //             eprintln!("Received (and executed) message {:#?}", pipe_message)
        //         }
        //     }

        //     close_self();
        //     should_render
    }

    fn is_ready(&self) -> bool {
        self.zellij_state.permission_granted && self.zellij_state.current_client_id.is_some()
    }

    fn change_size(&mut self, rows: usize, cols: usize) {
        self.display.rows = rows;
        self.display.columns = cols;
    }

    fn append_to_search_term(&mut self, key: KeyWithModifier) {
        let interface = Interface::Pane; // TODO: receive from parameter?

        // TODO: use modifiers?
        match key.bare_key {
            BareKey::Char(character) => {
                self.action.push(character, &interface);
            }
            BareKey::Backspace => {
                self.action.pop(&interface);
                if self.action.len() == 0 {
                    // self.clear_state();
                }
            }
            _ => {}
        }
    }

    // fn start_action(&mut self, override_action: Option<ActionList>) {
    //     let interface = Interface::Pane; // TODO: receive from parameter?
    //     let action = override_action.unwrap_or_else(|| self.action.action().clone());
    //     let mut done = true;

    //     eprintln!("start_action {action:#?}");

    //     if !self.is_ready() {
    //         eprintln!(
    //             "Not ready to execute command immediately: Not retrieve the whole needed state"
    //         );
    //         return;
    //     }

    //     match action {
    //         ActionList::ClearScreen => {
    //             // TODO: Clear applies on the focused pane, focus to the previous one before clearing the screen/scrollback
    //             // focus_previous_pane();
    //             clear_screen();
    //         }
    //         ActionList::CloseFocus => close_focus(),
    //         ActionList::CloseFocusTab => close_focused_tab(),
    //         ActionList::ClosePluginPane { id } => match id {
    //             Some(id) => close_plugin_pane(id),
    //             None => done = false,
    //         },
    //         ActionList::CloseTerminalPane { id } => match id {
    //             Some(id) => close_terminal_pane(id),
    //             None => done = false,
    //         },
    //         ActionList::DetachEveryone => {
    //             eprintln!("send message to pipe? DE");
    //             if let Interface::Pane = interface {
    //                 eprintln!("send message to pipe DE");
    //                 pipe_message_to_plugin(
    //                     MessageToPlugin::new("message_name")
    //                         .with_plugin_url(OWN_URL)
    //                         .with_payload("DetachMe")
    //                         .with_args(BTreeMap::from([
    //                             (
    //                                 String::from("client_id"),
    //                                 self.zellij_state
    //                                     .current_client_id
    //                                     .expect("Client ID is present since we are ready")
    //                                     .to_string(),
    //                             ),
    //                             (String::from("force_available"), String::new()),
    //                         ])),
    //                 );
    //             }
    //             detach();
    //         }
    //         ActionList::DetachMe => {
    //             eprintln!(
    //                 "Detaching me, client ID: {}",
    //                 self.zellij_state.current_client_id.unwrap()
    //             );
    //             detach();
    //         }
    //         ActionList::DetachOthers => {
    //             eprintln!("send message to pipe? DO");
    //             if let Interface::Pane = interface {
    //                 eprintln!("send message to pipe DO");
    //                 pipe_message_to_plugin(
    //                     MessageToPlugin::new("message_name")
    //                         .with_plugin_url(OWN_URL)
    //                         .with_payload("DetachMe")
    //                         .with_args(BTreeMap::from([
    //                             (
    //                                 String::from("client_id"),
    //                                 self.zellij_state
    //                                     .current_client_id
    //                                     .expect("Client ID is present since we are ready")
    //                                     .to_string(),
    //                             ),
    //                             (String::from("force_available"), String::new()),
    //                         ])),
    //                 );
    //                 eprintln!("sent message to pipe DO");
    //             }
    //         }
    //         // ActionList::EditScrollback => {
    //         //     // TODO: Edit scrollback applies on the focused pane, focus to the previous one before clearing the screen/scrollback
    //         //     // focus_previous_pane();
    //         //     edit_scrollback();
    //         // }
    //         ActionList::Edit(FileToOpen {
    //             path,
    //             line_number,
    //             cwd,
    //         }) => {
    //             let file = FileToOpen {
    //                 path: path.to_owned(),
    //                 line_number: line_number.to_owned(),
    //                 cwd: cwd.to_owned(),
    //             };

    //             if self.should_open_floating {
    //                 open_file_floating(file, None, BTreeMap::default()); // TODO: Make it possible to provide the coordinates & context
    //             } else {
    //                 open_file(file, BTreeMap::default()); // TODO: Make it possible to provide context
    //             }
    //         }
    //         ActionList::NewPane { path } => {
    //             if self.should_open_floating {
    //                 open_terminal_floating(path, None); // TODO: Make it possible to provide the coordinates
    //             } else {
    //                 open_terminal(path);
    //             }
    //         }
    //         ActionList::Run(CommandToRun { path, args, cwd }) => {
    //             let (path, args) = match self.search_filter {
    //                 EnvironmentFrom::ZellijSession => (path, args),
    //                 EnvironmentFrom::DefaultShell => {
    //                     let mut a = vec![
    //                         "-c".to_string(),
    //                         path.to_str().unwrap_or_default().to_string(),
    //                     ];
    //                     a.append(&mut args.clone());

    //                     ("fish".into(), a) // TODO: get user’s shell
    //                 }
    //             };
    //             let cmd = CommandToRun { path, args, cwd };

    //             if self.should_open_floating {
    //                 open_command_pane_floating(cmd, None, BTreeMap::default()); // TODO: Make it possible to provide the coordinates & context
    //             } else {
    //                 open_command_pane(cmd, BTreeMap::default()); // TODO: Make it possible to provide context
    //             }
    //         }

    //         ActionList::HelpAll { selection }
    //         | ActionList::HelpPane { selection }
    //         | ActionList::HelpPipe { selection } => {
    //             done = false;

    //             let mut docs: Box<dyn Iterator<Item = ActionList>> = match action {
    //                 ActionList::HelpAll { .. } => Box::new(ActionList::filter_any()),
    //                 ActionList::HelpPane { .. } => Box::new(ActionList::filter_pane()),
    //                 ActionList::HelpPipe { .. } => Box::new(ActionList::filter_pipe()),
    //                 _ => panic!("Should be one of the help modes"),
    //             };

    //             match selection {
    //                 action::Selection::One { row, max: _ } => {
    //                     let variant = docs
    //                         .nth(row)
    //                         .expect("Selection {selection:?} is bounded to the iter size");
    //                     let s = variant.get_serializations().first().unwrap_or_else(|| {
    //                         panic!("{variant:?} is garanteed to have a serialization string")
    //                     });

    //                     self.action.set(s, &interface);
    //                 }
    //                 action::Selection::Expand => {}
    //             }
    //         }
    //         ActionList::Unknown => {}
    //         ActionList::Unavailable { .. } => {
    //             // TODO: ring a bell, screenshake, print the same text in red? Something like this.
    //             done = false;
    //         }
    //     }

    //     if done {
    //         self.action.clear();
    //     }
    // }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum EnvironmentFrom {
    #[default]
    ZellijSession,
    DefaultShell,
}

impl EnvironmentFrom {
    pub fn progress(&mut self) {
        match self {
            EnvironmentFrom::ZellijSession => *self = EnvironmentFrom::DefaultShell,
            EnvironmentFrom::DefaultShell => *self = EnvironmentFrom::ZellijSession,
        }
    }
}
