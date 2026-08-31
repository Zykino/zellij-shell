pub mod builtin;

use std::path::PathBuf;

use strum::{EnumMessage, EnumProperty, IntoEnumIterator};
use strum_macros::{EnumIter, EnumMessage, EnumProperty};

#[cfg(not(test))]
use zellij_tile::prelude::{CommandToRun, FileToOpen};

use crate::action::builtin::{BuiltinFactoryFull, BuiltinFull};

// TODO@Errors: Do not use `&str`
const INVALID_ACTION_ERR_VALUE: &str = "is not a valid command";

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub(crate) enum Selection {
    One {
        row: usize,
        max: usize,
    },
    #[default]
    Expand,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    #[default]
    All, // TODO@Decision: Do not expose All, and the function should return an array (so All is not the magic of every other elements)?
    Pane,
    Pipe,
}

/// All the action this plugin can execute.
///
/// There are currently 2 ways of sending commands to this plugin:
/// 1) Writting in the plugin interface
/// 2) Sending a `command` message through a pipe
///
/// Some actions only makes sense in one or the other interactive way, and others are available on all. Note the different `helps` are available on all interfaces so you can view them on your prefered one.

// Here I'm using `strum`'s ` EnumProperty` to keep a common list but activate certain command only for some  differentiate between the 2. The `props(Interface = "<interface-name>")` can take the following values:
// 1) "Pane": Only available on the "pane" interface // TODO: "pane" or "plugin" or "plugin-pane"?
// 2) "Pipe": Only available on the "pipe" interface
// 3) "All": Available "all" the interfaces
// TODO: Is it possible to not repeat the `serialize` with all "-" and "_" combinations?
// #[derive(Debug, Default, Clone, EnumIter, EnumMessage, EnumProperty)]
// pub(crate) enum ActionList {
//     /*
//         Technical actions
//     */
//     /// No action have been recognized
//     #[default]
//     Unknown,
//     /// This recognized `action` is not available through the requested `calling_interface`
//     //
//     // This is an equivalent of custom Result::Error.
//     // TODO: is there a better way to integreate this?
//     Unavailable {
//         action: Box<ActionList>,
//         calling_interface: Interface,
//     },
//     /// Show the list of all commands
//     #[strum(
//         props(Interface = "All"),
//         serialize = "Help",
//         serialize = "HelpAll",
//         serialize = "Help-All",
//         serialize = "Help_All",
//         serialize = "h",
//         serialize = "?"
//     )]
//     HelpAll { selection: Selection },
//     /// Show the list of commands available through the `pane` interface
//     #[strum(
//         props(Interface = "All"),
//         serialize = "HelpPane",
//         serialize = "Help-Pane",
//         serialize = "Help_Pane"
//     )]
//     HelpPane { selection: Selection },
//     /// Show the list of commands available through the `Pipe`
//     #[strum(
//         props(Interface = "All"),
//         serialize = "HelpPipe",
//         serialize = "Help-Pipe",
//         serialize = "Help_Pipe"
//     )]
//     HelpPipe { selection: Selection },

