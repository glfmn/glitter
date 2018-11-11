#!/usr/bin/env sh
# pre-commit.sh

BRANCH_NAME=$(git branch | grep '*' | sed 's/* //')
STASH_NAME="pre-commit-$BRANCH_NAME-$(date +%s)"
RED='\033[1;31m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
NC='\033[0m'
BOLD='\033[1m'

# Check if commit is on a rebase, if not proceed as usual
if [ $BRANCH_NAME != '(no branch)' ]
then
    stash=0
    # Check to make sure commit isn't empty
    if git diff-index --quiet HEAD --; then
        echo "${RED}You've tried to commit an empty commit${NC}"
        echo "\tMake sure to add your changes with 'git add'"
    else
        # Stash all changes in the working directory so we test only commit files
        if git stash save -u -k -q $STASH_NAME; then
            echo "${YELLOW}Stashed changes as:${NC} ${STASH_NAME}\n\n"
            stash=1
        fi
    fi

    echo "${GREEN} Testing commit\n\n${NC}"

    cargo doc --no-deps &&
    cargo build &&
    cargo test --all &&
    # Capture exit code from tests
    status=$?

    for crate in $(find $(pwd) -name Cargo.toml); do
      (
        echo >&2 "\tRunning tests for ${GREEN}${crate}${NC}:"
        export RUST_BACKTRACE=1
        cargo test --manifest-path=${crate}
      ) || status=1
    done

    # Revert stash if changes were stashed to restor working directory files
    if [ "$stash" -eq 1 ]
    then
        if git stash pop -q; then
            echo "\n\n${GREEN}Reverted stash command${NC}"
        else
            echo "\n\n${RED}Unable to revert stash command${NC}"
        fi
    fi

    # Inform user of build failure
    if [ "$status" -ne "0" ]
    then
        echo "${RED}Build failed:${NC} if you still want to commit use ${BOLD}'--no-verify'${NC}"
    fi

    # Exit with exit code from tests, so if they fail, prevent commit
    exit $status
else
    # Tests were skipped for rebase, inform user and exit zero
    echo "${YELLOW}Skipping tests on branchless commit${NC}"
    exit 0
fi
