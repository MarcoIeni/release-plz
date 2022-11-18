use assert_cmd::Command;
use clap::ValueEnum;
use clap_complete::Shell;

#[test]
fn test_generate_completions() -> anyhow::Result<()> {
    for &shell in Shell::value_variants() {
        Command::cargo_bin(env!("CARGO_PKG_NAME"))?
            .arg("generate-completions")
            .arg(shell.to_string())
            .assert()
            .success();
    }
    Ok(())
}