//     /*
//         Zellij actions (sorted alphabetically)
//     */
//     /// Clear the last focused pane’s scroll buffer
//     #[strum(
//         props(Interface = "All"),
//         serialize = "ClearScreen",
//         serialize = "Clear-Screen",
//         serialize = "Clear_Screen",
//         serialize = "Clear",
//         serialize = "cl"
//     )]
//     ClearScreen,
//     /// Close the focused pane
//     #[strum(
//         props(Interface = "All"),
//         serialize = "Close",
//         serialize = "Exit",
//         serialize = "CloseFocus",
//         serialize = "Close-Focus",
//         serialize = "Close_Focus",
//         serialize = "ClosePane",
//         serialize = "Close-Pane",
//         serialize = "Close_Pane",
//         serialize = "ExitFocus",
//         serialize = "Exit-Focus",
//         serialize = "Exit_Focus",
//         serialize = "ExitPane",
//         serialize = "Exit-Pane",
//         serialize = "Exit_Pane"
//     )]
//     CloseFocus,
//     /// Close the focused tab
//     #[strum(
//         props(Interface = "All"),
//         serialize = "CloseFocusTab",
//         serialize = "Close-Focus-Tab",
//         serialize = "Close_Focus_Tab",
//         serialize = "CloseTab",
//         serialize = "Close-Tab",
//         serialize = "Close_Tab"
//     )]
//     CloseFocusTab,
//     /// Close the specified plugin pane
//     #[strum(
//         props(Interface = "All"),
//         serialize = "ClosePluginPane",
//         serialize = "Close-Plugin-Pane",
//         serialize = "Close_Plugin_Pane",
//         serialize = "ClosePlugin",
//         serialize = "Close-Plugin",
//         serialize = "Close_Plugin"
//     )]
//     ClosePluginPane { id: Option<u32> },
//     /// Close the specified terminal pane
//     #[strum(
//         props(Interface = "All"),
//         serialize = "CloseTerminalPane",
//         serialize = "Close-Terminal-Pane",
//         serialize = "Close_Terminal_Pane",
//         serialize = "CloseTerminal",
//         serialize = "Close-Terminal",
//         serialize = "Close_Terminal"
//     )]
//     CloseTerminalPane { id: Option<u32> },
//     /// Detach everyone from the current session
//     #[strum(
//         props(Interface = "All"),
//         serialize = "DetachEveryone",
//         serialize = "Detach-Everyone",
//         serialize = "Detach_Everyone"
//     )]
//     DetachEveryone,
//     /// Detach me from the current session
//     #[strum(
//         props(Interface = "Pane"), // The cli does not know who wrote the command or who pressed `Enter`
//         serialize = "DetachMe",
//         serialize = "Detach-Me",
//         serialize = "Detach_Me"
//     )]
//     DetachMe,
//     /// Detach other clients from the current session
//     #[strum(
//         props(Interface = "Pane"), // The cli does not know who wrote the command or who pressed `Enter`
//         serialize = "DetachOthers",
//         serialize = "Detach-Others",
//         serialize = "Detach_Others"
//     )]
//     DetachOthers,
//     // /// Edit a pane scrollback
//     // EditScrollback,
//     /// Edit a file in a new edit pane
//     #[cfg(not(test))]
//     #[strum(
//         props(Interface = "Pane"), // The cli does not know who wrote the command or who pressed `Enter` --> The edit will be done by every users
//     )]
//     Edit(FileToOpen),
//     /// Open a new pane in the current tab
//     #[strum(
//         props(Interface = "Pane"), // The cli does not know who wrote the command or who pressed `Enter` --> The edit will be done by every users
//         serialize = "NewPane",
//         serialize = "New-Pane",
//         serialize = "New_Pane",
//         serialize = "np"
//     )]
//     NewPane { path: String }, // TODO: add a way to call `new_tiled_pane_in_tab()`
//     /// Run a command in a new edit pane
//     #[cfg(not(test))]
//     #[strum(
//         props(Interface = "Pane"), // The cli does not know who wrote the command or who pressed `Enter` --> The edit will be done by every users
//     )]
//     Run(CommandToRun),
// }

// fn deserialize_action(action: &String, variant: impl EnumMessage) -> bool {
//     variant
//         .get_serializations()
//         .iter()
//         .map(|a| a.to_lowercase())
//         .collect::<Vec<_>>()
//         .contains(action)
// }

// impl ActionList {
//     fn parse(command: String, interface: &Interface) -> Self {
//         let mut split = command.split_whitespace();
//         let action = split.next().unwrap_or_default().to_lowercase();
//         let mut action_arguments = split.map(|v| v.to_owned());

//         match action.as_str() {
//             // Zellij actions
//             _ if deserialize_action(&action, ActionList::ClearScreen) => ActionList::ClearScreen,
//             _ if deserialize_action(&action, ActionList::CloseFocus) => ActionList::CloseFocus,
//             _ if deserialize_action(&action, ActionList::CloseFocusTab) => {
//                 ActionList::CloseFocusTab
//             }
//             _ if deserialize_action(
//                 &action,
//                 ActionList::ClosePluginPane {
//                     id: Default::default(), // TODO: Am I forced to defines every variable. Can’t I just `Default::default()`?                },
//                 },
//             ) =>
//             {
//                 let id = action_arguments
//                     .last()
//                     .map(|s| s.parse::<u32>().ok())
//                     .unwrap_or_default();

