#![no_std]

use embedded_can::Id;
use embedded_io::{Read, Write};
use evil_core::{
    clock::TicksClock, tranceiver::Tranceiver, AttackCmd, CanBitrates, EvilCore, FastBitQueue,
    MAX_ATTACK_SIZE,
};
use menu::{argument_finder, Item, ItemType, Menu, Parameter, Runner};
use noline::builder::EditorBuilder;

#[derive(Clone, Copy, PartialEq)]
enum HighLevelAttackCmd {
    None,
    // Provisorio
    Match {
        id: Id,
        data_len: usize,
        data: Option<[u8; 8]>,
    },
    MatchId {
        id: Id,
    },
    MatchData {
        data_len: usize,
        data: Option<[u8; 8]>,
    },
    SkipData,
    Wait {
        bits: usize,
    },
    SendError {
        count: usize,
    },
    SendRaw {
        bits: u64,
        bits_count: usize,
        force: bool,
    },
    WaitEof,
    SendMsg {
        id: Id,
        data: Option<[u8; 8]>,
        data_len: usize,
        rtr: bool,
        force: bool,
    },
    SendOverloadFrame,
}

struct AttackBuilder {
    low_level_attack: [AttackCmd; MAX_ATTACK_SIZE],
    low_level_attack_index: usize,
    high_level_attack: [HighLevelAttackCmd; MAX_ATTACK_SIZE],
    high_level_attack_index: usize,
}

impl AttackBuilder {
    fn new() -> Self {
        let mut attack = [AttackCmd::None; MAX_ATTACK_SIZE];
        let high_level_attack = [HighLevelAttackCmd::None; MAX_ATTACK_SIZE];
        // Start of frame
        attack[0] = AttackCmd::Wait { bits: 1 };

        Self {
            low_level_attack: attack,
            low_level_attack_index: 1,
            high_level_attack,
            high_level_attack_index: 0,
        }
    }

    fn reset(&mut self) {
        self.low_level_attack_index = 1;
        self.low_level_attack = [AttackCmd::None; MAX_ATTACK_SIZE];
        self.low_level_attack[0] = AttackCmd::Wait { bits: 1 };
        self.high_level_attack = [HighLevelAttackCmd::None; MAX_ATTACK_SIZE];
        self.high_level_attack_index = 0;
    }

    fn push_low_level_attack_cmd(&mut self, attack: AttackCmd) {
        self.low_level_attack[self.low_level_attack_index] = attack;
        self.low_level_attack_index += 1;
    }

    fn push_high_level_attack_cmd(&mut self, attack: HighLevelAttackCmd) {
        self.high_level_attack[self.high_level_attack_index] = attack;
        self.high_level_attack_index += 1;
    }

    fn build(&mut self) -> &[AttackCmd] {
        // Iterate over high level attack commands and build the atttack
        for i in 0..self.high_level_attack_index {
            let cmd = self.high_level_attack[i];
            match cmd {
                HighLevelAttackCmd::None => {}
                HighLevelAttackCmd::Match { id, data, data_len } => match id {
                    Id::Standard(id) => {
                        match data {
                            None => {
                                // Match {Id (11 bits), RTR bit = 0, Id Extension bit = 0, Reserved bit = 0, DLC (4 bits) = 0b0000 }
                                // Total 18 bits
                                let bits = (id.as_raw() as u64) << 7;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits.into(), 14),
                                });
                            }
                            Some(data) => {
                                // Match {Id (11 bits), RTR bit = 0, Id Extension bit = 0, Reserved bit = 0, DLC (4 bits) }
                                // Total 18 bits
                                let bits = ((id.as_raw() as u64) << 7) | data_len as u64;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, 18),
                                });

                                // Data
                                let bits = data[..data_len]
                                    .iter()
                                    .rev()
                                    .enumerate()
                                    .map(|(i, &b)| (b as u64) << (i * 8))
                                    .sum();
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, data_len),
                                });
                            }
                        }
                    }
                    Id::Extended(id) => {
                        match data {
                            None => {
                                // Match { Id (11 bits), SRR bit = 1, Id Extension bit = 1, Id (18 bits), RTR = 0, Reserved bit = 0, Reserved bit = 0, DLC (4 bits) = 0b0000 }
                                // Total 38 bits
                                let bits_high = (id.as_raw() as u64 & 0x1ffc0000) >> 18;
                                let bits_low = id.as_raw() as u64 & 0x3ffff;
                                let bits = bits_high << 27 | 1 << 26 | 1 << 25 | bits_low << 7;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, 38),
                                });
                            }
                            Some(data) => {
                                // Match { Id (11 bits), SRR bit = 1, Id Extension bit = 1, Id (18 bits), RTR = 0, Reserved bit = 0, Reserved bit = 0, DLC (4 bits) }
                                // Total 38 bits
                                let bits_high = (id.as_raw() as u64 & 0x1ffc0000) >> 18;
                                let bits_low = id.as_raw() as u64 & 0x3ffff;
                                let bits = bits_high << 27
                                    | 1 << 26
                                    | 1 << 25
                                    | bits_low << 7
                                    | data_len as u64;
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, 38),
                                });

                                // Data
                                let bits = data[..data_len]
                                    .iter()
                                    .rev()
                                    .enumerate()
                                    .map(|(i, &b)| (b as u64) << (i * 8))
                                    .sum();
                                self.push_low_level_attack_cmd(AttackCmd::Match {
                                    stream: FastBitQueue::new(bits, data_len),
                                });
                            }
                        }
                    }
                },
                _ => {
                    todo!()
                }
            }
        }

        &self.low_level_attack[..self.low_level_attack_index]
    }

    fn set_test_attack(&mut self) {
        self.low_level_attack = [AttackCmd::None; MAX_ATTACK_SIZE];
        self.low_level_attack[0] = AttackCmd::Wait { bits: 1 };
        self.low_level_attack[1] = AttackCmd::Force {
            stream: FastBitQueue::new(0b1010_101, 7),
        };
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
                    item_type: ItemType::Callback {
                        function: add_match,
                        parameters: &[
                            Parameter::Mandatory {
                                parameter_name: "id",
                                help: Some("CAN ID to match in hex (e.g, 0x123))"),
                            },
                            Parameter::Named {
                                parameter_name: "extended",
                                help: Some("Whether this is an extended ID (defaults to standard ID)"),
                            },
                            Parameter::Optional {
                                parameter_name: "data",
                                help: Some("Optional data bytes as comma-separated hex values (e.g., 0x10,0x20,0x30)"),
                            },
                        ],
                    },
                    command: "add_match",
                    help: Some("Add a CAN frame match condition to the attack"),
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

// Menu callbacks
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

fn add_match<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let id_str = argument_finder(item, args, "id").unwrap();
    let data_str = argument_finder(item, args, "data").unwrap();
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

            // Parse data if provided
            let (data_opt, data_len) = match data_str {
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

            // Add the match command
            context
                .attack_builder
                .push_high_level_attack_cmd(HighLevelAttackCmd::Match {
                    id,
                    data: data_opt,
                    data_len,
                });

            writeln!(
                interface,
                "Added match command with ID: {:?} and data: {:?} with len {}",
                id,
                data_opt.unwrap(),
                data_len
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
