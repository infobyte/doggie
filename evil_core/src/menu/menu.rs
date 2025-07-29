use crate::bsp::{CanBitrates, TicksClock, Tranceiver};
use crate::evil_core::EvilCore;
use crate::machine::commands::builder::{
    AttackBuilder, HighLevelAttackCmd, PredefAttacks, MAX_HL_COMMANDS,
};
use crate::machine::commands::{AttackCmd, FastBitQueue};
use crate::machine::new_attack_buf;
use crate::menu::callbacks::*;
use defmt::{info, Debug2Format};
use embedded_can::Id;
use embedded_io::{Read, Write};
use heapless::Vec;
use menu::{argument_finder, Item, ItemType, Menu, Parameter, Runner};
use noline::builder::EditorBuilder;

pub const MAX_PLAN_SIZE: usize = 32;
pub const MAX_ATTACK_SIZE: usize = 128;

pub struct Context<CLK, TR>
where
    CLK: TicksClock,
    TR: Tranceiver,
{
    // Hay que chequear esto porque un comando de alto nivel puede dar lugar a multiples de bajo nivel.
    pub custom_attack: Vec<HighLevelAttackCmd, MAX_HL_COMMANDS>,
    pub plan_builder: AttackBuilder<MAX_PLAN_SIZE, MAX_ATTACK_SIZE, PredefAttacks>,
    attack_builder: AttackBuilder<MAX_ATTACK_SIZE, MAX_ATTACK_SIZE, HighLevelAttackCmd>,
    core: EvilCore<CLK, TR>,
}

