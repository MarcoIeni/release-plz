/// Configuration for version incrementation rules.
#[derive(Debug)]
pub struct NextVersionConfig {
    /// Indicates whether uncontrolled minor version bumps are enabled.
    /// When false, minor version increments are not performed and they
    /// should be manually done to signal higher API stability to the user.
    ///
    /// Default value is `false`.
    pub uncontrolled_minor_bump: bool,

    /// Indicates whether initial major version increments are enabled.
    /// When true, allows for a major version increment from 0 to 1,
    /// indicating a breaking change in the API.
    ///
    /// Default value is `false`.
    pub initial_major_increment: bool,
}

impl Default for NextVersionConfig {
    fn default() -> Self {
        NextVersionConfig {
            uncontrolled_minor_bump: false,
            initial_major_increment: false,
        }
    }
}
