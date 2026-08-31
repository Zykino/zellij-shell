use std::fmt::Display;

use crate::{action::Interface, ui::colors};

use zellij_tile::shim;

use crate::action::{
    builtin::{Builtin, BuiltinFactory, BuiltinFull, Pickers},
    CAction,
};

// TODO@Errors: Do not use `&str`
const TRY_FROM_ERR_VALUE: &str =
    "Targeted pane should be written as `terminal_<id>` or `plugin_<id>`";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PaneType {
    Plugin(u32),
    Terminal(u32),
}

// TODO@Minimize: transform this to manage all close actions at once?
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosePanes {
    pub id: Vec<PaneType>,
}

impl ClosePanes {
    pub const fn default() -> Self {
        Self { id: vec![] }
    }

    pub fn new(pane: PaneType) -> Self {
        ClosePanes { id: vec![pane] }
    }

    pub fn new_terminal(id: u32) -> Self {
        ClosePanes {
            id: vec![PaneType::Terminal(id)],
        }
    }

    pub fn new_plugin(id: u32) -> Self {
        ClosePanes {
            id: vec![PaneType::Plugin(id)],
        }
    }
}

impl Builtin for ClosePanes {
    fn execute(&self) {
        for pane in &self.id {
            match pane {
                PaneType::Terminal(id) => shim::close_terminal_pane(*id),
                PaneType::Plugin(id) => shim::close_plugin_pane(*id),
            }
        }
    }

    fn pickers(&self) -> Vec<Pickers> {
        vec![Pickers::Pane]
    }
}

impl TryFrom<&str> for ClosePanes {
    // TODO@Errors: Use miette to explain where the issue come from?
    type Error = &'static str;

    // TODO@Usability: Default to self? Find a way to easily expose own pane?
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut panes_type = vec![];
        for spaced_values in value.split_whitespace() {
            let data = spaced_values
                .split_once('_')
                // TODO@Errors: return correct errors instead of `&str`
                .ok_or(TRY_FROM_ERR_VALUE)?;
            match data.0 {
                "terminal" => panes_type.push(PaneType::Terminal(
                    data.1.parse::<u32>().map_err(|_| TRY_FROM_ERR_VALUE)?,
                )),
                "plugin" => panes_type.push(PaneType::Plugin(
                    data.1.parse::<u32>().map_err(|_| TRY_FROM_ERR_VALUE)?,
                )),
                // TODO@Errors: return correct errors instead of `&str`
                _ => return Err(TRY_FROM_ERR_VALUE),
            };
        }

        Ok(ClosePanes { id: panes_type })
    }
}

impl Display for ClosePanes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut plugins = vec![];
        let mut terminals = vec![];

        for pane in &self.id {
            match pane {
                PaneType::Plugin(id) => {
                    plugins.push(id);
                }
                PaneType::Terminal(id) => {
                    terminals.push(id);
                }
            }
        }

        write!(
            f,
            "Close panes\n{} {:?}\n{} {:?}",
            shim::serialize_text(
                &shim::Text::new("PLUGIN:").color_substring(colors::AT_LEAST_ONE, "PLUGIN")
            ),
            plugins,
            shim::serialize_text(
                &shim::Text::new("TERMINAL:").color_substring(colors::AT_LEAST_ONE, "TERMINAL")
            ),
            terminals // id.unwrap_or_default() // TODO: not default when unset…
        )
    }
}

#[derive(Debug)]
pub struct ClosePanesFactory;

impl BuiltinFactory for ClosePanesFactory {
    fn names(&self) -> &[&str] {
        &["Close", "ClosePane"]
    }

    fn description(&self) -> &str {
        "Close some panes"
    }

    fn interface(&self) -> Interface {
        Interface::All
    }

    fn try_from(&self, action: &CAction) -> Result<std::boxed::Box<dyn BuiltinFull>, &str> {
        // TODO@Minimize: check `action.command` to create the correct action
        Ok(Box::new(ClosePanes::try_from(
            action.arguments().ok_or(TRY_FROM_ERR_VALUE)?,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::builtin;

    // TODO@Test: Choose to either parse from string (with intermediary struct) or directly the `try_from` from this component
    #[test]
    fn parse() {
        let ca = CAction::from(String::from("ClosePanes terminal_1"));

        assert_eq!(
            format!(
                "{:?}",
                builtin::BuiltinFactory::try_from(&ClosePanesFactory, &ca).unwrap()
            ),
            format!(
                "{:?}",
                (ClosePanes {
                    id: vec![PaneType::Terminal(1)]
                })
            )
        );

        assert_eq!(
            ClosePanes::try_from("plugin_2"),
            Ok(ClosePanes {
                id: vec![PaneType::Plugin(2)]
            })
        );
    }

    #[test]
    fn multiple_parse() {
        assert_eq!(
            ClosePanes::try_from("terminal_1 plugin_2"),
            Ok(ClosePanes {
                id: vec![PaneType::Terminal(1), PaneType::Plugin(2)]
            })
        );
        assert_eq!(
            ClosePanes::try_from("plugin_3 terminal_4 terminal_5"),
            Ok(ClosePanes {
                id: vec![
                    PaneType::Plugin(3),
                    PaneType::Terminal(4),
                    PaneType::Terminal(5)
                ]
            })
        );
    }

    #[test]
    fn multidigit_id() {
        assert_eq!(
            ClosePanes::try_from("terminal_42 plugin_1337"),
            Ok(ClosePanes {
                id: vec![PaneType::Terminal(42), PaneType::Plugin(1337),]
            })
        );
    }

    #[test]
    fn fail_parse_pane_type() {
        // assert_eq!(ClosePanes::try_from(""), Err(""));
        assert_eq!(ClosePanes::try_from("plop_1"), Err(TRY_FROM_ERR_VALUE));
    }

    #[test]
    fn fail_parse_id() {
        // assert_eq!(ClosePanes::try_from(""), Err(""));
        assert_eq!(ClosePanes::try_from("plugin_Plop"), Err(TRY_FROM_ERR_VALUE));
    }
}
