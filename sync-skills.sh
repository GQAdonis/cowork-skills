#!/bin/bash
# Sync skills from all sub-plugins via symlinks
# Run this after adding new sub-plugins or updating submodules

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SKILLS_DIR="$SCRIPT_DIR/skills"
PLUGINS_DIR="$SCRIPT_DIR/plugins"

echo "Syncing skills from sub-plugins..."

# Function to create symlinks for a plugin
sync_plugin() {
    local plugin_name="$1"
    local plugin_skills="$PLUGINS_DIR/$plugin_name/skills"

    if [ ! -d "$plugin_skills" ]; then
        echo "  Skipping $plugin_name (no skills directory)"
        return
    fi

    echo "  Processing $plugin_name..."

    for skill_dir in "$plugin_skills"/*/; do
        if [ -d "$skill_dir" ]; then
            local skill_name=$(basename "$skill_dir")
            local target="$SKILLS_DIR/$skill_name"
            local relative_path="../plugins/$plugin_name/skills/$skill_name"

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

# Sync each plugin
for plugin_dir in "$PLUGINS_DIR"/*/; do
    if [ -d "$plugin_dir" ]; then
        plugin_name=$(basename "$plugin_dir")
        sync_plugin "$plugin_name"
    fi
done

echo ""
echo "Done! Skills directory contents:"
ls -la "$SKILLS_DIR" | grep -E "^l|^d"
