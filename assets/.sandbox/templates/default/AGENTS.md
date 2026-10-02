You are an AI agent doing work in the current directory. The current directory is your sandbox and you should not go outside of this sandbox.

Below are some rules for operating inside the sandbox.

# Scripts

All scripts should go in the `bin` folder and begin with:

```sh
#!/usr/bin/env bash

set -euo pipefail
```

# Repositories

All repositories use worktrees, with the main worktrees and linked worktrees living at specific paths.

Main worktrees live in the `code` folder of the user's home directory (i.e. `~/code`), and pathing matches the URL from which the repository was cloned. For example, the main worktree for the repository `https://github.com/my-org/my-repo` lives at `~/code/github.com/my-org/my-repo`.

Linked worktrees use the same `code` folder and pathing rules, but live in the sandbox itself. Using the same example as before, the linked worktree lives at `code/github.com/my-org/my-repo` inside the sandbox.

To create a linked worktree, run the following or an equivalent set of commands (again using the same example as before):

```bash
# Capture the sandbox path
sandbox="$(pwd)"

# Move to the main worktree
cd ~/code/github.com/my-org/my-repo

# Create the linked worktree
git worktree add "${sandbox}/code/github.com/my-org/my-repo"

# Return to the sandbox
cd "${sandbox}"
```

# Temporary files

All temporary files should go in the `tmp` folder.
