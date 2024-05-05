pub const PACKAGE_VAR: &str = "package";
pub const VERSION_VAR: &str = "version";
pub const CHANGELOG_VAR: &str = "changelog";

pub fn tera_var(var_name: &str) -> String {
    format!("{{{{ {var_name} }}}}")
}

pub fn release_body_from_template(
    package_name: &str,
    version: &str,
    changelog: &str,
    body_template: Option<&str>,
) -> String {
    let mut tera = tera::Tera::default();
    let mut context = tera_context(package_name, version);
    context.insert(CHANGELOG_VAR, changelog);

    let default_body_template = tera_var(CHANGELOG_VAR);
    let body_template = body_template.unwrap_or(&default_body_template);

    render_template(&mut tera, body_template, &context, "release_body")
}

pub fn render_template(
    tera: &mut tera::Tera,
    template: &str,
    context: &tera::Context,
    template_name: &str,
) -> String {
    tera.add_raw_template(template_name, template)
        .expect("failed to add release_body raw template");

    tera.render(template_name, context)
        .unwrap_or_else(|e| panic!("failed to render {template_name}: {e}"))
}

pub fn tera_context(package_name: &str, version: &str) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert(PACKAGE_VAR, package_name);
    context.insert(VERSION_VAR, version);
    context
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_body_template_is_rendered() {
        let body = release_body_from_template("my_package", "0.1.0", "my changes", None);
        assert_eq!(body, "my changes");
    }
}
