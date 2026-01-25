#!/bin/bash
# Sync skills from all sub-plugins and external sources via symlinks
# Run this after adding new sub-plugins, external skills, or updating submodules
#
# NOTE: Prefer using the Rust CLI tool instead:
#   cowork sync
#
# This script is kept for backward compatibility.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SKILLS_DIR="$SCRIPT_DIR/skills"
PLUGINS_DIR="$SCRIPT_DIR/plugins"
EXTERNAL_DIR="$SCRIPT_DIR/external"

echo "Syncing skills from all sources..."

# Function to create symlinks for a source directory
sync_source() {
    local source_type="$1"    # "plugins" or "external"
    local source_name="$2"    # plugin/external name
    local source_dir="$3"     # full path to source directory

    # Resolve symlinks if necessary
    if [ -L "$source_dir" ]; then
        source_dir=$(readlink -f "$source_dir" 2>/dev/null || readlink "$source_dir")
    fi

    local skills_path="$source_dir/skills"

    if [ ! -d "$skills_path" ]; then
        echo "  Skipping $source_name (no skills directory)"
        return
    fi

    echo "  Processing $source_name..."

    for skill_dir in "$skills_path"/*/; do
        if [ -d "$skill_dir" ]; then
            local skill_name=$(basename "$skill_dir")

            # Skip hidden and special directories
            if [[ "$skill_name" == .* ]] || [[ "$skill_name" == _* ]]; then
                continue
            fi

            local target="$SKILLS_DIR/$skill_name"
            local relative_path="../$source_type/$source_name/skills/$skill_name"

            if [ -L "$target" ]; then
                # Symlink exists, check if it points to the same location
                current=$(readlink "$target")
                if [ "$current" = "$relative_path" ]; then
                    echo "    ✓ $skill_name (unchanged)"
                else
                    echo "    ⚠ $skill_name (conflict: $current)"
                fi
            elif [ -e "$target" ]; then
                echo "    ⚠ $skill_name (exists, not a symlink)"
            else
                ln -s "$relative_path" "$target"
                echo "    + $skill_name (created)"
            fi
        fi
    done
}

# Sync from plugins/
echo ""
echo "Plugins:"
if [ -d "$PLUGINS_DIR" ]; then
    for plugin_dir in "$PLUGINS_DIR"/*/; do
        if [ -d "$plugin_dir" ] || [ -L "$plugin_dir" ]; then
            plugin_name=$(basename "$plugin_dir")
            [[ "$plugin_name" == .* ]] && continue
            sync_source "plugins" "$plugin_name" "$plugin_dir"
        fi
    done
else
    echo "  (no plugins/ directory)"
fi

# Sync from external/
echo ""
echo "External:"
if [ -d "$EXTERNAL_DIR" ]; then
    for external_dir in "$EXTERNAL_DIR"/*/; do
        if [ -d "$external_dir" ] || [ -L "$external_dir" ]; then
            external_name=$(basename "$external_dir")
            [[ "$external_name" == .* ]] && continue
            sync_source "external" "$external_name" "$external_dir"
        fi
    done
else
    echo "  (no external/ directory)"
fi

echo ""
echo "Done! Skills directory contents:"
ls -la "$SKILLS_DIR" | grep -E "^l|^d"
echo ""
echo "Total: $(ls -1 "$SKILLS_DIR" | grep -v "^_" | wc -l | tr -d ' ') skills"