impl<CLK, TR> Context<CLK, TR>
where
    CLK: TicksClock,
    TR: Tranceiver,
{
    fn new_with(core: EvilCore<CLK, TR>) -> Self {
        Self {
            custom_attack: Vec::new(),
            plan_builder: AttackBuilder::new(),
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
    pub fn new(serial: SERIAL, mut core: EvilCore<CLK, TR>) -> Self {
        let menu = Menu {
            label: "root",
            items: &[
                &Item {
                    item_type: ItemType::Callback {
                        function: cmd_set_baudrate,
                        parameters: &[Parameter::Mandatory {
                            parameter_name: "baudrate",
                            help: Some(
                                "In kbps. Valid baudrates are 5, 10, 20, 50, 100, 125, 250, 500, 1000",
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
                                                function: wait_bus_free,
                                                parameters: &[],
                                            },
                                            command: "wait_bus_free",
                                            help: Some("Add a wait for bus free command to the attack"),
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
                                                function: set_bitstuffing,
                                                parameters: &[
                                                    Parameter::Mandatory {
                                                        parameter_name: "state",
                                                        help: Some("'enable' or 'disable'"),
                                                    },
                                                ],
                                            },
                                            command: "set_bitstuffing",
                                            help: Some("Set bitstuffing state"),
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
                        function: spoofing_attack,
                        parameters: &[
                            Parameter::Mandatory {
                                parameter_name: "id",
                                help: Some("CAN ID to send in hex (e.g, 0x123)"),
                            },
                            Parameter::Mandatory {
                                parameter_name: "spoofed_data",
                                help: Some("Data bytes to spoof as comma-separated hex values (e.g., 0x10,0x20,0x30)"),
                            },
                            Parameter::Optional {
                                parameter_name: "match_data",
                                help: Some("First data bytes to match as comma-separated hex values (e.g., 0x10,0x20,0x30)"),
                            },
                            Parameter::Named {
                                parameter_name: "extended",
                                help: Some("Whether this is an extended ID (defaults to standard ID)"),
                            },
                        ],
                    },
                    command: "spoofing_attack",
                    help: Some("Push a spoofing attack over an ID"),
                },


                &Item {
                    item_type: ItemType::Callback {
                        function: bus_off_attack,
                        parameters: &[
                            Parameter::Mandatory {
                                parameter_name: "id",
                                help: Some("CAN ID to send in hex (e.g, 0x123)"),
                            },
                            Parameter::Mandatory { parameter_name: "errors", help: Some("Amount of consecutive error to send") },
                            Parameter::Optional {
                                parameter_name: "match_data",
                                help: Some("First data bytes to match as comma-separated hex values (e.g., 0x10,0x20,0x30)"),
                            },
                            Parameter::Named {
                                parameter_name: "extended",
                                help: Some("Whether this is an extended ID (defaults to standard ID)"),
                            },
                        ],
                    },
                    command: "bus_off_attack",
                    help: Some("Push a bus_off_attack over an ID"),
                },

                &Item {
                    item_type: ItemType::Callback {
                        function: attack,
                        parameters: &[
                            Parameter::Optional {
                                parameter_name: "successes",
                                help: Some("Number of successfull attacks to do (default: 1)"),
                            },
                            Parameter::Optional {
                                parameter_name: "retries",
                                help: Some("Number of retries until aborting the attack (default: infinite)"),
                            },                        ],
                    },
                    command: "attack",
                    help: Some("Start the attack"),
                },
            ],
            entry: None,
            exit: None,
        };

        // Warmup attack
        let mut warmup_buf = new_attack_buf();
        warmup_buf[1] = AttackCmd::WaitBusFree { count: 0 };
        warmup_buf[1] = AttackCmd::Send {
            stream: FastBitQueue::new(0xFF, 8),
        };
        warmup_buf[0] = AttackCmd::SetBitStuffing { state: false };
        warmup_buf[1] = AttackCmd::Send {
            stream: FastBitQueue::new(0xFF, 8),
        };
        warmup_buf[2] = AttackCmd::SetBitStuffing { state: true };
        core.board_specific_attack(&warmup_buf, 1, Some(1));

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
                //5, 10, 20, 50, 100, 125, 250, 500, 1000
                CanBitrates::Kbps5,
                CanBitrates::Kbps10,
                CanBitrates::Kbps20,
                CanBitrates::Kbps50,
                CanBitrates::Kbps100,
                CanBitrates::Kbps125,
                CanBitrates::Kbps250,
                CanBitrates::Kbps500,
                CanBitrates::Kbps1000,
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

fn parse_data<'a>(
    input_str: &'a str,
    data_array: &'a mut Vec<u8, 8>,
) -> Result<usize, &'static str> {
    for (i, hex_str) in input_str.split(',').enumerate() {
        if i >= 8 {
            return Err("Error: Data exceeds maximum length of 8 bytes");
        }

        let trimmed = hex_str.trim().trim_start_matches("0x");
        match u8::from_str_radix(trimmed, 16) {
            Ok(value) => {
                data_array.push(value).unwrap();
            }
            Err(_) => {
                return Err("Error parsing hex data");
            }
        }
    }

    Ok(data_array.len())
}

fn spoofing_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "Spoofing attack").unwrap();
    let mut id_str = argument_finder(item, args, "id").unwrap().unwrap();
    let match_data_str_opt = argument_finder(item, args, "match_data").unwrap();
    let spoof_data_str = argument_finder(item, args, "spoofed_data")
        .unwrap()
        .unwrap();
    let is_extended = match argument_finder(item, args, "extended").unwrap() {
        Some(_) => true,
        None => false,
    };

    id_str = id_str.trim_start_matches("0x");
    let id = if let Ok(id_val) = u32::from_str_radix(id_str, 16) {
        if is_extended {
            Id::Extended(embedded_can::ExtendedId::new(id_val).unwrap())
        } else {
            Id::Standard(embedded_can::StandardId::new(id_val as u16).unwrap())
        }
    } else {
        writeln!(interface, "Invalid ID format").unwrap();
        return;
    };

    let mut spoof_data: Vec<u8, 8> = Vec::new();

    match parse_data(spoof_data_str, &mut spoof_data) {
        Err(err_str) => {
            writeln!(interface, "{}", err_str).unwrap();
            return;
        }
        _ => {}
    }

    let mut match_data: Vec<u8, 8> = Vec::new();

    match match_data_str_opt {
        Some(match_data_str) => match parse_data(match_data_str, &mut match_data) {
            Err(err_str) => {
                writeln!(interface, "{}", err_str).unwrap();
                return;
            }
            _ => {}
        },
        _ => {}
    }

    context
        .plan_builder
        .push(PredefAttacks::SpoofingAttack {
            id,
            spoof_data: spoof_data.clone(),
            match_data: match_data.clone(),
        })
        .unwrap();

    writeln!(
        interface,
        "Added Spoofing attack with:\n\tid: {:?}\n\tspoof data: {:?}\n\tdata to match {:?}",
        id, spoof_data, match_data
    )
    .unwrap();
}

fn bus_off_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "Spoofing attack").unwrap();
    let mut id_str = argument_finder(item, args, "id").unwrap().unwrap();
    let match_data_str_opt = argument_finder(item, args, "match_data").unwrap();
    let is_extended = match argument_finder(item, args, "extended").unwrap() {
        Some(_) => true,
        None => false,
    };

    id_str = id_str.trim_start_matches("0x");
    let id = if let Ok(id_val) = u32::from_str_radix(id_str, 16) {
        if is_extended {
            Id::Extended(embedded_can::ExtendedId::new(id_val).unwrap())
        } else {
            Id::Standard(embedded_can::StandardId::new(id_val as u16).unwrap())
        }
    } else {
        writeln!(interface, "Invalid ID format").unwrap();
        return;
    };

    let errors_opt = argument_finder(item, args, "errors").unwrap();

    let errors = if let Some(errors_str) = errors_opt {
        if let Ok(errors) = str::parse(errors_str) {
            errors
        } else {
            writeln!(interface, "Invalid errors format").unwrap();
            return;
        }
    } else {
        writeln!(interface, "errors is required").unwrap();
        return;
    };

    let mut match_data: Vec<u8, 8> = Vec::new();

    match match_data_str_opt {
        Some(match_data_str) => match parse_data(match_data_str, &mut match_data) {
            Err(err_str) => {
                writeln!(interface, "{}", err_str).unwrap();
                return;
            }
            _ => {}
        },
        _ => {}
    }

    context
        .plan_builder
        .push(PredefAttacks::BusOffAttack {
            id,
            match_data: match_data.clone(),
            errors,
        })
        .unwrap();

    writeln!(
        interface,
        "Added Bus Off attack with:\n\tid: {:?}\n\tErrors: {:?}\n\tdata to match {:?}",
        id, errors, match_data
    )
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
    context
        .plan_builder
        .push(PredefAttacks::TestAttack)
        .unwrap();
}

