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
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: skip_data,
                                                parameters: &[],
                                            },
                                            command: "skip_data",
                                            help: Some("Add a skip data command to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: wait,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "bits",
                                                        help: Some("Number of bits to wait"),
                                                    },
                                                ],
                                            },
                                            command: "wait",
                                            help: Some("Add a wait command to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: send_error,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "count",
                                                        help: Some("Number of error frames to send"),
                                                    },
                                                ],
                                            },
                                            command: "send_error",
                                            help: Some("Add a send error command to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: send_raw,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "bits",
                                                        help: Some("Bits to send (e.g, 11010101))"),
                                                    },
                                                    Parameter::Named {
                                                        parameter_name: "force",
                                                        help: Some("Whether to force this bits"),
                                                    },
                                                ],
                                            },
                                            command: "send_raw",
                                            help: Some("Add a send raw data command to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: wait_eof,
                                                parameters: &[],
                                            },
                                            command: "wait_eof",
                                            help: Some("Add a wait EOF command to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: send_msg,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "id",
                                                        help: Some("CAN ID to send in hex (e.g, 0x123)"),
                                                    },
                                                    Parameter::Named {
                                                        parameter_name: "extended",
                                                        help: Some("Whether this is an extended ID (defaults to standard ID)"),
                                                    },
                                                    Parameter::Named {
                                                        parameter_name: "rtr",
                                                        help: Some("Set frame RTR bit"),
                                                    },
                                                    Parameter::Named {
                                                        parameter_name: "force",
                                                        help: Some("Whether to force this bits"),
                                                    },
                                                    Parameter::Optional {
                                                        parameter_name: "data",
                                                        help: Some("Data bytes as comma-separated hex values (e.g., 0x10,0x20,0x30)"),
                                                    },
                                                ],
                                            },
                                            command: "send_msg",
                                            help: Some("Add a send message command to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: send_overload_frame,
                                                parameters: &[],
                                            },
                                            command: "send_overload_frame",
                                            help: Some("Add a send overload frame command to the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: delete,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "idx",
                                                        help: Some("Index of the command to delete"),
                                                    },
                                                ],
                                            },
                                            command: "delete",
                                            help: Some("Delete a command from the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: relocate,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "from",
                                                        help: Some("Source index of the command"),
                                                    },
                                                    Parameter::Mandatory {
                                                        parameter_name: "to",
                                                        help: Some("Destination index for the command"),
                                                    },
                                                ],
                                            },
                                            command: "move",
                                            help: Some("Move a command in the attack"),
                                        },
                                        &Item {
                                            item_type: ItemType::Callback {
                                                function: list,
                                                parameters: &[],
                                            },
                                            command: "list",
                                            help: Some("List all commands in the current attack"),
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
    context: &mut Context<C, T>,
) {
    writeln!(interface, "In enter_custom_attack").unwrap();
    context.attack_builder.reset();
}

