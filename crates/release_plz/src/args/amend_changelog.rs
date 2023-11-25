use std::env;
use git_cmd::Repo;
use clap::builder::NonEmptyStringValueParser;

#[derive(clap::Parser, Debug)]
pub struct AmendChangelog {
    /// Git token used to create the pull request.
    #[arg(long, value_parser = NonEmptyStringValueParser::new(), visible_alias = "github-token", env)]
    git_token: String,
}

impl AmendChangelog {
    pub fn edit_changelog(&self) -> anyhow::Result<()>{
        let repo = Repo::new(env::current_dir()?)?;

        repo.is_clean().unwrap();




        Ok(())
    }
}
