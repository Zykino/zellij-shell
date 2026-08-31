use std::fmt::{Debug, Display};

use crate::action::{CAction, Interface};

// TODO@Include: Is it possible to "include all file in the directory"? (like `mod *`)
pub mod close_panes;
pub mod help;

pub const BUILTIN: &[&dyn BuiltinFactoryFull] =
    &[&close_panes::ClosePanesFactory, &help::HelpFactory];

fn usable_in_all(factory: &dyn BuiltinFactoryFull) -> bool {
    factory.interface() == Interface::All
}

fn usable_in_pane(factory: &dyn BuiltinFactoryFull) -> bool {
    factory.interface() == Interface::Pane || factory.interface() == Interface::All
}

fn usable_in_pipe(factory: &dyn BuiltinFactoryFull) -> bool {
    factory.interface() == Interface::Pipe || factory.interface() == Interface::All
}

// TODO@Readability: Move close to the Interface enum?
pub enum Pickers {
    Pane,
    Session,
    Tab,
}

// TODO@Types: Force implementation of `TryFrom((&str, &str))`? (tuple of name/rest of line) a "better" struct?
/// The trait to implement for each new component. It is a composit trait consisting of the custom [`Builtin`] one and some default others
pub trait BuiltinFull: Builtin + Debug + Display {}
impl<T> BuiltinFull for T where T: Builtin + Debug + Display {}
pub trait BuiltinFactoryFull: BuiltinFactory + Debug {}
impl<T> BuiltinFactoryFull for T where T: BuiltinFactory + Debug {}

pub trait Builtin {
    fn execute(&self);

    fn pickers(&self) -> Vec<Pickers> {
        vec![]
    }
}

pub trait BuiltinFactory {
    fn names(&self) -> &[&str];
    fn description(&self) -> &str;
    fn interface(&self) -> Interface;
    fn try_from(&self, action: &CAction) -> Result<std::boxed::Box<dyn BuiltinFull>, &str>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbtest::arbtest;
    use exhaustigen::Gen;
    use std::{collections::HashSet, str::FromStr};

    // Not sure this is really useful…
    #[test]
    fn names_are_sorted_individually() {
        for c in BUILTIN {
            assert!(c.names().is_sorted())
        }
    }

    #[test]
    fn names_are_uniques_over_all_components() {
        let mut m = HashSet::new();
        for c in BUILTIN {
            for n in c.names() {
                assert!(m.insert(n));
            }
        }
    }

    #[test]
    fn all_name_no_arg_buildable_without_panic() {
        dbg!("In case of error, the last (factory, name) pair printed is the one we cannot build without argument (should return an error instead of panicing)");

        // TODO@Readability: choose a form, here we test 3 time the same things with different iterator/loop style. Maybe change the other test too to reflect the choice.
        for factory in BUILTIN {
            for name in factory.names() {
                dbg!(factory, name);
                let _ = factory.try_from(&CAction::from(name.to_string()));
            }
        }

        BUILTIN.iter().for_each(|factory| {
            factory.names().iter().for_each(|name| {
                dbg!(factory, name);
                let _ = factory.try_from(&CAction::from(name.to_string()));
            });
        });

        BUILTIN
            .iter()
            .flat_map(|factory| std::iter::repeat(factory).zip(factory.names()))
            .for_each(|(factory, name)| {
                dbg!(factory, name);
                let _ = factory.try_from(&CAction::from(name.to_string()));
            });
    }

    #[test]
    fn arb_all_name_random_arg_buildable_without_panic() {
        arbtest(|u| {
            let b_index = u.int_in_range(0..=BUILTIN.len() - 1)?;
            let n_index = u.int_in_range(0..=BUILTIN[b_index].names().len() - 1)?;

            let action = CAction::from(BUILTIN[b_index].names()[n_index].to_string() + " " + u.arbitrary()?);

            dbg!(&action);

            // Can result in error as long as it does not crash
            let _ = BUILTIN[b_index].try_from(&action);

            // No crash
            Ok(())
        })
        // .seed(0x979d27f700010000)
        // Keep semicolon on its own line. That way we can easily prepend `.seed(…)` to fix issues as they come
        ;
    }

    // Well wanted to try exhausting all permutation on (more or less) ascii for only 1 builder 1st name it is already way too long (more than a couple of minutes). Even by parallellizing it would be way too much to exhaukstively check for no crash on every builder's (even worse 1 for each name).
    // #[test]
    // fn exhaust_all_name_random_arg_buildable_without_panic() {
    //     const MAX_USEFUL_MESSAGE_LEN: usize = "terminal_10".len(); // ClosePane's longest string, with multidigit to parse
    //     const EVERY_CHARS: &[char] = &[
    //         'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
    //         'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
    //         'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y',
    //         'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', '*', '&', '^', '<',
    //         '>', '$', '{', '(', ')', '}', '~', '[', ']', '_', '+', '-', '/', '\"', '\\', '!', ';',
    //         ':', '?', '|', '@', '%', '=', '#',
    //     ];
    //     let mut perms = 0;

    //     let mut g = Gen::new();
    //     while !g.done() {
    //         perms += 1;
    //         let b_index = 0; //= u.int_in_range(0..=BUILTIN.len() - 1)?;
    //         let n_index = 0; //= u.int_in_range(0..=BUILTIN[b_index].names().len() - 1)?;

    //         let qco = g
    //             .gen_bound_comb(MAX_USEFUL_MESSAGE_LEN, EVERY_CHARS)
    //             .collect();
    //         // dbg!(&qco);

    //         let action = CAction {
    //             command: BUILTIN[b_index].names()[n_index].to_string(),
    //             action: qco,
    //         };

    //         // Can result in error as long as it does not crash
    //         let _ = BUILTIN[b_index].try_from(&action);

    //         // No crash
    //         // Ok(())
    //     }
    //     dbg!(perms);
    // }
}
