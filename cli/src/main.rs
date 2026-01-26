#![warn(clippy::all)]

mod agents;
mod commands;
mod config;
mod generator;
mod github;
mod parser;
mod security;
mod skills_toml;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::audit::AuditOptions;
use commands::generate::GenerateOptions;
use commands::install::InstallOptions;
use commands::search::SearchOptions;
use commands::test::TestOptions;
use commands::verify::VerifyOptions;

#[derive(Parser)]
#[command(name = "cowork")]
#[command(about = "CLI tool for managing CoWork Skills", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize cowork: install built-in skills
    ///
    /// Examples:
    ///   cowork init                    # Install all built-in skills (global)
    ///   cowork init --local            # Install to project .claude/skills/
    ///   cowork init --list             # List available built-in skills
    ///   cowork init -s memory-skills   # Install specific skill only
    ///   cowork init --remove memory-filesystem  # Remove a skill
    Init {
        /// List available built-in skills without installing
        #[arg(long)]
        list: bool,

        /// Install specific skills only (can be repeated)
        #[arg(short = 's', long = "skill", value_name = "SKILL")]
        skills: Vec<String>,

        /// Force overwrite existing skills
        #[arg(short, long)]
        force: bool,

        /// Install to project .claude/skills/ instead of global ~/.claude/skills/
        #[arg(short = 'l', long)]
        local: bool,

        /// Remove specified skills (can be repeated)
        #[arg(short = 'r', long = "remove", value_name = "SKILL")]
        remove: Vec<String>,
    },

    /// List all available skills
    List {
        /// Filter by type: project, global, or all
        #[arg(short = 't', long, default_value = "all")]
        skill_type: String,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show current status and configuration
    Status,

    /// Check for configuration issues
    Doctor,

    /// Security audit of installed skills
    ///
    /// Scans skills for dangerous patterns, credential leaks, and prompt injection.
    ///
    /// Examples:
    ///   cowork audit                    # Audit all skills
    ///   cowork audit --verbose          # Show detailed findings
    ///   cowork audit -o report.md       # Save report to file
    ///   cowork audit --format json      # Output as JSON
    Audit {
        /// Scan global skills (~/.claude/skills/)
        #[arg(long, default_value = "true")]
        global: bool,

        /// Scan project skills (.claude/skills/)
        #[arg(long, default_value = "true")]
        project: bool,

        /// Scan installed plugins
        #[arg(long, default_value = "true")]
        plugins: bool,

        /// Output format (text, json, markdown)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Fix issues automatically (where possible)
        #[arg(long)]
        fix: bool,
    },

    /// Verify checksums of installed skills
    ///
    /// Validates skills against their recorded checksums in Skills.lock.
    ///
    /// Examples:
    ///   cowork verify                   # Verify all skills
    ///   cowork verify --update          # Update checksums
    ///   cowork verify rust-skills       # Verify specific skill
    Verify {
        /// Specific skill to verify (if None, verify all)
        skill: Option<String>,

        /// Update checksums in lockfile
        #[arg(short, long)]
        update: bool,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Install skills globally from GitHub or current project
    ///
    /// Examples:
    ///   cowork install                              # Install current project
    ///   cowork install user/repo                    # Install from GitHub
    ///   cowork install user/repo -s skill1 -s skill2  # Install specific skills
    ///   cowork install user/repo --plugin           # Install as plugin (full structure)
    ///   cowork install user/repo -a claude-code -a cursor  # Install to multiple agents
    ///   cowork install --list                       # List installed repos
    ///   cowork install --uninstall repo             # Uninstall a repo
    ///   cowork install user/repo --reinstall        # Force reinstall (remove + install)
    ///   cowork install user/repo --update           # Update to latest version
    ///   cowork install user/repo --use-add-skill   # Use add-skill backend
    Install {
        /// GitHub repository (user/repo) or full URL to install
        repo: Option<String>,

        /// Uninstall instead of install
        #[arg(short, long)]
        uninstall: bool,

        /// List installed repositories
        #[arg(long)]
        list: bool,

        /// Force reinstall (remove existing and install fresh)
        #[arg(long)]
        reinstall: bool,

        /// Update to latest version (git pull + reinstall)
        #[arg(long)]
        update: bool,

        /// Install specific skills by name (can be repeated)
        #[arg(short = 's', long = "skill", value_name = "SKILL")]
        skills: Vec<String>,

        /// Target specific agents (can be repeated)
        /// Available: amp, antigravity, claude-code, clawdbot, codex, cursor,
        /// droid, gemini-cli, github-copilot, goose, kilo, kiro-cli,
        /// opencode, roo, trae, windsurf
        #[arg(short = 'a', long = "agent", value_name = "AGENT")]
        agents: Vec<String>,

        /// Install as plugin (preserves entire repository structure)
        #[arg(long)]
        plugin: bool,

        /// Include additional directories alongside skills (can be repeated)
        #[arg(long = "include-dir", value_name = "DIR")]
        include_dirs: Vec<String>,

        /// Copy files instead of creating symlinks
        #[arg(long)]
        no_symlink: bool,

        /// Skip all confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,

        /// Use add-skill (npx) as backend instead of built-in installer
        #[arg(long)]
        use_add_skill: bool,

        /// Install to current project instead of global (~/.claude/skills/)
        /// Skills will be installed to .claude/skills/ or skills/ in current directory
        #[arg(short = 'l', long)]
        local: bool,
    },

    /// Generate skills from a GitHub repository or local directory
    ///
    /// Examples:
    ///   cowork generate user/repo                   # Generate from GitHub repo
    ///   cowork generate --path ./my-project        # Generate from local directory
    ///   cowork generate user/repo --lang rust      # Specify language(s)
    ///   cowork generate user/repo --llms-only      # Only generate llms.txt
    ///   cowork generate --from-llms ./llms.txt     # Generate from existing llms.txt
    ///   cowork generate user/repo -o ./output      # Specify output directory
    ///   cowork generate user/repo --ref v1.0.0     # Specify git ref
    Generate {
        /// GitHub repository (user/repo) or full URL
        repo: Option<String>,

        /// Generate from local directory path
        #[arg(long = "path", short = 'p', value_name = "DIR")]
        local_path: Option<PathBuf>,

        /// Generate skills from existing llms.txt file
        #[arg(long = "from-llms", value_name = "PATH")]
        from_llms: Option<PathBuf>,

        /// Languages to parse (rust, typescript, python)
        #[arg(short = 'l', long = "lang", value_name = "LANG")]
        languages: Vec<String>,

        /// Output directory
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<PathBuf>,

        /// Only generate llms.txt (no skills)
        #[arg(long)]
        llms_only: bool,

        /// Split modules into separate skills (default: true)
        #[arg(long, default_value = "true")]
        split_modules: bool,

        /// Target specific agents for installation (can be repeated)
        #[arg(short = 'a', long = "agent", value_name = "AGENT")]
        agents: Vec<String>,

        /// Git ref (branch, tag, or commit) to use
        #[arg(long = "ref", value_name = "REF")]
        git_ref: Option<String>,
    },

    /// Search GitHub for skill repositories
    ///
    /// Examples:
    ///   cowork search agent-skill                   # Search for agent-skill topic
    ///   cowork search rust-skills --verbose        # Search with details
    ///   cowork search tokio --limit 5              # Limit results
    Search {
        /// Search query (repository name, topic, or keyword)
        query: String,

        /// Search by topic instead of general search
        #[arg(short, long)]
        topic: bool,

        /// Maximum number of results
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Manage Claude Code marketplace plugins
    ///
    /// Examples:
    ///   cowork plugins list                         # List installed marketplace plugins
    ///   cowork plugins status                       # Show plugin system status
    ///   cowork plugins uninstall rust-skills        # Uninstall a plugin
    ///   cowork plugins enable rust-skills           # Enable a plugin
    Plugins {
        #[command(subcommand)]
        action: PluginsAction,
    },

    /// Generate trigger tests for installed skills
    ///
    /// Examples:
    ///   cowork test                                 # Show all skill triggers
    ///   cowork test --check-conflicts              # Check for trigger conflicts
    ///   cowork test -o triggers.md                 # Export to markdown file
    ///   cowork test --format json -o triggers.json # Export as JSON
    ///   cowork test triggers                       # List all triggers with skills
    ///   cowork test --global                       # Scan project + global skills
    ///   cowork test --plugins                      # Scan project + plugin skills
    ///   cowork test --all                          # Scan project + global + plugins
    ///   cowork test --path /custom/dir             # Scan custom directory
    ///   cowork test --run                          # Run actual trigger tests with claude
    ///   cowork test --run --limit 5                # Test max 5 triggers per skill
    Test {
        #[command(subcommand)]
        action: Option<TestAction>,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format (markdown, json, yaml)
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Check for trigger conflicts
        #[arg(long)]
        check_conflicts: bool,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,

        /// Scan global skills (~/.claude/skills/)
        #[arg(long)]
        global: bool,

        /// Scan specified directory
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,

        /// Scan installed plugins
        #[arg(long)]
        plugins: bool,

        /// Scan all: project + global + plugins
        #[arg(long)]
        all: bool,

        /// Run actual trigger tests using `claude -p`
        #[arg(long)]
        run: bool,

        /// Maximum triggers to test per skill (default: 3)
        #[arg(short = 'n', long, default_value = "3")]
        limit: usize,

        /// Filter skills by name pattern
        #[arg(long, value_name = "PATTERN")]
        filter: Option<String>,
    },
    /// Manage project-level skill configuration via skills.toml
    ///
    /// Examples:
    ///   cowork config init                           # Create skills.toml
    ///   cowork config show                           # Show current config
    ///   cowork config add rust ZhangHanDong/rust-skills  # Add dependency
    ///   cowork config install                        # Install dependencies
    ///   cowork config enable rust-core dora          # Enable skill groups
    ///   cowork config disable domain-fintech         # Disable skills
    ///   cowork config priority dora-router rust-router  # Set priority
    ///   cowork config apply                          # Generate SKILLS.md
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum TestAction {
    /// List all triggers with their skills
    Triggers,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Initialize skills.toml in the project
    ///
    /// Auto-detects installed plugins and skills from ~/.claude/ and .claude/
    /// and prompts for confirmation before adding them to the config.
    Init {
        /// Overwrite existing config
        #[arg(short, long)]
        force: bool,

        /// Skip auto-detection of installed plugins/skills
        #[arg(long)]
        no_detect: bool,
    },

    /// Show current skills.toml configuration
    Show,

    /// Add a skill dependency or plugin to skills.toml
    ///
    /// Examples:
    ///   cowork config add rust ZhangHanDong/rust-skills
    ///   cowork config add rust ZhangHanDong/rust-skills --local
    ///   cowork config add tokio user/tokio -s tokio-runtime -s tokio-sync
    ///   cowork config add makepad user/makepad-skills --plugin
    ///   cowork config add dora-dev /path/to/dora --dev
    Add {
        /// Dependency name (used as key in skills.toml)
        name: String,

        /// GitHub repo (user/repo) or local path
        source: String,

        /// Specific skills to install (can be repeated)
        #[arg(short = 's', long = "skill", value_name = "SKILL")]
        skills: Vec<String>,

        /// Git reference (branch, tag, commit)
        #[arg(long = "ref", value_name = "REF")]
        git_ref: Option<String>,

        /// Target agents (can be repeated)
        #[arg(short = 'a', long = "agent", value_name = "AGENT")]
        agents: Vec<String>,

        /// Install as plugin (preserves full repository structure)
        #[arg(long)]
        plugin: bool,

        /// Install to project local (.claude/skills/) instead of global
        #[arg(short = 'l', long)]
        local: bool,

        /// Add as disabled (installed but not enabled)
        #[arg(long)]
        disabled: bool,

        /// Add as development link (symlink for testing)
        #[arg(long)]
        dev: bool,
    },

    /// Remove a skill dependency from skills.toml
    Remove {
        /// Dependency name to remove
        name: String,
    },

    /// Install all dependencies from Skills.toml
    Install,

    /// Sync Skills.lock with Skills.toml (update enabled/disabled status)
    ///
    /// Use this after modifying enabled status in Skills.toml
    ///
    /// Examples:
    ///   cowork config sync              # Sync enabled status only
    ///   cowork config sync --update     # Also git pull remote repos
    Sync {
        /// Also update remote repos (git pull) and refresh versions
        #[arg(short, long)]
        update: bool,
    },

    /// Enable skills or groups
    Enable {
        /// Skill or group names to enable
        names: Vec<String>,
    },

    /// Disable skills or groups
    Disable {
        /// Skill or group names to disable
        names: Vec<String>,
    },

    /// Set skill priority for trigger conflicts
    Priority {
        /// Skills in priority order (highest first)
        skills: Vec<String>,
    },

    /// Override a trigger to use specific skill
    Override {
        /// Trigger keyword
        trigger: String,

        /// Target skill name
        skill: String,
    },

    /// Apply config and generate SKILLS.md
    Apply {
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List available skill groups
    Groups,

    /// Generate dynamic cowork-router based on installed plugins
    ///
    /// Creates a router skill that detects domains and routes to
    /// appropriate plugin routers based on keyword triggers.
    ///
    /// Examples:
    ///   cowork config router              # Generate router skill
    ///   cowork config router --hooks      # Also generate hooks.json
    ///   cowork config router --analyze    # AI-enhanced with three-layer intent
    ///   cowork config router --analyze --no-cache  # Force re-analysis
    Router {
        /// Also generate hooks.json for auto-triggering
        #[arg(long)]
        hooks: bool,

        /// Use AI analysis for intelligent three-layer routing
        #[arg(long)]
        analyze: bool,

        /// Ignore cached analysis and force re-analysis
        #[arg(long)]
        no_cache: bool,
    },
}

#[derive(Subcommand)]
enum PluginsAction {
    /// List all marketplace plugins installed via /plugin
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show plugin system status
    Status,

    /// Uninstall a marketplace plugin completely
    ///
    /// Removes the plugin from:
    /// - installed_plugins.json
    /// - settings.json enabledPlugins
    /// - Plugin cache directory
    /// - Skills symlinks
    Uninstall {
        /// Plugin ID or name (e.g., rust-skills or rust-skills@rust-skills)
        plugin_id: String,
    },

    /// Enable a marketplace plugin
    Enable {
        /// Plugin ID or name
        plugin_id: String,
    },

    /// Disable a marketplace plugin
    Disable {
        /// Plugin ID or name
        plugin_id: String,
    },

    /// List all marketplaces
    Marketplaces,

    /// Remove a marketplace (registry)
    RemoveMarketplace {
        /// Marketplace name (e.g., rust-skills, makepad-skills)
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { list, skills, force, local, remove } => {
            commands::init::execute(list, &skills, force, local, &remove)
        }
        Commands::List {
            skill_type,
            verbose,
        } => commands::list::execute(&skill_type, verbose),
        Commands::Status => commands::status::execute(),
        Commands::Doctor => commands::doctor::execute(),
        Commands::Audit {
            global,
            project,
            plugins,
            format,
            output,
            verbose,
            fix,
        } => {
            let options = AuditOptions {
                global,
                project,
                plugins,
                format,
                output,
                verbose,
                fix,
            };
            commands::audit::execute(options)
        }
        Commands::Verify {
            skill,
            update,
            verbose,
        } => {
            let options = VerifyOptions {
                skill,
                update,
                verbose,
            };
            commands::verify::execute(options)
        }
        Commands::Install {
            repo,
            uninstall,
            list,
            reinstall,
            update,
            skills,
            agents,
            plugin,
            include_dirs,
            no_symlink,
            yes,
            use_add_skill,
            local,
        } => {
            let options = InstallOptions {
                repo,
                uninstall,
                list,
                reinstall,
                update,
                skills,
                agents,
                plugin,
                include_dirs,
                no_symlink,
                yes,
                use_add_skill,
                local,
            };
            commands::install::execute(options)
        }
        Commands::Generate {
            repo,
            local_path,
            from_llms,
            languages,
            output,
            llms_only,
            split_modules,
            agents,
            git_ref,
        } => {
            let options = GenerateOptions {
                repo,
                local_path,
                from_llms,
                languages,
                output,
                llms_only,
                split_modules,
                agents,
                git_ref,
            };
            commands::generate::execute(options)
        }
        Commands::Search {
            query,
            topic,
            limit,
            verbose,
        } => {
            let options = SearchOptions {
                query,
                topic,
                limit,
                verbose,
            };
            commands::search::execute(options)
        }
        Commands::Plugins { action } => match action {
            PluginsAction::List { verbose } => commands::plugins::execute_list(verbose),
            PluginsAction::Status => commands::plugins::execute_status(),
            PluginsAction::Uninstall { plugin_id } => {
                commands::plugins::execute_uninstall_plugin(&plugin_id)
            }
            PluginsAction::Enable { plugin_id } => {
                commands::plugins::execute_enable_plugin(&plugin_id)
            }
            PluginsAction::Disable { plugin_id } => {
                commands::plugins::execute_disable_plugin(&plugin_id)
            }
            PluginsAction::Marketplaces => commands::plugins::execute_list_marketplaces(),
            PluginsAction::RemoveMarketplace { name } => {
                commands::plugins::execute_remove_marketplace(&name)
            }
        },
        Commands::Test {
            action,
            output,
            format,
            check_conflicts,
            verbose,
            global,
            path,
            plugins,
            all,
            run,
            limit,
            filter,
        } => {
            if let Some(TestAction::Triggers) = action {
                let options = TestOptions {
                    output: None,
                    format: "markdown".to_string(),
                    check_conflicts: false,
                    verbose: false,
                    global: global || all,
                    path,
                    plugins: plugins || all,
                    run: false,
                    limit: 3,
                    filter: None,
                };
                commands::test::execute_list_triggers(options)
            } else if run {
                let options = TestOptions {
                    output,
                    format,
                    check_conflicts,
                    verbose,
                    global: global || all,
                    path,
                    plugins: plugins || all,
                    run: true,
                    limit,
                    filter,
                };
                commands::test::execute_run_tests(options)
            } else {
                let options = TestOptions {
                    output,
                    format,
                    check_conflicts,
                    verbose,
                    global: global || all,
                    path,
                    plugins: plugins || all,
                    run: false,
                    limit,
                    filter,
                };
                commands::test::execute(options)
            }
        }
        Commands::Config { action } => {
            let project_root = std::env::current_dir()?;
            match action {
                ConfigAction::Init { force, no_detect } => {
                    commands::config::execute_init_with_options(&project_root, force, !no_detect)
                }
                ConfigAction::Show => {
                    commands::config::execute_show(&project_root)
                }
                ConfigAction::Add {
                    name,
                    source,
                    skills,
                    git_ref,
                    agents,
                    plugin,
                    local,
                    disabled,
                    dev,
                } => {
                    let options = commands::config::AddOptions {
                        skills,
                        git_ref,
                        agents,
                        plugin,
                        local,
                        disabled,
                        dev,
                    };
                    commands::config::execute_add(&project_root, &name, &source, options)
                }
                ConfigAction::Remove { name } => {
                    commands::config::execute_remove(&project_root, &name)
                }
                ConfigAction::Install => {
                    commands::config::execute_install_deps(&project_root)
                }
                ConfigAction::Sync { update } => {
                    commands::config::execute_sync(&project_root, update)
                }
                ConfigAction::Enable { names } => {
                    commands::config::execute_enable(&project_root, &names)
                }
                ConfigAction::Disable { names } => {
                    commands::config::execute_disable(&project_root, &names)
                }
                ConfigAction::Priority { skills } => {
                    commands::config::execute_priority(&project_root, &skills)
                }
                ConfigAction::Override { trigger, skill } => {
                    commands::config::execute_override(&project_root, &trigger, &skill)
                }
                ConfigAction::Apply { output } => {
                    commands::config::execute_apply(&project_root, output.as_deref())
                }
                ConfigAction::Groups => {
                    commands::config::execute_list_groups()
                }
                ConfigAction::Router { hooks, analyze, no_cache } => {
                    commands::config::execute_generate_router(&project_root, hooks, analyze, no_cache)
                }
            }
        }
    }
}