//                 ActionList::ClosePluginPane { id }
//             }
//             _ if deserialize_action(
//                 &action,
//                 ActionList::CloseTerminalPane {
//                     id: Default::default(), // TODO: Am I forced to defines every variable. Can’t I just `Default::default()`?                },
//                 },
//             ) =>
//             {
//                 let id = action_arguments
//                     .last()
//                     .map(|s| s.parse::<u32>().ok())
//                     .unwrap_or_default();

//                 ActionList::CloseTerminalPane { id }
//             }
//             _ if deserialize_action(&action, ActionList::DetachEveryone) => {
//                 ActionList::DetachEveryone
//             }
//             _ if deserialize_action(&action, ActionList::DetachMe) => ActionList::DetachMe,
//             _ if deserialize_action(&action, ActionList::DetachOthers) => ActionList::DetachOthers,
//             // _ if deserialize_action(&action, ActionList::EditScrollback) => {
//             //     ActionList::EditScrollback
//             // }
//             #[cfg(not(test))]
//             _ if deserialize_action(&action, ActionList::Edit(Default::default())) => {
//                 let last = action_arguments.next_back();
//                 let line_number = last.clone().unwrap_or(String::new()).parse::<usize>().ok();
//                 let mut path = action_arguments.collect::<Vec<String>>().join(" ");
//                 if line_number.is_none() {
//                     if !path.is_empty() {
//                         path.push(' ');
//                     }
//                     path.push_str(&last.unwrap_or_default())
//                 }

//                 ActionList::Edit(FileToOpen {
//                     path: path.into(),
//                     line_number,
//                     cwd: None, // TODO: get the cwd
//                 })
//             }
//             _ if deserialize_action(
//                 &action,
//                 ActionList::NewPane {
//                     path: Default::default(),
//                 },
//             ) =>
//             {
//                 ActionList::NewPane {
//                     path: action_arguments.collect::<Vec<String>>().join(" "),
//                 }
//             }
//             #[cfg(not(test))]
//             _ if deserialize_action(&action, ActionList::Run(Default::default())) => {
//                 let mut cmd = String::new();
//                 let mut args: Vec<String> = Default::default();
//                 let mut cwd = Default::default();

//                 let mut val = action_arguments.next();
//                 while val.is_some() {
//                     // Safety: checked in the while loop
//                     let v = unsafe { val.unwrap_unchecked() };

//                     match v.as_str() {
//                         "---cwd" => cwd = action_arguments.next().map(PathBuf::from),
//                         _ => {
//                             if cmd.is_empty() {
//                                 cmd = v;
//                             } else {
//                                 args.push(v);
//                             }
//                         }
//                     };
//                     val = action_arguments.next();
//                 }

//                 ActionList::Run(CommandToRun {
//                     path: cmd.into(),
//                     args,
//                     cwd,
//                 })
//             }

//             // Technicals
//             _ if deserialize_action(
//                 &action,
//                 ActionList::HelpAll {
//                     selection: Default::default(),
//                 },
//             ) =>
//             {
//                 let max = ActionList::filter_any().count();
//                 let selection = match interface {
//                     Interface::All | Interface::Pane => Selection::One { max, row: 0 },
//                     Interface::Pipe => Default::default(),
//                 };

//                 ActionList::HelpAll { selection }
//             }
//             _ if deserialize_action(
//                 &action,
//                 ActionList::HelpPane {
//                     selection: Default::default(),
//                 },
//             ) =>
//             {
//                 let max = ActionList::filter_pane().count();
//                 let selection = match interface {
//                     Interface::All | Interface::Pane => Selection::One { max, row: 0 },
//                     Interface::Pipe => Default::default(),
//                 };

//                 ActionList::HelpPane { selection }
//             }
//             _ if deserialize_action(
//                 &action,
//                 ActionList::HelpPipe {
//                     selection: Default::default(),
//                 },
//             ) =>
//             {
//                 let max = ActionList::filter_pipe().count();
//                 let selection = match interface {
//                     Interface::All | Interface::Pane => Selection::One { max, row: 0 },
//                     Interface::Pipe => Default::default(),
//                 };

//                 ActionList::HelpPipe { selection }
//             }

//             _ => ActionList::Unknown,
//         }
//     }

//     pub(crate) fn get_usable_interface(&self) -> Option<&str> {
//         self.get_str("Interface")
//     }

//     pub(crate) fn usable_in_any(&self) -> bool {
//         self.get_str("Interface").is_some()
//     }

