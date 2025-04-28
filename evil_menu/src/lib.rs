#![no_std]

use core::usize::MAX;

use embedded_io::{Read, Write};
use evil_core::{clock::TicksClock, tranceiver::Tranceiver, EvilCore};
use evil_core::{AttackCmd, CanBitrates, FastBitQueue, MAX_ATTACK_SIZE};
use menu::{argument_finder, Item, ItemType, Menu, Parameter, Runner};
use noline::builder::EditorBuilder;

struct AttackBuilder {
    attack: [AttackCmd; MAX_ATTACK_SIZE],
    length: usize,
}

impl Default for AttackBuilder {
    fn default() -> Self {
        Self {
            attack: [AttackCmd::None; MAX_ATTACK_SIZE],
            length: MAX_ATTACK_SIZE,
        }
    }
}

impl AttackBuilder {
    fn set_default_attack(&mut self) {
        self.attack = [AttackCmd::None; MAX_ATTACK_SIZE];
        self.attack[0] = AttackCmd::Wait { bits: 1 };
        self.attack[1] = AttackCmd::Force {
            stream: FastBitQueue::new(0b1010_101, 7),
        };
        self.length = 2;
    }
}

struct Context<CLK, TR>
where
    CLK: TicksClock,
    TR: Tranceiver,
{
    attack_builder: AttackBuilder,
    core: EvilCore<CLK, TR>,
}

pub struct EvilMenu<'a, SERIAL, CLK, TR>
where
    SERIAL: Read + Write,
    CLK: TicksClock,
    TR: Tranceiver,
{
    serial: Option<SERIAL>,
    menu: Option<Menu<'a, SERIAL, Context<CLK, TR>>>,
    context: Context<CLK, TR>,
}

impl<'a, SERIAL, CLK, TR> EvilMenu<'a, SERIAL, CLK, TR>
where
    SERIAL: Read + Write,
    CLK: TicksClock,
    TR: Tranceiver,
{
    pub fn new(serial: SERIAL, core: EvilCore<CLK, TR>) -> Self {
        let menu = Menu {
            label: "root",
            items: &[
                &Item {
                    item_type: ItemType::Callback {
                        function: EvilMenu::<SERIAL, CLK, TR>::cmd_set_baudrate,
                        parameters: &[Parameter::Mandatory {
                            parameter_name: "baudrate",
                            help: Some(
                                "In kbps. Valid baudrates are 5, 10, 20, 50, 100, 125, 250 and 500",
                            ),
                        }],
                    },
                    command: "set_baudrate",
                    help: Some("Set the baudrate of the adapter"),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: EvilMenu::<SERIAL, CLK, TR>::default_attack,
                        parameters: &[],
                    },
                    command: "default_attack",
                    help: Some("Choose the default attack"),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: EvilMenu::<SERIAL, CLK, TR>::arm,
                        parameters: &[],
                    },
                    command: "arm",
                    help: Some("Load the attack and arm the device"),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: EvilMenu::<SERIAL, CLK, TR>::attack,
                        parameters: &[],
                    },
                    command: "attack",
                    help: Some("Start the attack"),
                },
            ],
            entry: None,
            exit: None,
        };

        EvilMenu {
            serial: Some(serial),
            menu: Some(menu),
            context: Context {
                attack_builder: AttackBuilder::default(),
                core,
            },
        }
    }

    pub fn run(&'a mut self) {
        let mut buffer = [0; 100];
        let mut history = [0; 200];
        let mut editor = EditorBuilder::from_slice(&mut buffer)
            .with_slice_history(&mut history)
            .build_sync(&mut self.serial.as_mut().unwrap())
            .unwrap();

        let mut r = Runner::new(
            self.menu.take().unwrap(),
            &mut editor,
            self.serial.take().unwrap(),
            &mut self.context,
        );
        while let Ok(_) = r.input_line(&mut self.context) {}
    }

    // Define callback functions
    fn cmd_set_baudrate(
        _menu: &Menu<SERIAL, Context<CLK, TR>>,
        item: &Item<SERIAL, Context<CLK, TR>>,
        args: &[&str],
        interface: &mut SERIAL,
        context: &mut Context<CLK, TR>,
    ) {
        match argument_finder(item, args, "baudrate").unwrap() {
            Some(baudrate_str) => {
                // Set the baudrate of the adapter
                let baudrate = CanBitrates::from(baudrate_str.parse::<u16>().unwrap());
                if [
                    CanBitrates::Kbps50,
                    CanBitrates::Kbps100,
                    CanBitrates::Kbps125,
                    CanBitrates::Kbps250,
                    CanBitrates::Kbps500,
                ]
                .contains(&baudrate)
                {
                    writeln!(interface, "Baudrate set to {:?}", baudrate).unwrap();
                    context.core.set_baudrate(baudrate);
                } else {
                    writeln!(interface, "Invalid baudrate").unwrap();
                }
            }
            None => {
                // Handle error case
                writeln!(interface, "Invalid baudrate").unwrap();
            }
        };
    }

    fn default_attack(
        _menu: &Menu<SERIAL, Context<CLK, TR>>,
        _item: &Item<SERIAL, Context<CLK, TR>>,
        _args: &[&str],
        interface: &mut SERIAL,
        context: &mut Context<CLK, TR>,
    ) {
        writeln!(interface, "Default attack").unwrap();
        context.attack_builder.set_default_attack();
    }

    fn arm(
        _menu: &Menu<SERIAL, Context<CLK, TR>>,
        item: &Item<SERIAL, Context<CLK, TR>>,
        args: &[&str],
        interface: &mut SERIAL,
        context: &mut Context<CLK, TR>,
    ) {
        writeln!(interface, "Arming the device").unwrap();
        context
            .core
            .arm(&context.attack_builder.attack[..context.attack_builder.length])
            .unwrap();
    }

    fn attack(
        _menu: &Menu<SERIAL, Context<CLK, TR>>,
        item: &Item<SERIAL, Context<CLK, TR>>,
        args: &[&str],
        interface: &mut SERIAL,
        context: &mut Context<CLK, TR>,
    ) {
        writeln!(interface, "Launching attack").unwrap();
        context.core.board_specific_attack();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_1() {
//         assert_eq!(1, 1);
//     }
// }
