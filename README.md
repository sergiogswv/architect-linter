# Architect Linter

<p align="center">
  <img src="./public/architect-linter-banner.png" alt="Architect Linter Banner" width="100%">
</p>

<p align="center">
  <img src="https://img.shields.io/badge/version-3.1.0-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/rust-2021-orange.svg" alt="Rust Edition">
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg" alt="Platform">
  <img src="https://img.shields.io/badge/language-Rust-red.svg" alt="Language">
  <img src="https://img.shields.io/badge/powered_by-Tree--sitter-green.svg" alt="Tree-sitter">
</p>

A multi-language software architecture linter written in Rust that validates architectural rules through a dynamic rule engine. Supports **TypeScript, JavaScript, Python, Go, PHP, and Java** using Tree-sitter for fast and accurate parsing. It ensures that the software design (Hexagonal, Clean, MVC, etc.) is respected regardless of who writes the code.

## Features

- **🌐 Multi-Language Support**: TypeScript, JavaScript, Python, Go, PHP, and Java with Tree-sitter parsing
- **🔧 Dynamic Rule Engine**: Define custom constraints between layers via `architect.json`
- **🔍 Circular Dependency Detection**: Analyzes the dependency graph and automatically detects cycles
- **🤖 AI-Powered Auto-Fix**: Automatically suggests and applies fixes for architectural violations (--fix)
- **👁️ Watch Mode**: Real-time monitoring with incremental analysis and intelligent debouncing (300ms)
- **📂 Smart Path Exclusion**: Automatically ignores node_modules, build folders, and framework-specific directories
- **💬 AI Configuration**: Architect assistant with Claude that suggests rules based on your project
- **⚙️ Separated Configuration**: `architect.json` for rules (sharable) and `.architect.ai.json` for API keys (private)
- **🎯 Automatic Framework Detection**: Recognizes NestJS, React, Angular, Express, Django, Laravel, Spring Boot and more
- **🏗️ Architectural Patterns**: Support for Hexagonal, Clean Architecture, MVC and more
- **📦 Import Validation**: Detects and blocks imports that violate the defined architecture across all supported languages
- **📏 Complexity Control**: Validates that functions don't exceed configurable line limits
- **⚡ Parallel Processing**: Ultra-fast analysis using multi-threaded processing with Rayon
- **📊 Visual Reports**: Detailed and colorful errors with exact problem location
- **🎨 Interactive Mode**: Guided configuration on first run with enhanced visual banner
- **🪝 Git Hooks Integration**: Automatic Husky and pre-commit hook configuration

## Supported Languages

Architect Linter uses **Tree-sitter** for fast and accurate multi-language parsing. The following languages are fully supported:

| Language | Extensions | Import Syntax | Example |
|----------|-----------|---------------|---------|
| **TypeScript** | `.ts`, `.tsx` | `import X from 'path'` | `import { UserService } from './services/user'` |
| **JavaScript** | `.js`, `.jsx` | `import X from 'path'` | `import UserController from '../controllers/user'` |
| **Python** | `.py` | `import X` / `from X import Y` | `from models.user import UserModel` |
| **Go** | `.go` | `import "package"` | `import "github.com/user/repo/models"` |
| **PHP** | `.php` | `use Namespace\Class` | `use App\Controllers\UserController;` |
| **Java** | `.java` | `import package.Class` | `import com.example.models.User;` |

### Language-Specific Features

- **TypeScript/JavaScript**: Full support for ES6 imports, dynamic imports, and type-only imports
- **Python**: Supports both `import` and `from...import` statements, dotted module paths
- **Go**: Package-based imports with full path support
- **PHP**: PSR-4 autoloading compatible, supports `use`, `require`, `include` statements
- **Java**: Package imports with wildcard support

All languages share the same rule engine, allowing you to define architectural constraints consistently across polyglot projects.

## Quick Start

### Option 1: Global Installation (Recommended)

Global installation allows you to run `architect-linter` from any directory.