fn exit_custom_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "In exit_custom_attack").unwrap();
    context
        .core
        .arm(&context.attack_builder.build().unwrap())
        .unwrap();
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
    let id_opt = argument_finder(item, args, "id").unwrap();
    let is_extended = match argument_finder(item, args, "extended").unwrap() {
        Some(_) => true,
        None => false,
    };

    if let Some(mut id_str) = id_opt {
        id_str = id_str.trim_start_matches("0x");
        if let Ok(id_val) = u32::from_str_radix(id_str, 16) {
            let id = if is_extended {
                Id::Extended(embedded_can::ExtendedId::new(id_val).unwrap())
            } else {
                Id::Standard(embedded_can::StandardId::new(id_val as u16).unwrap())
            };

            context
                .attack_builder
                .push(HighLevelAttackCmd::MatchId { id })
                .unwrap();

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
    let dlc_opt = argument_finder(item, args, "dlc").unwrap();
    let data_opt = argument_finder(item, args, "data").unwrap();

    if let Some(dlc_str) = dlc_opt {
        if let Ok(data_len) = str::parse(dlc_str) {
            let data = match data_opt {
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

            context
                .attack_builder
                .push(HighLevelAttackCmd::MatchData { data_len, data })
                .unwrap();

            writeln!(
                interface,
                "Added Match Data command with data: {:?} with len {}",
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

fn skip_data<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    context
        .attack_builder
        .push(HighLevelAttackCmd::SkipData)
        .unwrap();

    writeln!(interface, "Added Skip Data command").unwrap();
}

fn wait<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let bits_opt = argument_finder(item, args, "bits").unwrap();

    if let Some(bits_str) = bits_opt {
        if let Ok(bits) = str::parse(bits_str) {
            context
                .attack_builder
                .push(HighLevelAttackCmd::Wait { bits })
                .unwrap();

            writeln!(interface, "Added Wait command for {} bits", bits).unwrap();
        } else {
            writeln!(interface, "Invalid bits format").unwrap();
        }
    } else {
        writeln!(interface, "bits is required").unwrap();
    }
}

fn send_error<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let count_opt = argument_finder(item, args, "count").unwrap();

    if let Some(count_str) = count_opt {
        if let Ok(count) = str::parse(count_str) {
            context
                .attack_builder
                .push(HighLevelAttackCmd::SendError { count })
                .unwrap();

            writeln!(interface, "Added Send Error command with count = {}", count).unwrap();
        } else {
            writeln!(interface, "Invalid count format").unwrap();
        }
    } else {
        writeln!(interface, "count is required").unwrap();
    }
}

fn send_raw<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let bits_opt = argument_finder(item, args, "bits").unwrap();
    let force = match argument_finder(item, args, "force").unwrap() {
        Some(_) => true,
        None => false,
    };

    if let Some(bits_str) = bits_opt {
        if let Ok(bits) = u128::from_str_radix(bits_str, 2) {
            context
                .attack_builder
                .push(HighLevelAttackCmd::SendRaw {
                    bits,
                    bits_count: bits_str.len(),
                    force,
                })
                .unwrap();

            writeln!(
                interface,
                "Added Send Raw command with bits: {}, force {}",
                bits_str, force
            )
            .unwrap();
        } else {
            writeln!(interface, "Invalid ID format").unwrap();
        }
    } else {
        writeln!(interface, "ID is required").unwrap();
    }
}

fn wait_eof<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    context
        .attack_builder
        .push(HighLevelAttackCmd::WaitEof)
        .unwrap();

    writeln!(interface, "Added Wait EOF command").unwrap();
}

fn send_msg<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let id_opt = argument_finder(item, args, "id").unwrap();
    let data_opt = argument_finder(item, args, "data").unwrap();
    let is_extended = match argument_finder(item, args, "extended").unwrap() {
        Some(_) => true,
        None => false,
    };
    let rtr = match argument_finder(item, args, "rtr").unwrap() {
        Some(_) => true,
        None => false,
    };
    let force = match argument_finder(item, args, "force").unwrap() {
        Some(_) => true,
        None => false,
    };

    if let Some(mut id_str) = id_opt {
        id_str = id_str.trim_start_matches("0x");
        if let Ok(id_val) = u32::from_str_radix(id_str, 16) {
            let id = if is_extended {
                Id::Extended(embedded_can::ExtendedId::new(id_val).unwrap())
            } else {
                Id::Standard(embedded_can::StandardId::new(id_val as u16).unwrap())
            };

            let (data, data_len) = match data_opt {
                Some(s) => {
                    let mut data_array = [0u8; 8];
                    let mut data_len = 0;

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
                                data_len += 1;
                            }
                            Err(_) => {
                                writeln!(interface, "Error parsing hex data").unwrap();
                                return;
                            }
                        }
                    }
                    (Some(data_array), data_len)
                }
                None => (None, 0),
            };

            context
                .attack_builder
                .push(HighLevelAttackCmd::SendMsg {
                    id,
                    data,
                    data_len,
                    rtr,
                    force,
                })
                .unwrap();

            writeln!(
                interface,
                "Added Send Message command with id: {:?} and data: {:?} with len {}, RTR {}, force {}",
                id, data, data_len, rtr, force
            )
            .unwrap();
        } else {
            writeln!(interface, "Invalid ID format").unwrap();
        }
    } else {
        writeln!(interface, "ID is required").unwrap();
    }
}

fn send_overload_frame<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    context
        .attack_builder
        .push(HighLevelAttackCmd::SendOverloadFrame);

    writeln!(interface, "Added Send Overload Frame command").unwrap();
}

fn delete<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let idx_opt = argument_finder(item, args, "idx").unwrap();

    if let Some(idx_str) = idx_opt {
        if let Ok(idx) = str::parse(idx_str) {
            context.attack_builder.remove(idx);
            writeln!(interface, "Deleted command at idx {}", idx).unwrap();
        } else {
            writeln!(interface, "Invalid idx format").unwrap();
        }
    } else {
        writeln!(interface, "idx is required").unwrap();
    }
}

fn relocate<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let from_opt = argument_finder(item, args, "from").unwrap();
    let to_opt = argument_finder(item, args, "to").unwrap();

    if let Some(from_str) = from_opt {
        if let Ok(from) = str::parse(from_str) {
            if let Some(to_str) = to_opt {
                if let Ok(to) = str::parse(to_str) {
                    context.attack_builder.relocate(from, to);
                    writeln!(interface, "Moving command at from {} to {}", from, to).unwrap();
                } else {
                    writeln!(interface, "Invalid to format").unwrap();
                }
            } else {
                writeln!(interface, "to is required").unwrap();
            }
        } else {
            writeln!(interface, "Invalid from format").unwrap();
        }
    } else {
        writeln!(interface, "from is required").unwrap();
    }
}

fn list<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    for (idx, cmd) in context.attack_builder.iter().enumerate() {
        writeln!(interface, "\t{}: {:?}", idx, cmd).unwrap();
    }
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
