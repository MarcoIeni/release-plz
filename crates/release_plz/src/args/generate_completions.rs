use clap::IntoApp;
use clap_complete::Shell;

#[derive(clap::Parser, Debug)]
pub struct GenerateCompletions {
    /// Shell option
    #[clap(default_value = "bash")]
    shell: Shell,
}

impl GenerateCompletions {
    pub fn print(&self) {
        clap_complete::generate(
            self.shell,
            &mut super::CliArgs::command(),
            "release-plz",
            &mut std::io::stdout(),
        );
    }
}
