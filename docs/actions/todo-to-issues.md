# TODO to Issues Action

Automatically creates GitHub issues from a `TODO.md` file when pushed to the main branch.

## Features

- Automatically creates GitHub issues from TODO items
- Supports optional descriptions for each TODO
- Deletes TODO.md after processing to avoid duplicates
- Prevents concurrent runs to avoid duplicate issues
- Simple and intuitive TODO.md format

## TODO.md Format

The format is simple and intuitive:

```markdown
## Issue Title

Optional description for the issue.
Can span multiple lines.

## Another Issue Title

Another optional description.

## Issue Without Description
```

Each TODO item starts with `## ` followed by the title. Any text between the title and the next `## ` is treated as the issue description (optional).

## Usage

To use this action in your repositories:

1. Create a `.github/workflows/process-todos.yml` file:

```yaml
name: Process TODOs

on:
  push:
    branches:
      - main
    paths:
      - 'TODO.md'

# Prevent concurrent runs to avoid duplicate issues
concurrency:
  group: process-todos
  cancel-in-progress: false

jobs:
  create-issues:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      issues: write
    
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Process TODO file
        uses: project-zenith-systems/playground/.github/actions/todo-to-issues@main
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}
```

2. Create a `TODO.md` file in your repository root with your TODO items
3. Push to main branch
4. The action will create issues and delete the TODO.md file

## Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `github-token` | GitHub token for creating issues | Yes | `${{ github.token }}` |
| `todo-file` | Path to the TODO file | No | `TODO.md` |

## Example TODO.md

See [TODO.md.example](../../.github/actions/todo-to-issues/TODO.md.example) for a complete example.

```markdown
## Add user authentication

Implement OAuth2 authentication flow with support for:
- Google login
- GitHub login
- Email/password fallback

## Improve error handling

Add better error messages and logging throughout the application.

## Update documentation
```

This will create 3 issues with the titles and descriptions as specified.

## How It Works

1. When you push a TODO.md file to the main branch
2. The workflow triggers and checks for the TODO.md file
3. It parses the file looking for `## ` headings as issue titles
4. Text between headings becomes the issue description (optional)
5. Issues are created via the GitHub API
6. The TODO.md file is deleted with a commit
7. Concurrency control prevents duplicate runs

## Notes

- The action uses a simple `## Title` format for ease of use
- Descriptions are optional and can include blank lines, special characters, etc.
- The TODO.md file is automatically deleted after processing
- Concurrency control prevents race conditions and duplicate issues
- The action is reusable and can be used in any repository
