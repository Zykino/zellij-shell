pub mod action;
pub mod ui;

use crate::action::{
    builtin::{BuiltinFactoryFull, BuiltinFull},
    CAction,
};

const OWN_URL: &str = "zellij:OWN_URL";

pub fn execute_action(command: Box<dyn BuiltinFull>) {
    command.execute();
}