//     pub(crate) fn usable_in_all(&self) -> bool {
//         self.get_str("Interface").is_some_and(|i| i == "All")
//     }

//     fn usable_in_pane(&self) -> bool {
//         self.get_str("Interface")
//             .is_some_and(|i| i == "All" || i == "Pane")
//     }

//     fn usable_in_pipe(&self) -> bool {
//         self.get_str("Interface")
//             .is_some_and(|i| i == "All" || i == "Pipe")
//     }

//     // TODO: remove this filters function since we extracted their filter?
//     pub(crate) fn filter_any() -> impl Iterator<Item = ActionList> {
//         ActionList::iter().filter(Self::usable_in_any)
//     }

//     pub(crate) fn filter_all() -> impl Iterator<Item = ActionList> {
//         ActionList::iter().filter(Self::usable_in_all)
//     }

//     pub(crate) fn filter_pane() -> impl Iterator<Item = ActionList> {
//         ActionList::iter().filter(Self::usable_in_pane)
//     }

//     pub(crate) fn filter_pipe() -> impl Iterator<Item = ActionList> {
//         ActionList::iter().filter(Self::usable_in_pipe)
//     }
// }

// TODO@Minimize: Is this intermediary struct useful?
// TODO@Simplify: Maybe make ParsedAction store `&str`?
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedAction {
    command: String,
    arguments: String,
}

/// ParsedAction cannot fail but if we don't have a value, we return `Err(())` as if `Option::None`
impl From<String> for ParsedAction {
    fn from(value: String) -> Self {
        match value.split_once(char::is_whitespace) {
            Some(p) => ParsedAction {
                command: String::from(p.0),
                arguments: String::from(p.1),
            },
            None => ParsedAction {
                command: value,
                arguments: String::from(""),
            },
        }
    }
}

/// Yeah I know, 2 time the same data once raw, once "parsed". But I don't want to try my hand at self referential structs in rust.
#[derive(Debug, Default)]
pub struct CAction {
    raw: String,
    builtin_factory: Option<&'static dyn BuiltinFactoryFull>,
    parsed: Option<ParsedAction>,
}

impl CAction {
    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn command(&self) -> Option<&str> {
        // TODO@Readability: can the match be removed?
        match self.parsed {
            Some(ref p) => Some(&p.command),
            None => None,
        }
    }

    fn arguments(&self) -> Option<&str> {
        // TODO@Readability: can the match be removed?
        match self.parsed {
            Some(ref p) => Some(&p.arguments),
            None => None,
        }
    }

    // Emulate a `String`
    pub fn push(&mut self, charactere: char, interface: &Interface) {
        self.raw.push(charactere);

        // TODO@Cache: Cache value and only re-compute builtin to use if the command change. Maybe still pass args dynamically
        *self = Self::from(self.raw.clone())
    }

    pub fn pop(&mut self, interface: &Interface) -> Option<char> {
        let res = self.raw.pop();

        // TODO@Cache: Cache value and only re-compute builtin to use if the command change. Maybe still pass args dynamically
        *self = Self::from(self.raw.clone());

        res
    }

    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear();
        self.parsed = None;
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    // TODO@Correctness: take ownership of self no?
    pub fn build_action<'a>(&self) -> Result<Box<dyn BuiltinFull>, &'a str> {
        dbg!(self.builtin_factory);
        match self.builtin_factory {
            Some(factory) => factory.try_from(self),
            None => Err(INVALID_ACTION_ERR_VALUE),
        }
    }
}

impl From<String> for CAction {
    fn from(value: String) -> Self {
        let parsed = ParsedAction::from(value.clone());
        CAction {
            raw: value,
            builtin_factory: find_builtin(&parsed),
            parsed: Some(parsed),
        }
    }
}

fn find_builtin(parsed: &ParsedAction) -> Option<&'static dyn BuiltinFactoryFull> {
    // TODO@Parser: Parse (at least split at first white element)

    let command = parsed.command.to_lowercase();

    for action in builtin::BUILTIN {
        if action
            .names()
            .iter()
            .find(|s| s.to_lowercase().as_str() == command)
            .is_some()
        {
            return Some(*action);
        }
    }

    None
}

// TODO: use self referential struct? So we do not store the data twice since `action` is computed from `command`
// #[derive(Debug, Clone)]
// pub struct Action {
//     command: String,
//     action: ActionList,
// }