#### Linux / macOS
```bash
git clone https://github.com/sergio/architect-linter.git
cd architect-linter
chmod +x setup.sh
./setup.sh
```

#### Windows (PowerShell)
```powershell
git clone https://github.com/sergiogswv/architect-linter.git
cd architect-linter

# Run the installation script (avoids execution policy errors)
powershell -NoProfile -ExecutionPolicy Bypass -File .\setup.ps1
```

**After installation**:
1. Open PowerShell as Administrator
2. Run the commands the script shows you to add to PATH
3. **Close ALL terminals** and open a new one
4. Verify: `architect-linter --version`

📖 **Complete Windows guide with troubleshooting**: [INSTALL_WINDOWS.md](INSTALL_WINDOWS.md)

The `setup.sh` / `setup.ps1` script automatically:
1. Detects if it's an initial installation or update
2. Compiles the project in release mode
3. Moves the binary to a global location (`/usr/local/bin` on Linux/macOS, `%USERPROFILE%\bin` on Windows)
4. On installation: Configures PATH if necessary
5. On update: Shows the old version and the new one

### Option 2: Manual Compilation

#### Linux / macOS
```bash
git clone https://github.com/sergiogswv/architect-linter.git
cd architect-linter
cargo build --release

# Move to a folder in your PATH
sudo cp target/release/architect-linter /usr/local/bin/
```

#### Windows (Manual Installation)
```powershell
git clone https://github.com/sergiogswv/architect-linter.git
cd architect-linter
cargo build --release

# Create bin folder if it doesn't exist
mkdir $env:USERPROFILE\bin -Force

# Copy the binary
copy target\release\architect-linter.exe $env:USERPROFILE\bin\

# Add to PATH (run PowerShell as administrator)
$oldPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$newPath = "$oldPath;$env:USERPROFILE\bin"
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')

# Restart your terminal for the changes to take effect
```

### First Use

```bash
# If you installed globally
architect-linter /path/to/your/project

# Or if you use the local binary
./target/release/architect-linter /path/to/your/project

# Interactive mode (shows you available projects)
architect-linter
```

