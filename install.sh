#!/bin/bash

# CKB Dev Skill Installer for Vibe Coding
#
# One-liner install (remote):
#   curl -fsSL https://raw.githubusercontent.com/gpBlockchain/ckb-dev-skills/main/install.sh | bash
#
# Local install:
#   ./install.sh [--project | --path <path>]

set -e

SKILL_NAME="ckb-dev"
REPO_URL="https://github.com/gpBlockchain/ckb-dev-skills.git"
CLONE_DIR="$HOME/.ckb-dev-skills"

# Detect if running from a local clone or remotely via curl|bash
if [ -n "${BASH_SOURCE[0]}" ] && [ -f "$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)/agents/ckb-dev-lead/SKILL.md" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    LOCAL_MODE=true
else
    LOCAL_MODE=false
fi

AGENTS_DIR=""
SHARED_DIR=""
SKILLS_DIR=""
COMMANDS_DIR=""
INSTALL_PATH="$HOME/.claude/skills/$SKILL_NAME"
MODE="personal"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --project)
            INSTALL_PATH=".claude/skills/$SKILL_NAME"
            MODE="project"
            shift
            ;;
        --path)
            INSTALL_PATH="$2"
            MODE="custom"
            shift 2
            ;;
        --update)
            if [ -d "$CLONE_DIR" ]; then
                echo "Updating CKB Dev Skills..."
                cd "$CLONE_DIR" && git pull
                echo "Updated. Re-run install to apply changes."
            else
                echo "Not installed yet. Run install first."
            fi
            exit 0
            ;;
        --uninstall)
            echo "Uninstalling CKB Dev Skills..."
            rm -rf "$HOME/.claude/skills/$SKILL_NAME"
            rm -rf "$CLONE_DIR"
            echo "Uninstalled."
            exit 0
            ;;
        -h|--help)
            echo "CKB Dev Skill Installer"
            echo ""
            echo "One-liner install (recommended):"
            echo "  curl -fsSL https://raw.githubusercontent.com/gpBlockchain/ckb-dev-skills/main/install.sh | bash"
            echo ""
            echo "Usage: ./install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --project     Install to current project (.claude/skills/$SKILL_NAME)"
            echo "  --path PATH   Install to custom path"
            echo "  --update      Update to the latest version"
            echo "  --uninstall   Remove CKB Dev Skills"
            echo "  -h, --help    Show this help message"
            echo ""
            echo "Default: Install to ~/.claude/skills/$SKILL_NAME"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# If running remotely, clone the repo first
if [ "$LOCAL_MODE" = false ]; then
    echo "🔗 Fetching CKB Dev Skills..."
    if [ -d "$CLONE_DIR" ]; then
        echo "   Updating existing clone..."
        cd "$CLONE_DIR" && git pull --quiet
    else
        git clone --quiet --depth 1 "$REPO_URL" "$CLONE_DIR"
    fi
    SCRIPT_DIR="$CLONE_DIR"
fi

AGENTS_DIR="$SCRIPT_DIR/agents"
SHARED_DIR="$SCRIPT_DIR/shared"
SKILLS_DIR="$SCRIPT_DIR/skills"
COMMANDS_DIR="$SCRIPT_DIR/commands"

# Check if source directories exist
if [ ! -d "$AGENTS_DIR" ]; then
    echo "Error: Agents directory '$AGENTS_DIR' not found"
    exit 1
fi

# Check if SKILL.md exists
if [ ! -f "$AGENTS_DIR/ckb-dev-lead/SKILL.md" ]; then
    echo "Error: SKILL.md not found in '$AGENTS_DIR/ckb-dev-lead'"
    exit 1
fi

# Create parent directory if needed
mkdir -p "$(dirname "$INSTALL_PATH")"

# Check if destination already exists
if [ -d "$INSTALL_PATH" ]; then
    echo "Warning: '$INSTALL_PATH' already exists"
    if [ -t 0 ]; then
        read -p "Overwrite? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Installation cancelled"
            exit 0
        fi
    else
        echo "   Overwriting (non-interactive mode)..."
    fi
    rm -rf "$INSTALL_PATH"
fi

# Copy skill files
echo "📦 Installing CKB Dev Skill..."
mkdir -p "$INSTALL_PATH"

# Copy agents and shared directories
cp -r "$AGENTS_DIR" "$INSTALL_PATH/agents"
if [ -d "$SHARED_DIR" ]; then
    cp -r "$SHARED_DIR" "$INSTALL_PATH/shared"
fi

# Copy skills (brainstorming, etc.)
if [ -d "$SKILLS_DIR" ]; then
    cp -r "$SKILLS_DIR" "$INSTALL_PATH/skills"
fi

# Copy commands (slash commands for agent interaction)
if [ -d "$COMMANDS_DIR" ]; then
    cp -r "$COMMANDS_DIR" "$INSTALL_PATH/commands"
fi

echo ""
echo "✅ Successfully installed to: $INSTALL_PATH"
echo ""
echo "📂 Installed files:"
find "$INSTALL_PATH" -type f -name "*.md" | sort | while read -r file; do
    echo "  - ${file#$INSTALL_PATH/}"
done
echo ""
echo "🚀 Available commands (Claude Code slash commands):"
echo "  /ckb-dev-lead   — Talk to the Team Lead (routes to the right agent)"
echo "  /brainstorm     — Interactive Q&A to design a new CKB project"
echo "  /ckb-core       — Talk to the Core Agent (Cell Model, transactions)"
echo "  /ckb-contract   — Talk to the Contract Agent (Rust Scripts, testing)"
echo "  /ckb-dapp       — Talk to the DApp Agent (CCC SDK, React, wallets)"
echo "  /ckb-fiber      — Talk to the Fiber Agent (payment channels)"
echo ""
echo "The skill is now available in Claude Code."
echo "Try: /brainstorm to start a new CKB project!"
echo ""
echo "📖 Update:    ./install.sh --update  (or re-run the curl one-liner)"
echo "🗑️  Uninstall: ./install.sh --uninstall"
