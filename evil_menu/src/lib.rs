#![no_std]

mod builder;

use builder::{AttackBuilder, HighLevelAttackCmd};

use embedded_can::Id;
use embedded_io::{Read, Write};
use evil_core::{clock::TicksClock, tranceiver::Tranceiver, CanBitrates, EvilCore};
use menu::{argument_finder, Item, ItemType, Menu, Parameter, Runner};
use noline::builder::EditorBuilder;

struct Context<CLK, TR>
where
    CLK: TicksClock,
    TR: Tranceiver,
{
    attack_builder: AttackBuilder,
    core: EvilCore<CLK, TR>,
}

impl<CLK, TR> Context<CLK, TR>
where
    CLK: TicksClock,
    TR: Tranceiver,
{
    fn new_with(core: EvilCore<CLK, TR>) -> Self {
        Self {
            attack_builder: AttackBuilder::new(),
            core,
        }
    }
}

pub struct EvilMenu<'a, SERIAL, CLK, TR>
where
    SERIAL: Read + Write,
    CLK: TicksClock,
    TR: Tranceiver,
{
    serial: Option<SERIAL>,
    menu: Option<Menu<'a, SERIAL, Context<CLK, TR>>>,
    context: Option<Context<CLK, TR>>,
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
                        function: cmd_set_baudrate,
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
                    item_type: ItemType::Menu(&Menu {
                                    label: "custom_attack",
                                    items: &[
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: match_id,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "id",
                                                        help: Some("CAN ID to match in hex (e.g, 0x123))"),
                                                    },
                                                    Parameter::Named {
                                                        parameter_name: "extended",
                                                        help: Some("Whether this is an extended ID (defaults to standard ID)"),
                                                    },
                                                ],
                                            },
                                            command: "match_id",
                                            help: Some("Add a CAN frame Id match condition to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: match_data,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "dlc",
                                                        help: Some("Data length code (0 to 8)"),
                                                    },
                                                    Parameter::Optional {
                                                        parameter_name: "data",
                                                        help: Some("Optional data bytes as comma-separated hex values (e.g., 0x10,0x20,0x30)"),
                                                    },
                                                ],
                                            },
                                            command: "match_data",
                                            help: Some("Add a CAN frame data match condition to the attack. If dlc > len(data) will match data partially (e.g., dlc = 3 and data 0x10,0x20 will match frames whith data starting 0x10,0x20 and any value for the 3rd byte."),
                                        },
                                    ],
                                    entry: Some(enter_custom_attack),
                                    exit: Some(exit_custom_attack)}),
                                command: "custom_attack",
                                help: Some("Build a custom attack"),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: test_attack,
                        parameters: &[],
                    },
                    command: "test_attack",
                    help: Some("Choose the test attack"),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: arm,
                        parameters: &[],
                    },
                    command: "arm",
                    help: Some("Load the attack and arm the device"),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: attack,
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
            context: Some(Context::new_with(core)),
        }
    }

    pub fn run(&'a mut self) {
        let mut buffer = [0; 100];
        let mut history = [0; 200];
        let mut editor = EditorBuilder::from_slice(&mut buffer)
            .with_slice_history(&mut history)
            .build_sync(self.serial.as_mut().unwrap())
            .unwrap();

        let mut runner = Runner::new(
            self.menu.take().unwrap(),
            &mut editor,
            self.serial.take().unwrap(),
            self.context.as_mut().unwrap(),
        );

        while let Ok(_) = runner.input_line(self.context.as_mut().unwrap()) {}
    }
}

// Define callback functions
fn cmd_set_baudrate<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
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

fn enter_custom_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    interface: &mut I,
    _context: &mut Context<C, T>,
) {
    writeln!(interface, "In enter_custom_attack").unwrap();
    todo!()
}

fn exit_custom_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    interface: &mut I,
    _context: &mut Context<C, T>,
) {
    writeln!(interface, "In exit_custom_attack").unwrap();
    todo!()
}

fn test_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "Test attack").unwrap();
    context.attack_builder.set_test_attack();
}

fn match_id<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let id_str = argument_finder(item, args, "id").unwrap();
    let is_extended = match argument_finder(item, args, "extended").unwrap() {
        Some(_) => true,
        None => false,
    };

    if let Some(mut id_str) = id_str {
        // Parse id as u32
        id_str = id_str.trim_start_matches("0x");
        if let Ok(id_val) = u32::from_str_radix(id_str, 16) {
            // Default to standard ID unless specified as extended

            let id = if is_extended {
                Id::Extended(embedded_can::ExtendedId::new(id_val).unwrap())
            } else {
                Id::Standard(embedded_can::StandardId::new(id_val as u16).unwrap())
            };

            // Add the match command
            context
                .attack_builder
                .push_high_level_attack_cmd(HighLevelAttackCmd::MatchId { id });

            writeln!(interface, "Added Match Id command with Id: {:?}", id).unwrap();
        } else {
            writeln!(interface, "Invalid ID format").unwrap();
        }
    } else {
        writeln!(interface, "ID is required").unwrap();
    }
}

fn match_data<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let dlc_str = argument_finder(item, args, "dlc").unwrap();
    let data_str = argument_finder(item, args, "data").unwrap();

    if let Some(dlc_str) = dlc_str {
        // Parse id as u32
        if let Ok(data_len) = str::parse(dlc_str) {
            // Parse data if provided
            let data = match data_str {
                Some(s) => {
                    let mut data_array = [0u8; 8];

                    for (i, hex_str) in s.split(',').enumerate() {
                        if i >= 8 {
                            writeln!(interface, "Error: Data exceeds maximum length of 8 bytes")
                                .unwrap();
                            return;
                        }

                        let trimmed = hex_str.trim().trim_start_matches("0x");
                        match u8::from_str_radix(trimmed, 16) {
                            Ok(value) => {
                                data_array[i] = value;
                            }
                            Err(_) => {
                                writeln!(interface, "Error parsing hex data").unwrap();
                                return;
                            }
                        }
                    }

                    Some(data_array)
                }
                None => None,
            };

            // Add the match command
            context
                .attack_builder
                .push_high_level_attack_cmd(HighLevelAttackCmd::MatchData { data_len, data });

            writeln!(
                interface,
                "Added match data command with data: {:?} with len {}",
                data, data_len
            )
            .unwrap();
        } else {
            writeln!(interface, "Invalid ID format").unwrap();
        }
    } else {
        writeln!(interface, "ID is required").unwrap();
    }
}

fn arm<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "Arming the device").unwrap();
    context.core.arm(&context.attack_builder.build()).unwrap();
}

fn attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "Launching attack").unwrap();
    context.core.board_specific_attack();
}
