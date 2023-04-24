# Keep the action up-to-date

To receive Dependabot updates when a new version of a GitHub action (including release-plz)
is available, add the following to your `.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "github-actions"
    directory: "/"
    # Check for updates every Monday
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
```
