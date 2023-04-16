# Triggering further workflow runs

Pull requests created by GitHub actions using the default `GITHUB_TOKEN` cannot
trigger other workflows.
If you have `on: pull_request` or `on: push` workflows acting as checks on pull
requests, they will not run.

> When you use the repository's `GITHUB_TOKEN` to perform tasks, events triggered
by the `GITHUB_TOKEN` will not create a new workflow run.
This prevents you from accidentally creating recursive workflow runs.
For example, if a workflow run pushes code using the repository's `GITHUB_TOKEN`,
a new workflow will not run even when the repository contains a workflow
configured to run when `push` events occur.

-- [GitHub Actions: Triggering a workflow from a workflow](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow)

Read [this](https://github.com/peter-evans/create-pull-request/blob/main/docs/concepts-guidelines.md#triggering-further-workflow-runs)
guide to learn about possible workarounds, such as setting a Personal Access
Token, a deploy key or a GitHub app.

If you want to use the release-plz logo for the GitHub app,
you can find it [here](../assets/robot_head.jpeg).
