use crate::bsp::{TicksClock, Tranceiver};
use crate::machine::commands::builder::{HighLevelAttackCmd, PredefAttacks};
use crate::menu::Context;
use embedded_can::Id;
use embedded_io::{Read, Write};
use menu::{argument_finder, Item, Menu};

pub fn enter_custom_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "In enter_custom_attack").unwrap();
    context.custom_attack.clear();
}

pub fn exit_custom_attack<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    writeln!(interface, "In exit_custom_attack").unwrap();
    context
        .plan_builder
        .push(PredefAttacks::CustomAttack {
            commands: context.custom_attack.clone(),
        })
        .unwrap();
}

pub fn match_id<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
                .custom_attack
                .push(HighLevelAttackCmd::MatchId { id, rtr: false })
                .unwrap(); // TODO: Add RTR to the command

            writeln!(interface, "Added Match Id command with Id: {:?}", id).unwrap();
        } else {
            writeln!(interface, "Invalid ID format").unwrap();
        }
    } else {
        writeln!(interface, "ID is required").unwrap();
    }
}

pub fn match_data<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
                    data_array
                }
                None => [0; 8],
            };

            context
                .custom_attack
                .push(HighLevelAttackCmd::MatchData {
                    match_size: data_len,
                    data,
                })
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

pub fn skip_data<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    context
        .custom_attack
        .push(HighLevelAttackCmd::SkipData)
        .unwrap();

    writeln!(interface, "Added Skip Data command").unwrap();
}

pub fn wait<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
                .custom_attack
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

pub fn send_error<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
                .custom_attack
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

pub fn send_raw<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
        if let Ok(bits) = u64::from_str_radix(bits_str, 2) {
            context
                .custom_attack
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

pub fn wait_eof<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    context
        .custom_attack
        .push(HighLevelAttackCmd::WaitBusFree)
        .unwrap();

    writeln!(interface, "Added Wait EOF command").unwrap();
}

pub fn send_msg<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
                .custom_attack
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

pub fn delete<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    item: &Item<I, Context<C, T>>,
    args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    let idx_opt = argument_finder(item, args, "idx").unwrap();

    if let Some(idx_str) = idx_opt {
        if let Ok(idx) = str::parse(idx_str) {
            context.custom_attack.remove(idx);
            writeln!(interface, "Deleted command at idx {}", idx).unwrap();
        } else {
            writeln!(interface, "Invalid idx format").unwrap();
        }
    } else {
        writeln!(interface, "idx is required").unwrap();
    }
}

pub fn relocate<I: Read + Write, C: TicksClock, T: Tranceiver>(
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
                    if from >= context.custom_attack.len() || to >= context.custom_attack.len() {
                        writeln!(interface, "Index out of bounds").unwrap();
                        return ();
                    }

                    let from_cmd = context.custom_attack.remove(from);
                    // Shouldn fail
                    context.custom_attack.insert(to, from_cmd).unwrap();
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

pub fn list<I: Read + Write, C: TicksClock, T: Tranceiver>(
    _menu: &Menu<I, Context<C, T>>,
    _item: &Item<I, Context<C, T>>,
    _args: &[&str],
    interface: &mut I,
    context: &mut Context<C, T>,
) {
    for (idx, cmd) in context.custom_attack.iter().enumerate() {
        writeln!(interface, "\t{}: {:?}", idx, cmd).unwrap();
    }
}
