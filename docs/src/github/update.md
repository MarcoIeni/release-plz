# Keep the action up-to-date

To receive Dependabot updates when a new version of a GitHub action (including release-plz)
is available, add the following to your `/.github/dependabot.yml` file:

```yaml
version: 2
updates:
  - package-ecosystem: "github-actions"
    directory: "/"
    # Check for updates every Monday
    schedule:
      interval: "weekly"
```

Learn more in GitHub
[docs](https://docs.github.com/en/code-security/dependabot/working-with-dependabot/keeping-your-actions-up-to-date-with-dependabot).
