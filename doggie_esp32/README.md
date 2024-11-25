cargo install ldproxy
cargo install espup
espup install
Correr $HOME/export-esp.sh o agregar lo que hace a .zshrc o .bashrc por ejemplo
cargo install espflash
cargo install cargo-espflash # Optional espflash cargo command


correr:
DEFMT_LOG=off cargo run --release

si quieren ver logs porque algo no funciona:
DEFMT_LOG=trace cargo run --release