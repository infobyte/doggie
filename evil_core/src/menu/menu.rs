use crate::bsp::{CanBitrates, TicksClock, Tranceiver};
use crate::evil_core::EvilCore;
use crate::machine::commands::builder::{
    AttackBuilder, HighLevelAttackCmd, PredefAttacks, MAX_HL_COMMANDS,
};
use crate::menu::{callbacks::*, optimizations};
use crate::strings;
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
                            help: Some(strings::SET_BAUDRATE_PARAM_TXT),
                        }],
                    },
                    command: "set_baudrate",
                    help: Some(strings::SET_BAUDRATE_TXT),
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
                                            help: Some(strings::PARAM_MATCH_ID_TXT),
                                        },
                                        Parameter::Named {
                                            parameter_name: "extended",
                                            help: Some(strings::PARAM_EXTENDED_TXT),
                                        },
                                    ],
                                },
                                command: "match_id",
                                help: Some(strings::AC_MATCH_ID_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: match_data,
                                    parameters: &[
                                        Parameter::Mandatory {
                                            parameter_name: "dlc",
                                            help: Some(strings::AC_MATCH_DATA_DLC_TXT),
                                        },
                                        Parameter::Optional {
                                            parameter_name: "data",
                                            help: Some(strings::AC_MATCH_DATA_DATA_TXT),
                                        },
                                    ],
                                },
                                command: "match_data",
                                help: Some(strings::AC_MATCH_DATA_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: skip_data,
                                    parameters: &[],
                                },
                                command: "skip_data",
                                help: Some(strings::AC_SKIP_DATA_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: wait,
                                    parameters: &[Parameter::Mandatory {
                                        parameter_name: "bits",
                                        help: Some(strings::AC_WAIT_BITS_TXT),
                                    }],
                                },
                                command: "wait",
                                help: Some(strings::AC_WAIT_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: send_error,
                                    parameters: &[Parameter::Mandatory {
                                        parameter_name: "count",
                                        help: Some(strings::AC_SEND_ERROR_COUNT_TXT),
                                    }],
                                },
                                command: "send_error",
                                help: Some(strings::AC_SEND_ERROR_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: send_raw,
                                    parameters: &[
                                        Parameter::Mandatory {
                                            parameter_name: "bits",
                                            help: Some(strings::AC_SEND_RAW_BITS_TXT),
                                        },
                                        Parameter::Named {
                                            parameter_name: "force",
                                            help: Some(strings::PARAM_FORCE_TXT),
                                        },
                                    ],
                                },
                                command: "send_raw",
                                help: Some(strings::AC_SEND_RAW_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: wait_bus_free,
                                    parameters: &[],
                                },
                                command: "wait_bus_free",
                                help: Some(strings::AC_WAIT_BUS_FREE_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: send_msg,
                                    parameters: &[
                                        Parameter::Mandatory {
                                            parameter_name: "id",
                                            help: Some(strings::PARAM_SEND_ID_TXT),
                                        },
                                        Parameter::Named {
                                            parameter_name: "extended",
                                            help: Some(strings::PARAM_EXTENDED_TXT),
                                        },
                                        Parameter::Named {
                                            parameter_name: "rtr",
                                            help: Some(strings::AC_SEND_MSG_RTR_TXT),
                                        },
                                        Parameter::Named {
                                            parameter_name: "force",
                                            help: Some(strings::PARAM_FORCE_TXT),
                                        },
                                        Parameter::Optional {
                                            parameter_name: "data",
                                            help: Some(strings::AC_SEND_MSG_DATA_TXT),
                                        },
                                    ],
                                },
                                command: "send_msg",
                                help: Some(strings::AC_SEND_MSG_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: set_bitstuffing,
                                    parameters: &[Parameter::Mandatory {
                                        parameter_name: "state",
                                        help: Some(strings::AC_SET_BITSTUFFING_STATE_TXT),
                                    }],
                                },
                                command: "set_bitstuffing",
                                help: Some(strings::AC_SET_BITSTUFFING_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: delete,
                                    parameters: &[Parameter::Mandatory {
                                        parameter_name: "index",
                                        help: Some(strings::AC_DELETE_IDX_TXT),
                                    }],
                                },
                                command: "delete",
                                help: Some(strings::AC_DELETE_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: relocate,
                                    parameters: &[
                                        Parameter::Mandatory {
                                            parameter_name: "from",
                                            help: Some(strings::AC_MOVE_FROM_TXT),
                                        },
                                        Parameter::Mandatory {
                                            parameter_name: "to",
                                            help: Some(strings::AC_MOVE_TO_TXT),
                                        },
                                    ],
                                },
                                command: "move",
                                help: Some(strings::AC_MOVE_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: list,
                                    parameters: &[],
                                },
                                command: "list",
                                help: Some(strings::AC_LIST_TXT),
                            },
                            &Item {
                                item_type: ItemType::Callback {
                                    function: save,
                                    parameters: &[],
                                },
                                command: "save",
                                help: Some(strings::AC_SAVE_AND_EXIT_TXT),
                            },
                        ],
                        entry: Some(enter_custom_attack),
                        exit: Some(exit_custom_attack),
                    }),
                    command: "custom_attack",
                    help: Some(strings::CUSTOM_ATTACK_TXT),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: spoofing_attack,
                        parameters: &[
                            Parameter::Mandatory {
                                parameter_name: "id",
                                help: Some(strings::PARAM_MATCH_ID_TXT),
                            },
                            Parameter::Mandatory {
                                parameter_name: "spoofed_data",
                                help: Some(strings::SPOOFING_ATTACK_DATA_TXT),
                            },
                            Parameter::Optional {
                                parameter_name: "match_data",
                                help: Some(strings::PARAM_MATCH_DATA_TXT),
                            },
                            Parameter::Named {
                                parameter_name: "extended",
                                help: Some(strings::PARAM_EXTENDED_TXT),
                            },
                        ],
                    },
                    command: "spoofing_attack",
                    help: Some(strings::SPOOFING_ATTACK_TXT),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: bus_off_attack,
                        parameters: &[
                            Parameter::Mandatory {
                                parameter_name: "id",
                                help: Some(strings::PARAM_MATCH_ID_TXT),
                            },
                            Parameter::Mandatory {
                                parameter_name: "errors",
                                help: Some(strings::PARAM_ERRORS_TXT),
                            },
                            Parameter::Optional {
                                parameter_name: "match_data",
                                help: Some(strings::PARAM_MATCH_DATA_TXT),
                            },
                            Parameter::Named {
                                parameter_name: "extended",
                                help: Some(strings::PARAM_EXTENDED_TXT),
                            },
                        ],
                    },
                    command: "bus_off_attack",
                    help: Some(strings::BUS_OFF_ATTACK_TXT),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: double_receive_attack,
                        parameters: &[
                            Parameter::Mandatory {
                                parameter_name: "id",
                                help: Some(strings::PARAM_MATCH_ID_TXT),
                            },
                            Parameter::Mandatory {
                                parameter_name: "errors",
                                help: Some(strings::PARAM_ERRORS_TXT),
                            },
                            Parameter::Optional {
                                parameter_name: "match_data",
                                help: Some(strings::PARAM_MATCH_DATA_TXT),
                            },
                            Parameter::Named {
                                parameter_name: "extended",
                                help: Some(strings::PARAM_EXTENDED_TXT),
                            },
                        ],
                    },
                    command: "double_receive_attack",
                    help: Some(strings::DOUBLE_RECEIVE_ATTACK_TXT),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: delete_attack,
                        parameters: &[Parameter::Mandatory {
                            parameter_name: "index",
                            help: Some(strings::DELETE_ATTACK_INDEX_TXT),
                        }],
                    },
                    command: "delete",
                    help: Some(strings::DELETE_ATTACK_TXT),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: relocate_attack,
                        parameters: &[
                            Parameter::Mandatory {
                                parameter_name: "from",
                                help: Some(
                                    "Source index of the attack in the plan (starts from 0)",
                                ),
                            },
                            Parameter::Mandatory {
                                parameter_name: "to",
                                help: Some("Destination index for the attack"),
                            },
                        ],
                    },
                    command: "move",
                    help: Some(strings::MOVE_ATTACK_TXT),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: list_attacks,
                        parameters: &[],
                    },
                    command: "list",
                    help: Some(strings::LIST_PLAN_TXT),
                },
                &Item {
                    item_type: ItemType::Callback {
                        function: attack,
                        parameters: &[
                            Parameter::Optional {
                                parameter_name: "successes",
                                help: Some(strings::ATTACK_SUCCESSES_TXT),
                            },
                            Parameter::Optional {
                                parameter_name: "retries",
                                help: Some(strings::ATTACK_RETIRES_TXT),
                            },
                        ],
                    },
                    command: "attack",
                    help: Some(strings::ATTACK_TXT),
                },
            ],
            entry: Some(Self::enter_root),
            exit: None,
        };

        EvilMenu {
            serial: Some(serial),
            menu: Some(menu),
            context: Some(Context::new_with(core)),
        }
    }

    pub fn enter_root<I: Read + Write, C: TicksClock, T: Tranceiver>(
        _menu: &Menu<I, Context<C, T>>,
        interface: &mut I,
        _context: &mut Context<C, T>,
    ) {
        interface.write(strings::ENTER_ROOT_TXT.as_bytes()).unwrap();
    }

    fn wait_and_print_banner(serial: &mut SERIAL) {
        let mut buf: [u8; 1] = [0];

        loop {
            match serial.read(&mut buf) {
                Ok(size) => {
                    if size > 0 {
                        break;
                    }
                }
                _ => {}
            }
        }

        serial
            .write(
                b"
         :==:                             .=+=.
       .*=*=+:                         .=+++-*.
      .*=%*++=+.                      :+=*%%*==
      ==%%*.=*:*.                    :*=*=#%%=+:
     .*=%%+  *%:*.                  =++*:=%%%#-=
     -=*%#:  .%%-*=+****=-:-=****==*:*%= =%%%%-=.
     +:#%*   =%%*:-+*##*+=--=*#%#*=:*%%. .*%%%-=:
    .+:-%*  .#%%%%%%%%%%%%%%%%%%%%%%%%%=  +%%*:+.
     :+ -*-=*%%%%%%%%%%%%%%%%%%%%%%%%%%%*:*%*.+:
      :*-*%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%=+:
        +=*%%%%%%%%%%%%#%%%%%%%%%%%%%%%%%%%**:
        :*=%%%%%%%**%%%**%%%%%#%#=+#%%%%%%*:+
        :+=%%%%=::=*#%%%%%%%%%%%**=::-*%%%=:*
        ==+%%%%*: :=#%%%%%%%%%%%*-: :*%%%%*.*:
        *-%%%%%%%%%%%%#%%%%%%%%%%%%%%%%%%%%:-=
       .*:%%%#%%%%%%+%**%%%%%%%%*%%%%%%%%%%::=
        *-#%%%%%%%%*%%*:-====:=%%#%%%%%%%%*.==
        :+=%%%%%%%%%%* . .     -%%%%%%%%%#: *.
        .=:=%%%*%%%%%+ ::-:.:..=%%%%%%%%#: -=
         -= -%%=%%%%%%+::.:..:=%%%%*%%%#:  *:
        .=:-::+=%%%%%%%%#=:-*%%%%%%%#%*.  .*.
        :*:##: =%%%%%%%%%=.=%%%%%%%#*%+.: .*.
        =-:*%+ =%%%%%%#-.   .:*%%%%#%%=::  +:
       .*:*#*%..%%%%%=:*%%%%%#-:%%%%%#:=  :*#.
       ==:*%%=- .-+-. +%%%%%%%= .=**::: :+- :*
      :# -%%%%%*                      :*:  .=-=
     .*. :*%%%%%%%=: .............:=+=    -: .==
    .+: -*:=*%%%%%%%+:.........:::     .::.-: :=:
    ==.+%%%#=:-**%%%+::---==-:==-: .::-:  :=.  :=.
   :=:*%%%%%%%%:=-:::::::::::::::::.      =-.  .=-
  .=-=%%%%*%%%%#**#:                      =-+   :=.
  :=-#%%%%#**%%%%%*:                      ::+   .=:
 .=-*%%%%%%%+-%%%%%%#:                    ::     =-
 :+:#%%%%%%%%%=%%%%%%#*:                         :=.
 -==%%%%%%%%%%%%%%%%%%%%=                         +:
 =-+%%%%%%%%%%%%%%%%%%%%%=                        +:
.+:*%%%%%%%%%%%%%%%%%%%%%%+.                      =-
:*:#%%%%%%%%%%%%%%%%%%%%%%%*:                     -=
:+-%%%%%%%%%%%%%%%%%%%%%%%%%#:                    :=
===%%%%%%%%%%%%%%%%%%%%%%%%%%#:                   :=
_______________   ____.___.____                       
\\_   _____/\\   \\ /   /|   |    |                      
 |    __)_  \\   Y   / |   |    |                      
 |        \\  \\     /  |   |    |___                   
/_______  /   \\___/   |___|_______ \\                  
        \\/                        \\/  
________  _______    ________  ________.______________
\\______ \\ \\   _  \\  /  _____/ /  _____/|   \\_   _____/
 |    |  \\/  /_\\  \\/   \\  ___/   \\  ___|   ||    __)_
 |    `   \\  \\_/   \\    \\_\\  \\    \\_\\  \\   ||        \\
/_______  /\\_____  /\\______  /\\______  /___/_______  /
        \\/       \\/        \\/        \\/            \\/
",
            )
            .unwrap();
    }

    pub fn run(&'a mut self) {
        let mut serial = self.serial.take().unwrap();
        Self::wait_and_print_banner(&mut serial);
        self.serial.replace(serial);

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
    writeln!(interface, "Bus off attack").unwrap();
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

fn double_receive_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "Double receive attack").unwrap();
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
        .push(PredefAttacks::DoubleReceiveAttack {
            id,
            match_data: match_data.clone(),
            errors,
        })
        .unwrap();

    writeln!(
        interface,
        "Added Double Receive attack with:\n\tid: {:?}\n\tErrors: {:?}\n\tdata to match {:?}",
        id, errors, match_data
    )
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
    for attack in context.plan_builder.iter() {
        info!("\t{:?}", Debug2Format(attack));
    }

    info!("WarmUp added at the beginning");
    hl_attack_vec.insert(0, HighLevelAttackCmd::WarmUp).unwrap();

    info!("HL Result:");
    for attack_cmd in &hl_attack_vec {
        info!("\t{:?}", Debug2Format(attack_cmd));
    }

    context.attack_builder.reset();

    hl_attack_vec
        .iter()
        .for_each(|attack_cmd| context.attack_builder.push(*attack_cmd).unwrap());

    context.attack_builder.build(&mut attack_vec).unwrap();

    writeln!(interface, "Optimizing the attack").unwrap();

    info!("Before optimizations:");
    for cmd in &attack_vec {
        info!("\t{:?}", Debug2Format(cmd));
    }

    optimizations::optimize_attack(&mut attack_vec);

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

pub fn delete_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let idx_opt = argument_finder(item, args, "idx").unwrap();

    if let Some(idx_str) = idx_opt {
        if let Ok(idx) = str::parse(idx_str) {
            match context.plan_builder.remove(idx) {
                Ok(_) => writeln!(interface, "Deleted attack at idx {}", idx).unwrap(),
                Err(_) => writeln!(interface, "Error deleting attack at idx {}", idx).unwrap(),
            }
        } else {
            writeln!(interface, "Invalid idx format").unwrap();
        }
    } else {
        writeln!(interface, "idx is required").unwrap();
    }
}

pub fn relocate_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
                    match context.plan_builder.relocate(from, to) {
                        Ok(_) => {
                            writeln!(interface, "Moved attack at from {} to {}", from, to).unwrap();
                        }
                        Err(_) => {
                            writeln!(interface, "Error moving attack at from {} to {}", from, to)
                                .unwrap();
                        }
                    }
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

pub fn list_attacks<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    for (idx, cmd) in context.plan_builder.iter().enumerate() {
        writeln!(interface, "\t{}: {:?}", idx, cmd).unwrap();
    }
}
