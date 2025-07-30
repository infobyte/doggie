use crate::machine::commands::AttackCmd;
use defmt::info;
use heapless::Vec;

fn optimize_bitstuffing<const SIZE: usize>(attack: &mut Vec<AttackCmd, SIZE>) {
    // Check if there is two SetBitStuffing with contraty states together and remove them
    let mut index = 0;
    while index < (attack.len() - 1) {
        if let AttackCmd::SetBitStuffing { state: fst_state } = attack.get(index).unwrap() {
            if let AttackCmd::SetBitStuffing { state: scd_state } = attack.get(index + 1).unwrap() {
                if fst_state != scd_state {
                    attack.remove(index);
                    attack.remove(index);
                } else {
                    attack.remove(index);
                }

                info!("Bitstuffing optimized at index {}", index);
            }
            index += 1;
        } else {
            index += 1;
        }
    }
}

pub fn optimize_attack<const SIZE: usize>(attack: &mut Vec<AttackCmd, SIZE>) {
    optimize_bitstuffing(attack);
}
