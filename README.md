# Architect Linter

**Version:** 2.0.0

A software architecture linter written in Rust that validates architectural rules in TypeScript/JavaScript projects through a dynamic rule engine. It ensures that the software design (Hexagonal, Clean, MVC, etc.) is respected regardless of who writes the code.

## Features

- **Dynamic Rule Engine**: Define custom constraints between layers via `architect.json`
- **Automatic Framework Detection**: Recognizes NestJS, React, Angular, Express and suggests optimal configurations
- **Architectural Patterns**: Support for Hexagonal, Clean Architecture, MVC and more
- **Import Validation**: Detects and blocks imports that violate the defined architecture
- **Complexity Control**: Validates that functions don't exceed configurable line limits
- **Parallel Processing**: Ultra-fast analysis using multi-threaded processing with Rayon
- **Visual Reports**: Detailed and colorful errors with exact problem location
- **Interactive Mode**: Guided configuration on first run
- **Git Hooks Integration**: Compatible with Husky for automatic pre-commit validation

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

**First run**: If `architect.json` doesn't exist, the linter will automatically detect your framework and guide you with an interactive wizard to configure the architectural rules.

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

### Git Hooks Integration (Recommended)

Validate the architecture automatically before each commit using Husky.

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

echo "🏗️  Validando arquitectura antes del commit..."
architect-linter .
```

**Option B: With specific path**
```bash
#!/bin/sh
. "$(dirname "$0")/_/husky.sh"

echo "🏗️  Validando arquitectura antes del commit..."
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

#### Properties

- **`max_lines_per_function`** (number): Line limit per method/function
- **`architecture_pattern`** (string): Architectural pattern (`"Hexagonal"`, `"Clean"`, `"MVC"`, `"Ninguno"`)
- **`forbidden_imports`** (array): List of rules with:
  - **`from`**: Folder/file pattern where the restriction applies
  - **`to`**: Forbidden folder/file pattern to import

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
1. Automatically detects the framework (NestJS, React, Angular, Express)
2. Suggests an architectural pattern
3. Proposes a line limit based on the detected framework
4. Creates the `architect.json` file with the selected configuration

### Automatic Mode (Subsequent Runs)

When `architect.json` already exists, the linter runs silently:

```bash
./target/release/architect-linter /path/to/project
```

or

```bash
cargo run -- /path/to/project
```

### CLI Arguments

```bash
architect-linter [OPTIONS] [PATH]
```

**Options**:
- `-v, --version`: Shows the linter version
- `-h, --help`: Shows complete help
- **No arguments**: Interactive mode, shows menu of available projects
- **With path**: `architect-linter /project/path` - Analyzes the specified project

**Examples**:
```bash
architect-linter --version          # Shows: architect-linter 2.0.0
architect-linter --help             # Shows complete help
architect-linter                    # Interactive mode
architect-linter .                  # Analyzes current directory
architect-linter /project/path      # Analyzes specific project
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

### What environment variables do I need for AI?
For AI-assisted configuration you need:
- `ANTHROPIC_AUTH_TOKEN`: Your Anthropic API key
- `ANTHROPIC_BASE_URL`: API endpoint URL

If they're not configured, the linter will indicate this on the first run.

## Example Output

### First Run (Configuration Mode)
```
🏛️  WELCOME TO ARCHITECT-LINTER
📝 No encontré 'architect.json'. Vamos a configurar tu proyecto.
? Confirmar Framework (Detectado: NestJS) › NestJS
? ¿Qué patrón arquitectónico quieres aplicar? › Hexagonal
? Límite de líneas por método › 40
✅ Configuración guardada en 'architect.json'
```

### Subsequent Runs (Automatic Mode)
```
🏛️  WELCOME TO ARCHITECT-LINTER

📌 Violación en: src/domain/user.entity.ts

  × Violación de Arquitectura
   ╭─[src/domain/user.entity.ts:3:1]
   │
 3 │ import { Repository } from 'typeorm';
   │ ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   │ Restricción: Archivos en '/domain/' no pueden importar de '/infrastructure/'.
   ╰────

❌ Se encontraron 1 violaciones.
```

## Project Structure

```
architect-linter/
├── src/
│   ├── main.rs                 # Orchestration, interactive configuration, file collection
│   ├── analyzer.rs             # TypeScript analysis, dynamic rule validation
│   ├── config.rs               # Types: LinterContext, ArchPattern, Framework, ForbiddenRule
│   └── detector.rs             # Framework detection and LOC suggestions
├── Cargo.toml                  # Dependencies and project configuration
├── README.md                   # This documentation
├── CHANGELOG.md                # Version history
├── NESTJS_INTEGRATION.md       # Git Hooks integration guide
└── pre-commit.example          # Husky template
```

## Technologies

- **swc_ecma_parser**: High-performance TypeScript/JavaScript parser
- **rayon**: Automatic parallel processing
- **miette**: Elegant diagnostic reports with context
- **walkdir**: Efficient directory traversal
- **dialoguer**: Interactive terminal UI
- **indicatif**: Progress bars
- **serde_json**: JSON configuration parsing

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
- [x] Automatic framework detection (NestJS, React, Angular, Express)
- [x] Interactive configuration on first run
- [x] Support for patterns: Hexagonal, Clean, MVC
- [x] Parallel processing with Rayon
- [x] Git Hooks integration (Husky)
- [x] Modular architecture (analyzer, config, detector)
- [x] Elegant reports with Miette
- [x] JavaScript support (.js, .jsx)
- [x] JSON schema validation with clear error messages

### Coming Soon 🚧
- [ ] Report export (JSON, HTML, Markdown)
- [ ] Watch mode for continuous development
- [ ] Incremental analysis with cache

### Future 🔮
- [ ] Custom rules via Rust/WASM plugins
- [ ] Native CI/CD integration (GitHub Actions, GitLab CI)
- [ ] Severity configuration per rule (error, warning, info)
- [ ] Web dashboard to visualize historical violations
- [ ] Support for more languages (Python, Go, Java)

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

### v2.0.0 (2026-02-03) - Major Update
- 🌐 Complete English translation of documentation
- 📚 Improved internationalization and accessibility
- ✨ Enhanced documentation structure and clarity

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
