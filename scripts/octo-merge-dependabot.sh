#!/bin/bash
set -euxo pipefail

################################################################################
# Merge all dependabot branches using fake octopus-merge strategy.
#
# Since git can't peprform octopus-merge if there are any conflicts, this script
# tries to achieve same result by -- essentially -- squashing merges.
# Script relies on a very limited set of changes done by dependabot. Regex magic
# assumes that merges are performed in alphabetical order, so in merge region it
# keeps all but last lines from HEAD and only last line from merged branch. This
# surprisingly works well for this scenario.
#
# Note:
#   - it creates fake_master branch,
#   - assumes neovim is installed with fugitive plugin (the '+Gwq' command),
#   - is eager to perform the Cargo.toml rewrites *without* ensuring any sanity,
#   - uses 'cargo update' to create fresh Cargo.lock file.
################################################################################

# Update remotes
git fetch --prune origin

# Prepare fake_master branch
git checkout master
git branch --no-track fake_master master
git checkout fake_master
git merge --ff-only origin/master

# Get hash of remote master & all dependabot branches
MASTER_HASH="$(git rev-parse HEAD)"
DEPENDABOT_BRANCHES="$(git branch -av | awk '/remotes\/origin\/dependabot/ {print $1}')"

# Create usual merges for each dependabot branch, one-by-one
for DEPENDABOT_BRANCH in $DEPENDABOT_BRANCHES; do
  echo ">>> Merging ${DEPENDABOT_BRANCH}"
  git branch --no-track fake_dependabot "${DEPENDABOT_BRANCH}"
  set +e; {
      git merge fake_dependabot
      MERGE_RES=$?
  }; set -e
  # For the first branch merge will succeed. Not for the others
  if [[ $MERGE_RES -ne 0 ]]; then
    # Cargo.lock will be recalculated later
    git rm Cargo.lock
    # Magic.
    nvim '+/^=\{7}$/-1,/^>\{7}/-2d' '+g/^[<>]\{7} /d' '+Gwq' Cargo.toml
    git commit --no-verify --no-edit
  fi
  git branch -d fake_dependabot
done

# Recreate Cargo.lock
cargo update
git add Cargo.lock
git commit --no-verify -m 'Cargo.lock'

# Go back to old master, but keep files & index
git reset --soft master
# Store octo-merged tree
NEW_TREE="$(git write-tree)"
# Store new commit
NEW_COMMIT="$(git commit-tree -m 'Dependabot updates' -p "${MASTER_HASH}" $(for B in $DEPENDABOT_BRANCHES; do echo "-p $(git rev-parse $B)"; done) "${NEW_TREE}")"
# Set branch pointer
git reset --soft "${NEW_COMMIT}"