// impl Action {
//     pub(crate) fn action(&self) -> &ActionList {
//         &self.action
//     }

//     pub(crate) fn command(&self) -> String {
//         self.command.clone()
//     }
// }

// impl Action {
//     fn parse_action(&mut self, interface: &Interface) {
//         // TODO: maybe don’t reparse all each time?
//         // if charactere.is_whitespace() {
//         let a = ActionList::parse(self.command.clone(), &interface);
//         if let ActionList::Unknown = a {
//             self.action = a;
//             return;
//         }

//         self.action = match &interface {
//             Interface::All => a,
//             Interface::Pane => {
//                 if a.usable_in_pane() {
//                     a
//                 } else {
//                     ActionList::Unavailable {
//                         action: Box::new(a),
//                         calling_interface: *interface,
//                     }
//                 }
//             }
//             Interface::Pipe => {
//                 if a.usable_in_pipe() {
//                     a
//                 } else {
//                     ActionList::Unavailable {
//                         action: Box::new(a),
//                         calling_interface: *interface,
//                     }
//                 }
//             }
//         }
//         // }
//     }

//     pub(crate) fn set(&mut self, command: &str, interface: &Interface) {
//         self.command = command.to_string();
//         self.parse_action(interface);
//     }

//     // Emulate a `String`
//     pub(crate) fn push(&mut self, charactere: char, interface: &Interface) {
//         self.command.push(charactere);
//         self.parse_action(interface);
//     }
//     pub(crate) fn pop(&mut self, interface: &Interface) -> Option<char> {
//         let res = self.command.pop();
//         self.parse_action(interface);
//         res
//     }
//     pub(crate) fn clear(&mut self) {
//         self.command.clear();
//         // We clear the action so don’t care from which interface. Improvise some value instead of requesting one from the caller.
//         self.parse_action(&Interface::All);
//     }
//     pub(crate) fn len(&self) -> usize {
//         self.command.len()
//     }
//     pub(crate) fn as_str(&self) -> &str {
//         &self.command
//     }

//     pub(crate) fn selection_up(&mut self) {
//         match &mut self.action {
//             ActionList::HelpAll { selection }
//             | ActionList::HelpPane { selection }
//             | ActionList::HelpPipe { selection } => match selection {
//                 Selection::One { row, max } => {
//                     if *row != 0 {
//                         *row -= 1;
//                     } else {
//                         *row = *max - 1
//                     }
//                 }
//                 Selection::Expand => {}
//             },
//             _ => {}
//         }
//     }

//     pub(crate) fn selection_down(&mut self) {
//         match &mut self.action {
//             ActionList::HelpAll { selection }
//             | ActionList::HelpPane { selection }
//             | ActionList::HelpPipe { selection } => match selection {
//                 Selection::One { row, max } => *row = (*row + 1) % *max,
//                 Selection::Expand => {}
//             },
//             _ => {}
//         }
//     }
// }

// impl Default for Action {
//     fn default() -> Self {
//         Action {
//             command: String::new(),
//             action: ActionList::Unknown,
//         }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;
    use crate::action::builtin::*;

    #[test]
    fn parse_action() {
        assert_eq!(
            ParsedAction::from(String::from("")),
            ParsedAction {
                command: String::from(""),
                arguments: String::from("")
            }
        );
        assert_eq!(
            ParsedAction::from(String::from("Plop")),
            ParsedAction {
                command: String::from("Plop"),
                arguments: String::from("")
            }
        );
    }

    #[test]
    fn find_builtin_no_argument() {
        let parsed = ParsedAction::from(String::from("h"));
        let h = find_builtin(&parsed);

        assert_eq!(
            format!("{:?}", h.unwrap()),
            format!("{:?}", help::HelpFactory {})
        )
    }

    #[test]
    fn find_builtin_with_argument() {
        let parsed = ParsedAction::from(String::from("ClosePane terminal 1"));
        let cp = find_builtin(&parsed);

        assert_eq!(
            format!("{:?}", cp.unwrap()),
            format!("{:?}", close_panes::ClosePanesFactory)
        )
    }

    #[test]
    fn factory_to_concrete_type() {
        let cmd = CAction::from(String::from("h"));
        assert_eq!(
            format!("{:?}", cmd.build_action().unwrap()),
            format!(
                "{:?}",
                help::Help {
                    interface: Interface::All
                }
            )
        );
    }
}
