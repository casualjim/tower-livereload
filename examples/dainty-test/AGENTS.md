# Agent Coding Guidelines

> **Important:** Prefer the `mise` tasks for installs, builds, tests, and formatting. Only use raw toolchain commands when no `mise` wrapper exists, and call that out explicitly.
>
> **CRITICAL: Use Language Servers for all code navigation!** Do NOT use grep/find/rg or manual file browsing - language servers provide accurate, fast, type-aware navigation. See the "Code Navigation" section for detailed commands.

## Code Navigation

**IMPORTANT: Always use language servers for code navigation!** Choose the right tool based on the file type:

### For Rust Code - Use Rust LSP

The Rust LSP (`mcp__rust-lsp__*` tools) is your primary tool for navigating Rust code:
- Finding symbols and definitions
- Navigating to references
- Getting function signatures and documentation
- Understanding code structure
- Finding implementations and usages

**Prefer Rust LSP over:** grep/find/rg, manual file browsing, or any other navigation method!

#### Rust LSP Commands

```bash
# Get file structure and symbols
mcp__rust-lsp__outline <file_path>

# Search for symbols across the codebase
mcp__rust-lsp__search <query>

# Find all references to a symbol
mcp__rust-lsp__references <file_path> <line> <character>

# Get detailed info about a symbol at cursor position
mcp__rust-lsp__inspect <file_path> <line> <character>

# Get code completions at a position
mcp__rust-lsp__completion <file_path> <line> <character>

# Rename a symbol across the codebase
mcp__rust-lsp__rename <file_path> <line> <character> <new_name>

# Get diagnostics (errors/warnings) for a file
mcp__rust-lsp__diagnostics <file_path>
```

#### Rust Navigation Examples

```bash
# Find all AppState references
mcp__rust-lsp__search "AppState"

# Explore the main application structure
mcp__rust-lsp__outline "src/lib.rs"

# Find all references to a function
mcp__rust-lsp__references "src/app.rs" 16 1

# Inspect a function to get its documentation
mcp__rust-lsp__inspect "src/routes/pages.rs" 42 10

# Get completions for method calls
mcp__rust-lsp__completion "src/routes/mod.rs" 20 15
```

### For TypeScript/JavaScript, CSS, and HTML - Use Semantic Search

For non-Rust code (TypeScript, CSS, HTML templates in Rust files), use **semantic search** as your primary navigation tool:

```bash
# Use semantic_search for finding code patterns, functions, components
semantic_search("function that handles layout persistence")
semantic_search("CSS styles for sidebar")
semantic_search("HTMX components for navigation")
semantic_search("Tailwind classes for responsive layout")
```

**When to use semantic search:**
- Finding TypeScript/JavaScript functions and patterns
- Locating CSS styles and Tailwind configurations
- Understanding HTML templates embedded in Rust code
- Discovering component patterns across the codebase
- Finding usage examples of libraries or patterns

**Benefits of semantic search for non-Rust code:**
- Understands intent and context, not just literal text matches
- Finds related code even with different naming
- Works across embedded code (HTML in Rust, inline styles)
- Discovers patterns and usage examples
- Fast and comprehensive

### Fallback: Targeted Grep Search

Only use `grep_search` when you need exact string matching or regex patterns:

```bash
# Find specific class names
grep_search("flex-1", false, "**/*.rs")

# Find specific function calls
grep_search("sqlx::query", false, "src/**/*.rs")

# Find configuration keys
grep_search("DATABASE_URL", false)
```

**Use grep_search sparingly** - prefer language servers (for Rust) or semantic search (for other languages) first.

### Navigation Strategy

1. **For Rust code**: Always use Rust LSP first (`mcp__rust-lsp__*`)
2. **For TypeScript/CSS/HTML**: Use semantic search first
3. **For exact string/regex matching**: Use grep_search as a last resort
4. **Never**: Use manual file browsing or blind searching

### Why Use Language Servers and Semantic Search?

- **Accurate**: Understands language semantics and type systems
- **Fast**: Instant navigation without scanning files
- **Context-aware**: Knows about imports, traits, generics, scopes
- **Complete**: Shows parameters, return types, documentation
- **IDE-quality**: Same experience as modern IDEs
- **Intent-based**: Semantic search understands what you're looking for

**Remember: When you need to understand or navigate code, reach for the appropriate language server or semantic search first!**

## Build, Test, and Development Commands

