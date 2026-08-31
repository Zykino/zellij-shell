use std::fmt::Display;

use zellij_tile::shim::{self, NestedListItem};

use crate::action::{
    self,
    builtin::{
        self, usable_in_all, Builtin, BuiltinFactory, BuiltinFactoryFull, BuiltinFull, BUILTIN,
    },
    CAction, Interface,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Help {
    pub interface: Interface,
}

impl Help {
    pub const fn default() -> Self {
        Self {
            interface: Interface::All,
        }
    }
}

impl Builtin for Help {
    fn execute(&self) {
        todo!()
    }
}

impl Display for Help {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let factory_list: Box<dyn Iterator<Item = &&dyn BuiltinFactoryFull> + '_> =
            match self.interface {
                Interface::All => Box::new(BUILTIN.iter()),
                Interface::Pane => Box::new(
                    BUILTIN
                        .iter()
                        .filter(|factory| builtin::usable_in_pane(**factory)),
                ),
                Interface::Pipe => Box::new(
                    BUILTIN
                        .iter()
                        .filter(|factory| builtin::usable_in_pipe(**factory)),
                ),
            };

        let nested_list =
            factory_list
                .enumerate()
                .flat_map(|(i, factory)| -> Vec<NestedListItem> {
                    let name = factory.names();
                    let shortcut_msg = "Shortcut variations";
                    let interface_msg = "Interface restriction";

                    let mut result = Vec::with_capacity(3);
                    result.push(
                        NestedListItem::new(format!("{}:\t{}", name[0], factory.description()))
                            .color_range(1, 0..name.len()),
                    );

                    // TODO2: Populate a Selection struct
                    // // TODO: || Interface::Pipe { // --> Expand all when asking for Help from the CLI.
                    // let (show, select) = match selection {
                    //     Selection::One { row, max: _ } => (&i == row, &i == row),
                    //     Selection::Expand => (true, false),
                    // };

                    // if show{
                    if true {
                        result.push(
                            NestedListItem::new(format!(
                                "{}:\t{}",
                                shortcut_msg,
                                factory.names().join(", ")
                            ))
                            .indent(1)
                            .color_substring(2, shortcut_msg), // TODO@Colors: Rationalize indexes (either use only the const so I define a color for each thing, or go purely by order)
                        );

                        // TODO@Optimisation: inside `usable_in_all`, we do a `.interface()`, and we re-do one just after to print...
                        if !usable_in_all(*factory) {
                            result.push(
                                NestedListItem::new(format!(
                                    "{}:\t{:?}",
                                    interface_msg,
                                    factory.interface()
                                ))
                                .indent(1)
                                .color_substring(0, interface_msg), // TODO@Colors: Rationalize indexes (either use only the const so I define a color for each thing, or go purely by order)
                            );
                        }

                        // TODO@Pickers: Show the picker that may be used to the user? Just the picker names or find a way to show if one is configured or not? (and how/which one fullfill this requirement?)
                    }

                    // if select {
                    if true {
                        result.iter().map(|item| item.clone().selected()).collect()
                    } else {
                        result.iter().map(|item| item.to_owned()).collect()
                    }
                });

        write!(f, "{}", shim::serialize_nested_list(nested_list))

        //     // BUILTIN;
        //     let docs: Box<dyn Iterator<Item = ActionList>> = match self {
        //         Self::HelpAll { .. } => Box::new(ActionList::filter_any()),
        //         Self::HelpPane { .. } => Box::new(ActionList::filter_pane()),
        //         Self::HelpPipe { .. } => Box::new(ActionList::filter_pipe()),
        //         _ => panic!("Should be one of the help modes"),
        //     };
        //     let text: Vec<_> = docs
        //                 .enumerate()
        //                 .flat_map(|(i, variant)| -> Vec<NestedListItem> {
        //                     let name = variant
        //                         .get_serializations()
        //                         .first()
        //                         .expect("At least one serialization is garanteed");
        //                     let shortcut_msg = "Shortcut variations";
        //                     let interface_msg = "Interface restriction";

        //                     let mut result = Vec::with_capacity(3);
        //                     result.push(
        //                         NestedListItem::new(format!(
        //                             "{}:\t{}",
        //                             name,
        //                             variant
        //                                 .get_documentation()
        //                                 .expect("{variant:?} should have a line of documentation")
        //                         ))
        //                         .color_range(1, 0..name.len()),
        //                     );

        //                     // TODO: || Interface::Pipe { // --> Expand all when asking for Help from the CLI.
        //                     let (show, select) = match selection {
        //                         Selection::One { row, max: _ } => (&i == row, &i == row ),
        //                         Selection::Expand => (true, false),
        //                     };

        //                     if show {
        //                         result.push(
        //                             NestedListItem::new(format!(
        //                                 "{}:\t{}",
        //                                 shortcut_msg,
        //                                 variant.get_serializations().join(", ")
        //                             ))
        //                                 .indent(1)
        //                                 .color_range(2, 0..shortcut_msg.len()),
        //                         );

        //                         if !variant.usable_in_all() {
        //                             result.push(
        //                                 NestedListItem::new(format!(
        //                                     "{}:\t{}",
        //                                     interface_msg,
        //                                     variant.get_usable_interface().expect("Interface restriction should be set for command we show help on")
        //                                 ))
        //                                     .indent(1)
        //                                     .color_range(0, 0..interface_msg.len()),
        //                             );
        //                         }
        //                     }

        //                     if select {
        //                         result.iter().map(|item| item.clone().selected()).collect()
        //                     } else {
        //                         result.iter().map(|item| item.to_owned()).collect()
        //                     }
        //                 })
        //                 .collect();

        //     write!(f, "{}", shim::serialize_nested_list(&text))
    }
}

#[derive(Debug)]
pub struct HelpFactory;

impl BuiltinFactory for HelpFactory {
    fn names(&self) -> &[&str] {
        &[
            "?", "H", "Help", "HelpAll", "HelpAny", "HelpPane", "HelpPipe",
        ]
    }

    fn description(&self) -> &str {
        "Show the list of commands available. Can be filtered through their interface's availability"
    }

    fn interface(&self) -> Interface {
        Interface::All
    }

    fn try_from(&self, action: &CAction) -> Result<std::boxed::Box<dyn BuiltinFull>, &str> {
        let h = match action
            .command()
            .expect("The command should be set when calling `try_from` (Help)")
            .to_lowercase()
            .as_str()
        {
            "?" | "h" | "help" => match action
                .arguments()
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "pane" => Help {
                    interface: Interface::Pane,
                },
                "pipe" => Help {
                    interface: Interface::Pipe,
                },
                "any" | "all" => Help {
                    interface: Interface::All,
                },
                _ => Help {
                    interface: Interface::All,
                },
                // _ => todo!("Return an error"),
            },
            "helpall" | "helpany" => Help {
                interface: Interface::All,
            },
            "helppane" => Help {
                interface: Interface::Pane,
            },
            "helppipe" => Help {
                interface: Interface::Pipe,
            },
            v => unreachable!(
                "called `Help::try_from`, on a value that is not advertized in `Help::names`: {}",
                v
            ),
        };

        Ok(Box::new(h))
    }
}
