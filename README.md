# playground

A collection of reusable GitHub Actions for Project Zenith Systems.

## Available Actions

### [TODO to Issues](docs/actions/todo-to-issues.md)

Automatically creates GitHub issues from a `TODO.md` file when pushed to the main branch.

**Usage:**
```yaml
- uses: project-zenith-systems/playground/.github/actions/todo-to-issues@main
  with:
    github-token: ${{ secrets.GITHUB_TOKEN }}
```

See [full documentation](docs/actions/todo-to-issues.md) for details.
