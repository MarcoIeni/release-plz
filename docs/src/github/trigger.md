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

## Workarounds to trigger further workflow runs

Release-plz works fine without triggering further workflow runs.
However, if you want to run CI checks on the release PR,
or if you want to trigger another workflow after release-plz pushes
a tag or creates a release, you can use one of these workarounds:

- To run `on: pull_request` workflows, manually close and reopen the release pull request.

- Use a `repo` scoped
  [Personal Access Token (PAT)](https://docs.github.com/en/github/authenticating-to-github/creating-a-personal-access-token)
  created on an account that has write access to the repository that pull requests are being
  created in. This is the standard workaround and
  [recommended by GitHub](https://docs.github.com/en/actions/using-workflows/triggering-a-workflow#triggering-a-workflow-from-a-workflow).
  Note that the account that owns the PAT will be the author of the release pull request.

- Use [SSH (deploy keys)](#push-using-ssh-deploy-keys) to push the pull request branch.
  Note that this method will only trigger `on: push` workflows.

- Use a
  [GitHub App to generate a token](https://github.com/peter-evans/create-pull-request/blob/main/docs/concepts-guidelines.md#authenticating-with-github-app-generated-tokens)
  that can be used with this action.
  If you want to use the release-plz logo for the GitHub app, you can find it [here](../assets/robot_head.jpeg).

## Credits

This section is inspired by
[this](https://github.com/peter-evans/create-pull-request/blob/main/docs/concepts-guidelines.md#triggering-further-workflow-runs)
guide.
