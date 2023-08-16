use assert_cmd::Command;

pub fn release_plz_cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}