fn attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let successes = match argument_finder(item, args, "successes").unwrap() {
        Some(successes_str) => match successes_str.parse::<usize>() {
            Ok(res) => {
                if res > 0 {
                    res
                } else {
                    writeln!(
                        interface,
                        "Invalid number of successes, must be grater than 0"
                    )
                    .unwrap();
                    return;
                }
            }
            Err(_) => {
                writeln!(
                    interface,
                    "Invalid number of successes, must be a positive number."
                )
                .unwrap();
                return;
            }
        },
        None => 1,
    };

    let retries = match argument_finder(item, args, "retries").unwrap() {
        Some(retries_str) => match retries_str.parse::<usize>() {
            Ok(res) => Some(res),
            Err(_) => {
                writeln!(
                    interface,
                    "Invalid number of retries, must be a positive number."
                )
                .unwrap();
                return;
            }
        },
        None => None,
    };

    writeln!(interface, "Arming the attack").unwrap();
    let mut hl_attack_vec = Vec::new();
    let mut attack_vec = Vec::new();

    info!("Building the attack plan");
    context.plan_builder.build(&mut hl_attack_vec).unwrap();

    info!("Result:");
    for attack_cmd in &hl_attack_vec {
        info!("\t{:?}", Debug2Format(attack_cmd));
    }

    context.attack_builder.reset();

    hl_attack_vec
        .iter()
        .for_each(|attack_cmd| context.attack_builder.push(*attack_cmd).unwrap());

    context.attack_builder.build(&mut attack_vec).unwrap();

    info!("About to run attack with:");
    for cmd in &attack_vec {
        info!("\t{:?}", Debug2Format(cmd));
    }

    writeln!(interface, "Launching attack").unwrap();

    if context
        .core
        .board_specific_attack(attack_vec.as_slice(), successes, retries)
    {
        writeln!(interface, "Attack successfull!!").unwrap();
    } else {
        writeln!(interface, "Attack failed!!").unwrap();
    }
}
