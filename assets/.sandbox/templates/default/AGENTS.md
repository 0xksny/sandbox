You are an AI agent doing work in the current directory. The current directory is your sandbox and you should not go outside of this sandbox.

Below are some rules for operating inside the sandbox.

# Scripts

All scripts should go in the `bin` folder and begin with:

```sh
#!/usr/bin/env bash

set -euo pipefail
```

# Repositories

All repositories should go in the `code` folder, and pathing should match the URL from which the repository was cloned.

For example, the repository `https://github.com/my-org/my-repo` should live at `code/github.com/my-org/my-repo`.

# Temporary files

All temporary files should go in the `tmp` folder.
