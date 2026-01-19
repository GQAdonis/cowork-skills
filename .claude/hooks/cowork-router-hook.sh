#!/bin/bash
# CoWork Skills - Domain Router Hook
# Detects user input and routes to the appropriate domain skill

PROMPT="$CLAUDE_USER_PROMPT"

# ============================================
# Domain Detection Keywords
# ============================================

# Makepad UI Keywords (highest priority for UI-specific questions)
MAKEPAD_KEYWORDS="makepad|widget|view|live_design|Draw2d|DrawQuad|LiveHook|WidgetRef|WidgetAction"

# Dora Robotics Keywords
DORA_KEYWORDS="dora|dora-rs|node|operator|dataflow|robot|sensor|actuator|ROS|SLAM"

# Rust Core Keywords (fallback for general Rust questions)
RUST_KEYWORDS="rust|cargo|crate|rustc|Cargo\.toml|E0[0-9]{3}|borrow|ownership|lifetime|move|clone|Arc|Rc|RefCell|async|await|tokio|Send|Sync|trait|generic|impl|macro"

# ============================================
# Domain Routing Logic
# ============================================

# Check for Makepad UI domain
if echo "$PROMPT" | grep -qiE "$MAKEPAD_KEYWORDS"; then
    # Also check if it's a Rust-level question within Makepad context
    if echo "$PROMPT" | grep -qiE "E0[0-9]{3}|borrow|ownership|lifetime"; then
        # Cross-domain: UI + Rust mechanics
        echo "LOAD_SKILL: cowork-router"
        echo "CROSS_DOMAIN: makepad + rust"
    else
        echo "LOAD_SKILL: makepad-router"
    fi
    exit 0
fi

# Check for Dora Robotics domain
if echo "$PROMPT" | grep -qiE "$DORA_KEYWORDS"; then
    if echo "$PROMPT" | grep -qiE "E0[0-9]{3}|borrow|ownership|lifetime"; then
        # Cross-domain: Robotics + Rust mechanics
        echo "LOAD_SKILL: cowork-router"
        echo "CROSS_DOMAIN: dora + rust"
    else
        echo "LOAD_SKILL: dora-router"
    fi
    exit 0
fi

# Check for general Rust questions
if echo "$PROMPT" | grep -qiE "$RUST_KEYWORDS"; then
    echo "LOAD_SKILL: rust-router"
    exit 0
fi

# No domain detected - let Claude handle normally
exit 0
