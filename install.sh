#!/bin/bash

# CKB Dev Skill Installer for Vibe Coding
# Usage: ./install.sh [--project | --path <path>]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_NAME="ckb-dev"
SOURCE_DIR="$SCRIPT_DIR/skill"
AGENTS_DIR="$SCRIPT_DIR/agents"
SHARED_DIR="$SCRIPT_DIR/shared"

# Default to personal installation
INSTALL_PATH="$HOME/.claude/skills/$SKILL_NAME"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --project)
            INSTALL_PATH=".claude/skills/$SKILL_NAME"
            shift
            ;;
        --path)
            INSTALL_PATH="$2"
            shift 2
            ;;
        -h|--help)
            echo "CKB Dev Skill Installer"
            echo ""
            echo "Usage: ./install.sh [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --project     Install to current project (.claude/skills/$SKILL_NAME)"
            echo "  --path PATH   Install to custom path"
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

# Check if source directories exist
if [ ! -d "$SOURCE_DIR" ]; then
    echo "Error: Source directory '$SOURCE_DIR' not found"
    exit 1
fi

if [ ! -d "$AGENTS_DIR" ]; then
    echo "Error: Agents directory '$AGENTS_DIR' not found"
    exit 1
fi

# Check if SKILL.md exists
if [ ! -f "$SOURCE_DIR/SKILL.md" ]; then
    echo "Error: SKILL.md not found in '$SOURCE_DIR'"
    exit 1
fi

# Create parent directory if needed
mkdir -p "$(dirname "$INSTALL_PATH")"

# Check if destination already exists
if [ -d "$INSTALL_PATH" ]; then
    echo "Warning: '$INSTALL_PATH' already exists"
    read -p "Overwrite? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled"
        exit 0
    fi
    rm -rf "$INSTALL_PATH"
fi

# Copy skill files
echo "Installing CKB Dev Skill..."
cp -r "$SOURCE_DIR" "$INSTALL_PATH"

# Copy agents and shared directories
cp -r "$AGENTS_DIR" "$INSTALL_PATH/agents"
if [ -d "$SHARED_DIR" ]; then
    cp -r "$SHARED_DIR" "$INSTALL_PATH/shared"
fi

echo ""
echo "Successfully installed to: $INSTALL_PATH"
echo ""
echo "Installed files:"
find "$INSTALL_PATH" -type f -name "*.md" | sort | while read -r file; do
    # Show relative path from install path
    echo "  - ${file#$INSTALL_PATH/}"
done
echo ""
echo "The skill is now available in Claude Code."
echo "Try asking about CKB development to activate it!"
echo ""
echo "Using another AI agent? Point it at this folder (or link/symlink it):"
echo "  $INSTALL_PATH"
