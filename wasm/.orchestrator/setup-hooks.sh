#!/bin/bash
# Setup script to install hooks in all worktrees
#
# Usage:
#   ./setup-hooks.sh              # Install in all worktrees
#   ./setup-hooks.sh --list       # List all worktrees
#   ./setup-hooks.sh --dry-run    # Show what would be done

set -euo pipefail

# Find the parent repository
PARENT_REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ORCHESTRATOR_DIR="$PARENT_REPO/.orchestrator"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# List all worktrees
list_worktrees() {
    echo "=== Git Worktrees ==="
    git worktree list
    echo ""
}

# Install hooks in a worktree
install_hooks() {
    local worktree_path="$1"
    local dry_run="${2:-false}"

    echo "Installing hooks in: $worktree_path"

    # Create .claude directory
    if [[ "$dry_run" == "false" ]]; then
        mkdir -p "$worktree_path/.claude"
    fi

    # Create settings.json that references shared hooks
    local settings_file="$worktree_path/.claude/settings.json"
    local settings_content=$(cat <<'EOF'
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.orchestrator/shared-hooks/worker-stop.sh",
            "timeout": 30
          }
        ]
      }
    ],
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.orchestrator/shared-hooks/manager-check.sh",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
EOF
)

    if [[ "$dry_run" == "false" ]]; then
        echo "$settings_content" > "$settings_file"
        echo -e "${GREEN}✓${NC} Created $settings_file"
    else
        echo -e "${YELLOW}[dry-run]${NC} Would create $settings_file"
    fi
}

# Main execution
DRY_RUN=false
LIST_ONLY=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --list)
            LIST_ONLY=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--dry-run] [--list]"
            exit 1
            ;;
    esac
done

# List worktrees
list_worktrees

if [[ "$LIST_ONLY" == "true" ]]; then
    exit 0
fi

# Get all worktrees
WORKTREES=$(git worktree list | grep -v '\(bare\)' | awk '{print $1}')

echo "=== Installing Hooks ==="

if [[ "$DRY_RUN" == "true" ]]; then
    echo -e "${YELLOW}DRY RUN MODE - No changes will be made${NC}"
    echo ""
fi

# Install hooks in each worktree
for worktree in $WORKTREES; do
    if [[ -d "$worktree" ]]; then
        install_hooks "$worktree" "$DRY_RUN"
        echo ""
    fi
done

echo -e "${GREEN}Done!${NC}"
echo ""
echo "Hooks installed in all worktrees."
echo "Each worktree will use the shared hooks from:"
echo "  $ORCHESTRATOR_DIR/shared-hooks/"