**First run**: If `architect.json` doesn't exist, the linter:
1. Displays a visual welcome banner
2. Requests AI configuration (URL, API Key, Model)
   - Uses environment variables as defaults if available (`ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, `ANTHROPIC_MODEL`)
3. Automatically detects your framework
4. Queries AI to suggest architectural rules
5. Guides you through an interactive wizard to confirm suggestions
6. Creates two files:
   - `architect.json` with the selected rules
   - `.architect.ai.json` with AI configuration
7. Automatically updates `.gitignore` to exclude `.architect.ai.json`
8. Automatically configures Husky and the pre-commit hook

## Update

If you already have architect-linter installed and want to update to the latest version, use the **same installation script**:

### Linux / macOS
```bash
cd /path/to/repository/architect-linter
git pull origin master  # Or the branch you use
./setup.sh
```

### Windows (PowerShell)
```powershell
cd C:\path\to\repository\architect-linter
git pull origin master  # Or the branch you use
powershell -NoProfile -ExecutionPolicy Bypass -File .\setup.ps1
```

**The script automatically detects** if you already have architect-linter installed:
- ✅ If it exists: Update mode (shows old version → compiles → installs → shows new version)
- ✅ If it doesn't exist: Installation mode (compiles → installs → configures PATH if necessary)

**Important for Windows**: After updating, close and reopen your terminal for the changes to take effect.

### Manual Installation/Update

If you prefer to do it manually without using the script:

```bash
# 1. Update the code (if you already have it cloned)
git pull origin master

# 2. Compile
cargo build --release

# 3. Copy the binary

# Linux/macOS
sudo cp target/release/architect-linter /usr/local/bin/

# Windows PowerShell
copy target\release\architect-linter.exe $env:USERPROFILE\bin\
```

### Git Hooks Integration (Automatic)

**New in v2.0!** The linter now automatically configures Husky and the pre-commit hook when generating `architect.json`.

If you prefer to configure it manually:

#### Step 1: Install Husky in your project
```bash
cd /path/to/your/project
npx husky-init && npm install
```

#### Step 2: Configure the Pre-Commit Hook

**Option A: With global installation (Recommended)**
```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Running Architect Linter..."
architect-linter .

if [ $? -ne 0 ]; then
  echo ""
  echo "❌ The commit was cancelled due to architecture violations"
  echo "💡 Fix the errors reported above and try the commit again"
  exit 1
fi

echo "✅ Architecture validation successful"
exit 0
```

**Option B: With specific path**
```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Running Architect Linter..."
"/full/path/architect-linter/target/release/architect-linter" .
```

Edit the `.husky/pre-commit` file with the content of your preference and give it execution permissions:

```bash
chmod +x .husky/pre-commit
```

📖 **Complete integration guide**: [NESTJS_INTEGRATION.md](NESTJS_INTEGRATION.md)

## Dynamic Rule Engine

Architect-linter uses a dynamic rule system defined in `architect.json` that allows restricting which folders can interact with each other, ensuring the architectural design is respected.

### Concept

A forbidden rule defines a **Source (from)** → **Target (to)** relationship:
- If a file located in the **"Source"** path tries to import something from the **"Target"** path, the linter will generate an architecture error.

### Structure in architect.json

**Important**: Since v2.0, configuration is split into two files:

1. **`architect.json`** (sharable in repo):
```json
{
  "max_lines_per_function": 40,
  "architecture_pattern": "Hexagonal",
  "forbidden_imports": [
    {
      "from": "/domain/",
      "to": "/infrastructure/"
    }
  ]
}
```

2. **`.architect.ai.json`** (private, in `.gitignore`):
```json
{
  "api_url": "https://api.anthropic.com",
  "api_key": "sk-ant-api03-...",
  "model": "claude-sonnet-4-5-20250929"
}
```

#### Properties in architect.json

- **`max_lines_per_function`** (number): Line limit per method/function
- **`architecture_pattern`** (string): Architectural pattern (`"Hexagonal"`, `"Clean"`, `"MVC"`, `"Ninguno"`)
- **`forbidden_imports`** (array): List of rules with:
  - **`from`**: Folder/file pattern where the restriction applies
  - **`to`**: Forbidden folder/file pattern to import

#### Security

⚠️ **`.architect.ai.json` contains API keys and must never be shared**:
- Make sure `.architect.ai.json` is in your `.gitignore`
- Each developer should have their own AI configuration
- The `architect.json` file (rules only) can be safely shared in the repo

### How the Engine Works

1. **Scanning**: Converts all paths to lowercase to avoid case errors
2. **Match**: For each file, checks if its path contains the text defined in `from`
3. **Validation**: If there's a match, analyzes each `import`. If the import source contains `to`, it triggers a violation

### Common Use Cases

#### A. Hexagonal Architecture (Preserve the Core)

Prevent business logic from depending on implementation details (Database, External APIs).

```json
{
  "from": "/domain/",
  "to": "/infrastructure/"
}
```

**Result**: If you try to import a TypeORM Repository inside a domain Entity, the linter will block the commit.

#### B. Layer Decoupling (NestJS/MVC)

Prevent Controllers from skipping the service layer.

```json
{
  "from": ".controller.ts",
  "to": ".repository"
}
```

**Result**: Forces injecting a Service instead of querying the database directly from the entry point.

## Rules Guide by Architectural Pattern

### Comparative Restrictions Table

| Pattern | Source Layer (`from`) | Forbidden Folder (`to`) | Technical Reason |
|--------|---------------------|--------------------------|---------------|
| **Hexagonal** | `/domain/` | `/infrastructure/` | The core shouldn't know about database or external APIs |
| **Hexagonal** | `/domain/` | `/application/` | The domain shouldn't depend on specific use cases |
| **Clean** | `/entities/` | `/use-cases/` | High-level business rules shouldn't know orchestration |
| **Clean** | `/use-cases/` | `/controllers/` | Logic shouldn't know who calls it (web, CLI, etc.) |
| **MVC** | `.controller.ts` | `.repository` | Decoupling: Controller only talks to services |
| **MVC** | `.service.ts` | `.controller.ts` | Avoid circular dependencies and maintain pure logic |

### Example: Clean Architecture

```json
{
  "max_lines_per_function": 35,
  "architecture_pattern": "Clean",
  "forbidden_imports": [
    {
      "from": "/entities/",
      "to": "/use-cases/",
      "reason": "Las entidades son el corazón y deben ser agnósticas a los casos de uso."
    },
    {
      "from": "/use-cases/",
      "to": "/infrastructure/",
      "reason": "La lógica de aplicación no debe importar implementaciones directas como TypeORM o Axios."
    }
  ]
}
```

### Example: Hexagonal Architecture

```json
{
  "max_lines_per_function": 40,
  "architecture_pattern": "Hexagonal",
  "forbidden_imports": [
    {
      "from": "/domain/",
      "to": "/infrastructure/"
    },
    {
      "from": "/application/",
      "to": "/infrastructure/"
    }
  ]
}
```

## Usage

### Interactive Mode (First Run)

```bash
./target/release/architect-linter
```

If `architect.json` doesn't exist, the linter:
1. Shows the welcome banner
2. Requests AI configuration (URL, API Key, Model)
   - Uses environment variables as defaults if available
3. Automatically detects the framework (NestJS, React, Angular, Express)
4. Queries AI to suggest architectural rules
5. Presents suggestions in an interactive wizard
6. Creates two files:
   - `architect.json` with the selected rules
   - `.architect.ai.json` with AI configuration
7. Automatically updates `.gitignore` to exclude `.architect.ai.json`
8. Automatically configures Husky and the pre-commit hook

### Automatic Mode (Subsequent Runs)

When `architect.json` already exists, the linter runs silently:

```bash
./target/release/architect-linter /path/to/project
```

or

```bash
cargo run -- /path/to/project
```

### Watch Mode (Real-time Monitoring)

Watch mode enables continuous monitoring of your codebase during development:

```bash
architect-linter --watch .
```

**How it works**:
1. **Initial Analysis**: Performs a complete analysis and builds the dependency graph
2. **File Monitoring**: Watches for changes in `.ts`, `.tsx`, `.js`, `.jsx` files
3. **Intelligent Debouncing**: Waits 300ms after the last change to avoid excessive re-analysis
4. **Incremental Analysis**: Only re-analyzes changed files and their affected dependencies
5. **Partial Cycle Detection**: Runs cycle detection only on the strongly connected component (SCC) containing the changed file

**Benefits**:
- ⚡ **Fast**: Only analyzes what changed, not the entire project
- 🎯 **Smart**: Uses graph caching to avoid redundant work
- 🔄 **Real-time**: Instant feedback as you code
- 💾 **Memory-efficient**: Maintains the dependency graph in memory during the session

**Example output**:
```
🚀 Iniciando modo watch...
📊 Análisis inicial de 42 archivos...
✨ ¡Proyecto impecable! La arquitectura se respeta.
👁️  Modo Watch activado
📂 Observando: /path/to/project
⏱️  Debounce: 300ms
💡 Presiona Ctrl+C para detener

🔄 Cambios detectados en 1 archivo(s):
   📝 src/domain/user.ts

✅ Re-análisis completado
👁️  Esperando cambios...
```

### CLI Arguments

```bash
architect-linter [OPTIONS] [PATH]
```

**Options**:
- `-v, --version`: Shows the linter version
- `-h, --help`: Shows complete help
- `-w, --watch`: Watch mode - monitors file changes and re-analyzes automatically
- `-f, --fix`: Fix mode - AI-powered automatic fixing of architectural violations
- **No arguments**: Interactive mode, shows menu of available projects
- **With path**: `architect-linter /project/path` - Analyzes the specified project

**Examples**:
```bash
architect-linter --version          # Shows: architect-linter 2.3.0
architect-linter --help             # Shows complete help
architect-linter                    # Interactive mode
architect-linter .                  # Analyzes current directory
architect-linter /project/path      # Analyzes specific project
architect-linter --watch .          # Watch mode: monitors changes and re-analyzes
architect-linter --fix .            # Fix mode: auto-fix violations with AI
```

## The Complete Workflow

### First time using the linter

1. **Initial commit**: When running `git commit`, Husky automatically launches the linter
2. **Automatic discovery**: If it's the first time (no `architect.json` exists), the linter:
   - Reads your `package.json` and folder structure
   - Detects the framework (NestJS, React, Angular, Express)
   - Queries AI to suggest line limits and architectural rules
3. **Guided configuration**: Shows you the suggestions and requests confirmation
4. **Persistence**: Once you accept, creates `architect.json` and validates the code
5. **Result**: If there are no violations, the commit continues; if there are, it's aborted showing the errors

### Subsequent runs

Once `architect.json` exists:
- The linter silently loads the configuration
- Validates the code instantly (thanks to Rust)
- Shows violations if they exist or allows the commit

## FAQ (Frequently Asked Questions)

### What do I do if I get a configuration error in architect.json?

The linter automatically validates the `architect.json` file and shows clear error messages with suggestions on how to fix them. The most common errors are:

- **JSON with invalid syntax**: Missing comma, brace or extra characters
- **Missing fields**: `max_lines_per_function`, `architecture_pattern` or `forbidden_imports`
- **Incorrect types**: For example, putting `"50"` (string) instead of `50` (number)
- **Invalid values**: Architectural pattern that doesn't exist, or `max_lines_per_function` at 0

**Each error includes:**
- ✅ Clear description of the problem
- ✅ Suggestion on how to fix it
- ✅ Example of correct code

**Complete error guide:** See [CONFIG_ERRORS.md](CONFIG_ERRORS.md) for detailed examples of all possible errors.

### What happens if tests fail?
The commit is automatically aborted. Git will show you exactly which file and line is breaking the architecture, with visual context of the error.

### Can I skip the linter in an emergency?
Yes, you can use `git commit --no-verify` to skip the hooks, but use it responsibly! The Virtual Architect will feel disappointed 😢

### Do I need internet to use the linter?
Only the **first time** for the AI to suggest rules (assisted initial configuration). Once `architect.json` is created, the linter works **100% offline** and is instant.

### Does it work with JavaScript in addition to TypeScript?
Yes, the linter supports both TypeScript (`.ts`, `.tsx`) and JavaScript (`.js`, `.jsx`).

### How do I update the rules after initial configuration?
Simply edit the `architect.json` file manually. The linter will automatically load the changes on the next run.

### How do I configure AI?
The linter will request AI configuration on the first run. You can also:
- Use environment variables: `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`, `ANTHROPIC_MODEL`
- Edit the `.architect.ai.json` file directly

**Important**: The `.architect.ai.json` file must be in your `.gitignore` to avoid uploading API keys to the repository.

### Can I use the linter without AI?
Yes. You can manually create the `architect.json` file with your rules and the linter will work normally. AI is only used in the initial configuration to suggest rules.

## Example Output

### First Run (Configuration Mode)
```
╔══════════════════════════════════════════════════════════════════════════════════╗

    ___    ____  ______ __  __________________  ______ ______
   /   |  / __ \/ ____// / / /  _/_  __/ ____/ / ____//_  __/
  / /| | / /_/ / /    / /_/ // /  / / / __/   / /      / /
 / ___ |/ _, _/ /___ / __  // /  / / / /___  / /___   / /
/_/  |_/_/ |_|\____//_/ /_/___/ /_/ /_____/  \____/  /_/

    __     _____  _   __ ______ ______ ____
   / /    /  _/ / | / //_  __// ____// __ \
  / /     / /  /  |/ /  / /  / __/  / /_/ /
 / /___ _/ /  / /|  /  / /  / /___ / _, _/
/_____//___/ /_/ |_/  /_/  /_____//_/ |_|

╚══════════════════════════════════════════════════════════════════════════════════╝

                 Maintaining your code architecture ⚡

📝 'architect.json' not found. Starting AI-assisted discovery...

🤖 AI CONFIGURATION
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
To analyze your architecture with AI, you need to configure:
  • API URL (e.g.: https://api.anthropic.com)
  • API Key (your authentication token)
  • Model to use (e.g.: claude-sonnet-4-5-20250929)

API URL [https://api.anthropic.com]:
API Key: ••••••••••••••••
AI Model [claude-sonnet-4-5-20250929]:

✅ AI configuration saved.

🤖 The Virtual Architect has analyzed your project.
? Suggested max lines per function [60]: 40
? Apply the following import rules?
  ✓ src/**/.controller.ts → src/**/.repository.ts
     └─ Reason: Controllers should use services, not repositories
  ✓ src/**/.service.ts → src/**/.controller.ts
     └─ Reason: Services should not depend on controllers

✅ Configuration saved successfully.
🔐 AI configuration saved in: .architect.ai.json
⚠️  This file contains API keys and MUST NOT be shared in the repository.
💡 Make sure '.architect.ai.json' is in your .gitignore
```

### Subsequent Runs (Automatic Mode)
```
╔══════════════════════════════════════════════════════════════════════════════════╗

    ___    ____  ______ __  __________________  ______ ______
   /   |  / __ \/ ____// / / /  _/_  __/ ____/ / ____//_  __/
  / /| | / /_/ / /    / /_/ // /  / / / __/   / /      / /
 / ___ |/ _, _/ /___ / __  // /  / / / /___  / /___   / /
/_/  |_/_/ |_|\____//_/ /_/___/ /_/ /_____/  \____/  /_/

    __     _____  _   __ ______ ______ ____
   / /    /  _/ / | / //_  __// ____// __ \
  / /     / /  /  |/ /  / /  / __/  / /_/ /
 / /___ _/ /  / /|  /  / /  / /___ / _, _/
/_____//___/ /_/ |_/  /_/  /_____//_/ |_|

╚══════════════════════════════════════════════════════════════════════════════════╝

                 Maintaining your code architecture ⚡

📌 Violation in: src/domain/user.entity.ts

  × Architecture Violation
   ╭─[src/domain/user.entity.ts:3:1]
   │
 3 │ import { Repository } from 'typeorm';
   │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   │ Restriction: Files in '/domain/' cannot import from '/infrastructure/'.
   ╰────

❌ 1 architecture violations found.
```

### Circular Dependency Detection
```
🔍 Analyzing circular dependencies...

🔴 CIRCULAR DEPENDENCIES DETECTED

Found 1 dependency cycle(s):

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Cycle #1
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📂 Cycle paths:
  src/services/user.service.ts →
  src/repositories/user.repository.ts →
  src/entities/user.entity.ts →
  src/services/user.service.ts ↑ (closes the cycle)

Circular dependency detected:
  src/services/user.service.ts → src/repositories/user.repository.ts
  src/repositories/user.repository.ts → src/entities/user.entity.ts
  src/entities/user.entity.ts → src/services/user.service.ts

  ⚠️  This breaks the layer hierarchy and creates circular coupling.

💡 Suggested solutions:
  1. Apply Dependency Injection to break the cycle
  2. Extract shared logic to a third module
  3. Use events/observers instead of direct calls
  4. Apply Dependency Inversion Principle (DIP)

⚠️  Circular dependencies found that must be resolved.
```

## Project Structure

```
architect-linter/
├── src/
│   ├── main.rs                 # Main orchestration, circular dependency analysis
│   ├── analyzer.rs             # Multi-language analysis orchestrator
│   ├── autofix.rs              # AI-powered automatic violation fixing
│   ├── config.rs               # Types, config loading/saving in two files
│   ├── circular.rs             # Circular dependency detection (graph + DFS)
│   ├── ui.rs                   # Interactive UI, ASCII banner, configuration wizard
│   ├── ai.rs                   # Claude API integration for suggestions
│   ├── discovery.rs            # Project structure analysis
│   ├── detector.rs             # Automatic framework detection
│   ├── cli.rs                  # Command-line argument handling
│   ├── watch.rs                # Watch mode with incremental analysis
│   └── parsers/
│       ├── mod.rs              # Parser module exports and factory
│       ├── typescript.rs       # TypeScript/JavaScript parser (Tree-sitter)
│       ├── python.rs           # Python parser (Tree-sitter)
│       ├── go.rs               # Go parser (Tree-sitter)
│       ├── php.rs              # PHP parser (Tree-sitter)
│       └── java.rs             # Java parser (Tree-sitter)
├── public/
│   └── architect-linter-banner.png  # Project banner image
├── Cargo.toml                  # Dependencies and project configuration
├── README.md                   # This documentation (English)
├── README_ES.md                # Spanish documentation
├── CHANGELOG.md                # Version history
├── NESTJS_INTEGRATION.md       # NestJS integration guide
├── INSTALL_WINDOWS.md          # Windows installation guide
├── CONFIG_ERRORS_ES.md         # Configuration error guide (Spanish)
├── architect.json.example      # Rules file example
├── .architect.ai.json.example  # AI configuration example
├── .gitignore.example          # Template for project .gitignore
├── setup.sh                    # Installation script for Linux/macOS
├── setup.ps1                   # Installation script for Windows
└── pre-commit.example          # Husky template
```

## Technologies

- **Tree-sitter**: Universal parsing library for all 6 supported languages
  - `tree-sitter-typescript`: TypeScript/JavaScript grammar
  - `tree-sitter-python`: Python grammar
  - `tree-sitter-go`: Go grammar
  - `tree-sitter-php`: PHP grammar
  - `tree-sitter-java`: Java grammar
- **swc_ecma_parser**: High-performance TypeScript/JavaScript parser (legacy support)
- **rayon**: Automatic parallel processing for ultra-fast analysis
- **miette**: Elegant diagnostic reports with rich context
- **notify**: File system watcher for watch mode
- **walkdir**: Efficient directory traversal
- **dialoguer**: Interactive terminal UI
- **indicatif**: Progress bars and spinners
- **serde_json**: JSON configuration parsing
- **reqwest**: HTTP client for Claude API integration
- **tokio**: Async runtime for I/O operations

## Implemented Rules

### 1. Forbidden Imports (Dynamic)
Defined in `architect.json` with the `from` → `to` format. The engine validates each `import` against the configured rules.

### 2. Function Complexity
Counts the lines of each method/function and alerts if it exceeds `max_lines_per_function`.

### 3. Extra Rule: Controller → Repository (NestJS)
Hardcoded prohibition: files containing `"controller"` cannot import `".repository"`, reinforcing the MVC pattern.

## Roadmap

### Completed ✅
- [x] Dynamic rule engine with `forbidden_imports`
- [x] Automatic framework detection (NestJS, React, Angular, Express, Django, Laravel, Spring Boot)
- [x] Interactive configuration on first run
- [x] Support for patterns: Hexagonal, Clean, MVC
- [x] Parallel processing with Rayon
- [x] Automatic Git Hooks integration (Husky)
- [x] Modular architecture (analyzer, config, detector, circular, ui, ai)
- [x] Elegant reports with Miette
- [x] JavaScript support (.js, .jsx)
- [x] JSON schema validation with clear error messages
- [x] Visual ASCII art banner enhanced
- [x] **Separated AI configuration**: `architect.json` (rules) + `.architect.ai.json` (API keys)
- [x] **Circular dependency detection** with graph analysis and DFS
- [x] **Automatic Husky setup** during initial configuration
- [x] **Watch mode** with incremental analysis and intelligent caching
- [x] **Multi-language support**: TypeScript, JavaScript, Python, Go, PHP, Java (6 languages)
- [x] **Tree-sitter integration** for fast and accurate parsing across all languages
- [x] **AI-powered auto-fix** for architectural violations (--fix)

### Coming Soon 🚧
- [ ] Report export (JSON, HTML, Markdown)
- [ ] Web dashboard to visualize historical violations
- [ ] Support for more languages (Rust, C#, Ruby, Kotlin)

### Future 🔮
- [ ] Custom rules via Rust/WASM plugins
- [ ] Native CI/CD integration (GitHub Actions, GitLab CI)
- [ ] Severity configuration per rule (error, warning, info)
- [ ] Language-specific rule templates

## Contributing

Contributions are welcome. Please:

1. Fork the repository
2. Create a branch for your feature (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is under the MIT license.

## Author

Sergio Guadarrama - [GitHub](https://github.com/sergiogswv)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete version history.

### v3.1.0 (2026-02-06) - Multi-Language Support: PHP & Java
- 🌐 **PHP Parser**: Full Tree-sitter integration with support for use/require/include statements
- ☕ **Java Parser**: Complete Tree-sitter grammar support with import analysis
- 📚 **6 Languages Total**: TypeScript, JavaScript, Python, Go, PHP, Java now fully supported
- 🎨 **Professional Banner**: New project banner in documentation
- 📖 **Enhanced Documentation**: Multi-language support table in English and Spanish
- 🔧 **Improved Setup Scripts**: Better error handling and PATH configuration
- 🧹 **Code Cleanup**: Removed 72 lines of dead code (LanguageInfo, unused methods)
- ⚡ **Tree-sitter Dependencies**: Added tree-sitter-php and tree-sitter-java
- 📁 **Updated Examples**: architect.json.example with PHP and Java rule examples

### v2.0.0 (2026-02-04) - Major Release: Circular Dependencies & Security
- 🔴 **Circular dependency detection**: Graph-based analysis with DFS algorithm
- 🔐 **Separated configuration**: `architect.json` (sharable) + `.architect.ai.json` (private)
- 🎨 **Enhanced visual experience**: ASCII art banner with high-impact style
- ⚙️ **AI configuration**: URL, API key, and model now configurable via wizard
- 🪝 **Automatic Husky**: Pre-commit hook configuration during initial setup
- 📁 **Example files**: `.architect.ai.json.example` and `.gitignore.example`
- 🔒 **Security improvements**: API keys never shared in repository
- 📚 **Updated documentation**: README, examples, and error guide

### v1.0.0 (2026-01-31) - First Stable Release
- 🎉 First stable version ready for production
- 🚀 CLI Flags: `--version` and `--help` implemented
- 📦 Optimized installation for Windows with improved scripts
- 📚 Complete Windows installation documentation with troubleshooting
- ✅ Full validation on real projects

### v0.8.0 (2026-01-31) - AI-Assisted Configuration
- 🤖 Integration with Claude (Anthropic API) for intelligent architectural suggestions
- 🔍 Automatic project discovery with dependency and structure analysis
- 📦 Automated installation scripts for Linux/macOS and Windows
- 💡 Interactive wizard for AI-suggested rule confirmation
- 📚 Complete FAQ and workflow documentation
- 🎯 Separate UI module for better code organization

### v0.7.0 (2026-01-30) - Dynamic Rule Engine
- ✨ Fully functional dynamic rule engine
- 🔍 Automatic framework detection with `detector.rs` module
- 🎯 Interactive configuration on first run
- 📐 Support for architectural patterns: Hexagonal, Clean, MVC
- 🛠️ Compilation error and warning fixes
- 📚 Updated documentation with examples per pattern