**CRITICAL: Always use `mise` tasks for all builds, tests, formatting, and code generation!**

Never invoke toolchain commands directly unless explicitly told to do so. The mise tasks handle dependencies, environment setup, and proper execution order.

### Available Mise Tasks

View all available tasks:
```bash
mise tasks ls
```

#### Build Commands
- `mise build` - Production release build (runs tests first)
- `mise build:debug` - Debug build for development
- `mise build:dist` - Create distribution package
- `mise build:docker` - Build Docker container image

**DO NOT use:** `cargo build` directly - always use the mise tasks!

#### Test Commands
- `mise test` - Run ALL tests (Rust unit + E2E)
- `mise test:rust` - Run Rust unit tests with nextest
- `mise test:e2e` - Run Playwright E2E tests

**DO NOT use:** `cargo test`, `cargo nextest`, `bun test`, or `playwright test` directly!

#### Code Generation
- `mise generate:sql` - Generate Rust code from SQL queries

**DO NOT use:** `sqlc generate` directly!

#### Development Server
- `mise dev` - Start development server with hot-reload, TLS, and systemfd

**DO NOT use:** `cargo run`, `cargo watch`, or `systemfd` directly!

#### Formatting
- `mise format` - Format all code (Rust, TypeScript, CSS)

**DO NOT use:** `cargo fmt`, `biome format`, or any other formatter directly!

#### Hooks

Mise is configured with automatic hooks:
- **postinstall** - Automatically runs `bun install` after `mise install` completes

**DO NOT run:** `bun install` manually - it's automatically handled by the mise postinstall hook!

### Why Use Mise Tasks?

1. **Dependency Management**: Tasks automatically run prerequisites (e.g., `generate:sql` before builds)
2. **Environment Setup**: Correct environment variables and database URLs
3. **Consistency**: Same commands work for all developers and CI/CD
4. **Caching**: Mise tracks sources/outputs for intelligent rebuilds
5. **Orchestration**: Complex multi-step processes (E2E tests, Docker builds) handled correctly

**For code navigation and understanding, use the appropriate language server or semantic search!** See the "Code Navigation" section for detailed commands.

## Code Style & Formatting
- Rust:
  - Use `eyre::Result` for error handling, `thiserror` for domain errors
  - No `unwrap()` or `expect()` in public APIs
  - Async streaming first - avoid `collect()` patterns
  - Imports: Group std/core, external crates, and internal modules separately
  - Formatting: run `mise format`; never invoke `cargo fmt` directly
  - Strict error handling - fail spectacularly, don't swallow errors
- TypeScript:
  - Strict mode with no `any` or `unknown`
  - Bun package manager
  - Double quotes for strings
- General:
  - 2-space indentation (except Python which uses 4)
  - LF line endings with final newline
  - Trim trailing whitespace
  - UTF-8 encoding

## Naming Conventions
- Rust: snake_case for variables/functions, PascalCase for types
- TypeScript: camelCase for variables/functions, PascalCase for types
- Files: snake_case for Rust, camelCase for TypeScript

## Error Handling
- Rust: Use `eyre::Result` for function returns, `thiserror` for domain-specific errors
- TypeScript: Proper error catching and handling without swallowing
- Never ignore errors - propagate or handle explicitly



## Commit Messages
- Write clear, descriptive commit messages in plain English
- Do NOT use conventional commits, semantic commits, or any commit prefixes (no "feat:", "fix:", "refactor:", etc.)
- Focus on WHAT changed and WHY, not the type of change
- First line should be a clear summary (50-72 chars recommended)
- Use the body for detailed explanation if needed
- Reference issue IDs when relevant (e.g., "Closes: slipstream-24")

Good examples:
- "Split search into dedicated Searcher service"
- "Add reranking provider for DeepInfra Qwen3-Reranker"
- "Fix flaky test by increasing tolerance for timing variance"

Bad examples:
- "refactor(embedding): Split search into dedicated Searcher service"
- "feat: add reranking provider"
- "fix: flaky test"

## References
This file combines repository guidelines with specific agent instructions for working with the codebase effectively.

**Final Reminder: When working with this codebase, always reach for the appropriate language server or semantic search first for navigation and understanding code structure!**

- For Rust: Use the `mcp__rust-lsp__*` tools for IDE-quality code analysis
- For TypeScript/CSS/HTML/Tailwind: Use `semantic_search` to understand intent and find patterns
- The language servers and semantic search understand your code better than any text-based search tool
