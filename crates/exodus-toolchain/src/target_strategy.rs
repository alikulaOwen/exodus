//! Extensible Target Language Emitter Strategies & Registry for Project Exodus.

use crate::workspace::{DomainArchetype, PackageDescriptor};
use std::path::Path;

/// Trait governing target language scaffolding, entrypoint exports, manifest synthesis,
/// directory layout conventions, ecosystem-appropriate naming standards, and toolchain integration.
pub trait TargetLanguageEmitter: Send + Sync {
    /// Canonical target language identifier (e.g. "rust", "go", "zig", "typescript", "python", "csharp", "dart").
    fn name(&self) -> &'static str;

    /// File extension for code files in this target language (e.g. "rs", "go", "zig", "ts", "py", "cs", "dart").
    fn extension(&self) -> &'static str;

    /// Preferred source subdirectory (e.g. "src", ".", "lib", "src/main/java").
    fn source_dir(&self) -> &'static str;

    /// Primary package manifest name (e.g. "Cargo.toml", "go.mod", "build.zig", "package.json", "pyproject.toml", "pubspec.yaml").
    fn manifest_name(&self) -> &'static str;

    /// Default package entrypoint filename (e.g. "lib.rs", "doc.go", "lib.zig", "index.ts", "__init__.py", "main.dart").
    fn entrypoint_filename(&self) -> &'static str;

    /// Ecosystem-accurate container terminology: "Crate" for Rust, "Package" for Go / Dart / Python / TS, "Project" for C#.
    fn ecosystem_container_term(&self) -> &'static str;

    /// Collection directory convention for multi-package workspaces based on language and domain archetype
    /// (e.g. "packages" or "services" for Go/Dart/Node, "crates" for Rust, "src" for C#).
    fn collection_dir_name(&self, archetype: DomainArchetype) -> &'static str {
        match archetype {
            DomainArchetype::BackendService => "services",
            DomainArchetype::CliTool | DomainArchetype::FrontendApp => "apps",
            _ => {
                if self.name() == "rust" {
                    "crates"
                } else {
                    "packages"
                }
            }
        }
    }

    /// Contextual semantic label for migrated units (e.g. "Service Route / Endpoint", "CLI Command", "Task Consumer").
    fn contextual_unit_label(&self, archetype: DomainArchetype, unit_name: &str) -> String {
        match archetype {
            DomainArchetype::BackendService => {
                format!("API Service Route / Endpoint `{unit_name}`")
            }
            DomainArchetype::CliTool => {
                format!("CLI Command / Subcommand `{unit_name}`")
            }
            DomainArchetype::WorkerQueue => {
                format!("Worker Consumer / Queue Task `{unit_name}`")
            }
            DomainArchetype::FrontendApp => {
                format!("UI View / Client Component `{unit_name}`")
            }
            DomainArchetype::SharedLibrary => {
                format!("Domain Entity / Schema Module `{unit_name}`")
            }
            DomainArchetype::Unknown => {
                format!("Migrated Unit `{unit_name}`")
            }
        }
    }

    /// Generates root entrypoint export file connecting all migrated modules in a package.
    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        archetype: DomainArchetype,
        migrated_module_names: &[String],
    ) -> String;

    /// Generates deterministic module template/stub when LLM is offline or in fallback mode.
    fn generate_deterministic_module_source(
        &self,
        module_name: &str,
        source_path: &Path,
        archetype: DomainArchetype,
    ) -> String;

    /// Generates target package manifest (e.g. Cargo.toml, go.mod, package.json, pubspec.yaml).
    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String);

    /// Generates top-level workspace manifest (e.g. Cargo.toml [workspace], go.work, pnpm-workspace.yaml).
    fn generate_workspace_manifest(
        &self,
        packages: &[&PackageDescriptor],
    ) -> Option<(String, String)>;

    /// Generates ecosystem-specific version lock/manager files (e.g. .go-version, .python-version, rust-toolchain.toml).
    fn generate_version_files(&self, version_str: &str) -> Vec<(String, String)>;

    /// Returns recommended quickstart commands for the target language.
    fn quickstart_command(&self) -> &'static str;

    /// Generates modern cloud-native SDLC scaffolding files (.env.example, Dockerfile, ci.yml, health probes).
    fn generate_sdlc_scaffolding(
        &self,
        pkg_name: &str,
        archetype: DomainArchetype,
    ) -> Vec<(String, String)> {
        let mut files = Vec::new();
        files.push((
            ".env.example".to_string(),
            format!("PORT=8080\nENVIRONMENT=development\nAPP_NAME={pkg_name}\n"),
        ));

        let is_service = matches!(
            archetype,
            DomainArchetype::BackendService | DomainArchetype::WorkerQueue
        );

        if is_service {
            files.push((
                ".github/workflows/ci.yml".to_string(),
                format!(
                    "name: CI Pipeline\non: [push, pull_request]\njobs:\n  build-and-test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - name: Run Checks\n        run: echo 'Running automated checks for {pkg_name}'\n"
                ),
            ));
        }

        files
    }
}

// ----------------------------------------------------------------------------
// Built-in Language Emitter Implementations
// ----------------------------------------------------------------------------

pub struct RustEmitter;

impl TargetLanguageEmitter for RustEmitter {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn extension(&self) -> &'static str {
        "rs"
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn manifest_name(&self) -> &'static str {
        "Cargo.toml"
    }

    fn entrypoint_filename(&self) -> &'static str {
        "lib.rs"
    }

    fn ecosystem_container_term(&self) -> &'static str {
        "Crate"
    }

    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        _archetype: DomainArchetype,
        migrated_module_names: &[String],
    ) -> String {
        let mut content = format!("//! Migrated Rust crate for `{pkg_name}`\n\n");
        for m in migrated_module_names {
            content.push_str(&format!("pub mod {m};\npub use {m}::*;\n"));
        }
        content
    }

    fn generate_deterministic_module_source(
        &self,
        module_name: &str,
        source_path: &Path,
        _archetype: DomainArchetype,
    ) -> String {
        format!(
            "//! Migrated Rust module `{module_name}`\n// Migrated from {}\n\n",
            source_path.display()
        )
    }

    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String) {
        let raw_name = if is_standalone {
            format!("{}-rust", pkg.name.replace('_', "-"))
        } else {
            pkg.name.replace('_', "-")
        };
        let safe_name = if raw_name.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            format!("pkg-{raw_name}")
        } else {
            raw_name
        };

        let mut extra_deps = String::new();
        match &pkg.domain_archetype {
            DomainArchetype::BackendService => {
                extra_deps.push_str("axum = \"0.7\"\ntokio = { version = \"1\", features = [\"full\"] }\ntower = { version = \"0.5\", features = [\"util\"] }\ntower-http = { version = \"0.5\", features = [\"cors\", \"trace\"] }\nthiserror = \"1.0\"\nanyhow = \"1.0\"\nsha2 = \"0.10\"\nhex = \"0.4\"\n");
            }
            DomainArchetype::CliTool => {
                extra_deps.push_str("clap = { version = \"4.4\", features = [\"derive\"] }\nindicatif = \"0.17\"\nthiserror = \"1.0\"\nanyhow = \"1.0\"\n");
            }
            DomainArchetype::WorkerQueue => {
                extra_deps.push_str("tokio = { version = \"1\", features = [\"full\"] }\nthiserror = \"1.0\"\nanyhow = \"1.0\"\nsha2 = \"0.10\"\nhex = \"0.4\"\n");
            }
            DomainArchetype::SharedLibrary => {
                extra_deps.push_str("thiserror = \"1.0\"\nanyhow = \"1.0\"\n");
            }
            _ => {}
        }

        if !is_standalone {
            for dep_name in &pkg.dependencies {
                if let Some(dep_pkg) = all_packages.iter().find(|p| p.name == *dep_name) {
                    let dep_safe_name = dep_pkg.name.replace('_', "-");
                    let dep_depth = pkg.relative_path.components().count();
                    let up_prefix = "../".repeat(dep_depth);
                    let dep_rel_path = format!("{}{}", up_prefix, dep_pkg.relative_path.display());
                    extra_deps.push_str(&format!(
                        "{dep_safe_name} = {{ path = \"{dep_rel_path}\" }}\n"
                    ));
                }
            }
        }

        let cargo_toml = format!(
            r#"[package]
name = "{safe_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
{extra_deps}
"#
        );
        ("Cargo.toml".to_string(), cargo_toml)
    }

    fn generate_workspace_manifest(
        &self,
        packages: &[&PackageDescriptor],
    ) -> Option<(String, String)> {
        let mut member_paths = Vec::new();
        for pkg in packages {
            member_paths.push(format!("\"{}\"", pkg.relative_path.display()));
        }
        let members_str = if member_paths.is_empty() {
            "\"packages/*\", \"apps/*\", \"services/*\", \"crates/*\"".to_string()
        } else {
            member_paths.join(",\n    ")
        };

        let cargo_toml = format!(
            r#"[workspace]
resolver = "2"
members = [
    {members_str}
]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1", features = ["full"] }}
axum = "0.7"
clap = {{ version = "4.4", features = ["derive"] }}
thiserror = "1.0"
anyhow = "1.0"
"#
        );
        Some(("Cargo.toml".to_string(), cargo_toml))
    }

    fn generate_version_files(&self, _version_str: &str) -> Vec<(String, String)> {
        let rust_toolchain = r#"[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
profile = "minimal"
"#;
        vec![("rust-toolchain.toml".to_string(), rust_toolchain.to_string())]
    }

    fn quickstart_command(&self) -> &'static str {
        "cargo check\ncargo test"
    }

    fn generate_sdlc_scaffolding(
        &self,
        pkg_name: &str,
        archetype: DomainArchetype,
    ) -> Vec<(String, String)> {
        let mut files = Vec::new();
        files.push((
            ".env.example".to_string(),
            format!("PORT=8080\nRUST_LOG=info\nAPP_NAME={pkg_name}\n"),
        ));

        if matches!(archetype, DomainArchetype::BackendService | DomainArchetype::WorkerQueue) {
            let dockerfile = format!(
                r#"# Multi-Stage Distroless Containerfile for {pkg_name}
FROM rust:1.80-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/{pkg_name} /usr/local/bin/{pkg_name}
ENV PORT=8080
EXPOSE 8080
CMD ["/usr/local/bin/{pkg_name}"]
"#
            );
            files.push(("Dockerfile".to_string(), dockerfile));

            let ci = format!(
                r#"name: Rust CI Pipeline
on: [push, pull_request]
jobs:
  check-and-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - name: Cargo Check
        run: cargo check --all-targets
      - name: Cargo Test
        run: cargo test
"#
            );
            files.push((".github/workflows/ci.yml".to_string(), ci));
        }

        files
    }
}

pub struct GoEmitter;

impl TargetLanguageEmitter for GoEmitter {
    fn name(&self) -> &'static str {
        "go"
    }

    fn extension(&self) -> &'static str {
        "go"
    }

    fn source_dir(&self) -> &'static str {
        "."
    }

    fn manifest_name(&self) -> &'static str {
        "go.mod"
    }

    fn entrypoint_filename(&self) -> &'static str {
        "doc.go"
    }

    fn ecosystem_container_term(&self) -> &'static str {
        "Package"
    }

    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        archetype: DomainArchetype,
        _migrated_module_names: &[String],
    ) -> String {
        let pkg_ident = if archetype == DomainArchetype::CliTool {
            "main".to_string()
        } else {
            pkg_name.replace('-', "").to_lowercase()
        };
        format!(
            "// Package {pkg_ident} provides migrated domain services for `{pkg_name}`\npackage {pkg_ident}\n\n// Migrated via Project Exodus Autonomous Migration Engine\n"
        )
    }

    fn generate_deterministic_module_source(
        &self,
        _module_name: &str,
        source_path: &Path,
        archetype: DomainArchetype,
    ) -> String {
        let pkg_ident = if archetype == DomainArchetype::CliTool {
            "main".to_string()
        } else {
            "migrated".to_string()
        };
        format!(
            "package {pkg_ident}\n\n// Migrated from {}\n",
            source_path.display()
        )
    }

    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        _all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String) {
        let safe_name = if is_standalone {
            format!("{}-go", pkg.name.replace('_', "-"))
        } else {
            pkg.name.replace('_', "-")
        };
        let go_mod = format!("module {safe_name}\n\ngo 1.22\n");
        ("go.mod".to_string(), go_mod)
    }

    fn generate_workspace_manifest(
        &self,
        packages: &[&PackageDescriptor],
    ) -> Option<(String, String)> {
        let mut use_lines = Vec::new();
        for pkg in packages {
            use_lines.push(format!("\t./{}", pkg.relative_path.display()));
        }
        let uses_str = if use_lines.is_empty() {
            "\t./packages/*\n\t./apps/*\n\t./services/*".to_string()
        } else {
            use_lines.join("\n")
        };
        let go_work = format!("go 1.22\n\nuse (\n{uses_str}\n)\n");
        Some(("go.work".to_string(), go_work))
    }

    fn generate_version_files(&self, version_str: &str) -> Vec<(String, String)> {
        vec![(".go-version".to_string(), format!("{version_str}\n"))]
    }

    fn quickstart_command(&self) -> &'static str {
        "go build ./...\ngo test ./..."
    }

    fn generate_sdlc_scaffolding(
        &self,
        pkg_name: &str,
        archetype: DomainArchetype,
    ) -> Vec<(String, String)> {
        let mut files = Vec::new();
        files.push((
            ".env.example".to_string(),
            format!("PORT=8080\nLOG_LEVEL=info\nAPP_NAME={pkg_name}\n"),
        ));

        if matches!(archetype, DomainArchetype::BackendService | DomainArchetype::WorkerQueue) {
            let dockerfile = format!(
                r#"# Multi-Stage Distroless Containerfile for {pkg_name}
FROM golang:1.22-alpine as builder
WORKDIR /app
COPY . .
RUN CGO_ENABLED=0 GOOS=linux go build -o /app/server .

FROM gcr.io/distroless/static-debian12
COPY --from=builder /app/server /server
ENV PORT=8080
EXPOSE 8080
CMD ["/server"]
"#
            );
            files.push(("Dockerfile".to_string(), dockerfile));

            let ci = format!(
                r#"name: Go CI Pipeline
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: '1.22'
      - name: Go Vet
        run: go vet ./...
      - name: Go Test
        run: go test -v ./...
"#
            );
            files.push((".github/workflows/ci.yml".to_string(), ci));
        }

        files
    }
}

pub struct DartEmitter;

impl TargetLanguageEmitter for DartEmitter {
    fn name(&self) -> &'static str {
        "dart"
    }

    fn extension(&self) -> &'static str {
        "dart"
    }

    fn source_dir(&self) -> &'static str {
        "lib"
    }

    fn manifest_name(&self) -> &'static str {
        "pubspec.yaml"
    }

    fn entrypoint_filename(&self) -> &'static str {
        "main.dart"
    }

    fn ecosystem_container_term(&self) -> &'static str {
        "Package"
    }

    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        _archetype: DomainArchetype,
        migrated_module_names: &[String],
    ) -> String {
        let mut content = format!("/// Migrated Dart package for `{pkg_name}`\nlibrary {pkg_name};\n\n");
        for m in migrated_module_names {
            content.push_str(&format!("export '{m}.dart';\n"));
        }
        content
    }

    fn generate_deterministic_module_source(
        &self,
        module_name: &str,
        source_path: &Path,
        _archetype: DomainArchetype,
    ) -> String {
        format!(
            "/// Migrated Dart module `{module_name}`\n// Migrated from {}\n\nclass {}Service {{\n}}\n",
            source_path.display(),
            module_name
        )
    }

    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        _all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String) {
        let safe_name = if is_standalone {
            format!("{}_dart", pkg.name.replace('-', "_"))
        } else {
            pkg.name.replace('-', "_")
        };
        let pubspec = format!(
            r#"name: {safe_name}
description: Migrated Dart package for {safe_name}
version: 0.1.0
environment:
  sdk: '>=3.0.0 <4.0.0'

dependencies:
  meta: ^1.11.0

dev_dependencies:
  test: ^1.24.0
  lints: ^3.0.0
"#
        );
        ("pubspec.yaml".to_string(), pubspec)
    }

    fn generate_workspace_manifest(
        &self,
        _packages: &[&PackageDescriptor],
    ) -> Option<(String, String)> {
        None
    }

    fn generate_version_files(&self, _version_str: &str) -> Vec<(String, String)> {
        vec![(".dart-version".to_string(), "3.4.0\n".to_string())]
    }

    fn quickstart_command(&self) -> &'static str {
        "dart pub get\ndart test"
    }
}

pub struct TypeScriptEmitter {
    pub is_js: bool,
}

impl TargetLanguageEmitter for TypeScriptEmitter {
    fn name(&self) -> &'static str {
        if self.is_js {
            "javascript"
        } else {
            "typescript"
        }
    }

    fn extension(&self) -> &'static str {
        if self.is_js {
            "js"
        } else {
            "ts"
        }
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn manifest_name(&self) -> &'static str {
        "package.json"
    }

    fn entrypoint_filename(&self) -> &'static str {
        if self.is_js {
            "index.js"
        } else {
            "index.ts"
        }
    }

    fn ecosystem_container_term(&self) -> &'static str {
        "Package"
    }

    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        _archetype: DomainArchetype,
        migrated_module_names: &[String],
    ) -> String {
        let mut content = format!("//! Migrated TypeScript package for `{pkg_name}`\n\n");
        for m in migrated_module_names {
            content.push_str(&format!("export * from './{m}';\n"));
        }
        content
    }

    fn generate_deterministic_module_source(
        &self,
        module_name: &str,
        source_path: &Path,
        _archetype: DomainArchetype,
    ) -> String {
        format!(
            "// Migrated TypeScript module `{module_name}`\n// Migrated from {}\n\nexport const MODULE_NAME = \"{module_name}\";\n",
            source_path.display()
        )
    }

    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        _all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String) {
        let safe_name = if is_standalone {
            format!("{}-ts", pkg.name.replace('_', "-"))
        } else {
            pkg.name.replace('_', "-")
        };
        let pkg_json = format!(
            r#"{{
  "name": "@{safe_name}",
  "version": "0.1.0",
  "type": "module",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {{
    "build": "tsc",
    "test": "node --test"
  }}
}}
"#
        );
        ("package.json".to_string(), pkg_json)
    }

    fn generate_workspace_manifest(
        &self,
        _packages: &[&PackageDescriptor],
    ) -> Option<(String, String)> {
        let pkg_json = r#"{
  "name": "migrated-workspace",
  "private": true,
  "workspaces": [
    "packages/*",
    "apps/*",
    "services/*"
  ]
}
"#;
        Some(("package.json".to_string(), pkg_json.to_string()))
    }

    fn generate_version_files(&self, version_str: &str) -> Vec<(String, String)> {
        let short_node = version_str.split('.').next().unwrap_or("22");
        vec![
            (".nvmrc".to_string(), format!("{short_node}\n")),
            (".node-version".to_string(), format!("{version_str}\n")),
        ]
    }

    fn quickstart_command(&self) -> &'static str {
        "npm install\nnpm run build\nnpm test"
    }
}

pub struct PythonEmitter;

impl TargetLanguageEmitter for PythonEmitter {
    fn name(&self) -> &'static str {
        "python"
    }

    fn extension(&self) -> &'static str {
        "py"
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn manifest_name(&self) -> &'static str {
        "pyproject.toml"
    }

    fn entrypoint_filename(&self) -> &'static str {
        "__init__.py"
    }

    fn ecosystem_container_term(&self) -> &'static str {
        "Package"
    }

    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        _archetype: DomainArchetype,
        migrated_module_names: &[String],
    ) -> String {
        let mut content = format!("# Migrated Python package for `{pkg_name}`\n\n");
        for m in migrated_module_names {
            content.push_str(&format!("from . import {m}\n"));
        }
        content
    }

    fn generate_deterministic_module_source(
        &self,
        module_name: &str,
        source_path: &Path,
        _archetype: DomainArchetype,
    ) -> String {
        format!(
            "# Migrated Python module `{module_name}`\n# Migrated from {}\n\n",
            source_path.display()
        )
    }

    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        _all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String) {
        let safe_name = if is_standalone {
            format!("{}-python", pkg.name.replace('_', "-"))
        } else {
            pkg.name.replace('_', "-")
        };
        let pyproject = format!(
            r#"[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.build_meta"

[project]
name = "{safe_name}"
version = "0.1.0"
description = "Migrated Python package for {safe_name}"
readme = "README.md"
requires-python = ">=3.10"
dependencies = []

[tool.pytest.ini_options]
testpaths = ["tests", "src"]
"#
        );
        ("pyproject.toml".to_string(), pyproject)
    }

    fn generate_workspace_manifest(
        &self,
        _packages: &[&PackageDescriptor],
    ) -> Option<(String, String)> {
        None
    }

    fn generate_version_files(&self, version_str: &str) -> Vec<(String, String)> {
        let short_py = version_str.split('.').take(2).collect::<Vec<_>>().join(".");
        vec![(".python-version".to_string(), format!("{short_py}\n"))]
    }

    fn quickstart_command(&self) -> &'static str {
        "pip install -e .\npytest"
    }
}

pub struct ZigEmitter;

impl TargetLanguageEmitter for ZigEmitter {
    fn name(&self) -> &'static str {
        "zig"
    }

    fn extension(&self) -> &'static str {
        "zig"
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn manifest_name(&self) -> &'static str {
        "build.zig"
    }

    fn entrypoint_filename(&self) -> &'static str {
        "lib.zig"
    }

    fn ecosystem_container_term(&self) -> &'static str {
        "Package"
    }

    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        _archetype: DomainArchetype,
        migrated_module_names: &[String],
    ) -> String {
        let mut content = format!(
            "//! Migrated Zig package for `{pkg_name}`\n\nconst std = @import(\"std\");\n\n"
        );
        for m in migrated_module_names {
            content.push_str(&format!("pub const {m} = @import(\"{m}.zig\");\n"));
        }
        content
    }

    fn generate_deterministic_module_source(
        &self,
        module_name: &str,
        source_path: &Path,
        _archetype: DomainArchetype,
    ) -> String {
        format!(
            "//! Migrated Zig module `{module_name}`\nconst std = @import(\"std\");\n\n// Migrated from {}\n",
            source_path.display()
        )
    }

    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        _all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String) {
        let safe_name = if is_standalone {
            format!("{}-zig", pkg.name.replace('_', "-"))
        } else {
            pkg.name.replace('_', "-")
        };
        let build_zig = format!(
            r#"const std = @import("std");

pub fn build(b: *std.Build) void {{
    const target = b.standardTargetOptions(.{{}});
    const optimize = b.standardOptimizeOption(.{{}});

    const lib = b.addStaticLibrary(.{{
        .name = "{safe_name}",
        .root_source_file = b.path("src/lib.zig"),
        .target = target,
        .optimize = optimize,
    }});
    b.installArtifact(lib);

    const main_tests = b.addTest(.{{
        .root_source_file = b.path("src/lib.zig"),
        .target = target,
        .optimize = optimize,
    }});
    const run_main_tests = b.addRunArtifact(main_tests);
    const test_step = b.step("test", "Run library tests");
    test_step.dependOn(&run_main_tests.step);
}}
"#
        );
        ("build.zig".to_string(), build_zig)
    }

    fn generate_workspace_manifest(
        &self,
        _packages: &[&PackageDescriptor],
    ) -> Option<(String, String)> {
        None
    }

    fn generate_version_files(&self, version_str: &str) -> Vec<(String, String)> {
        vec![(".zig-version".to_string(), format!("{version_str}\n"))]
    }

    fn quickstart_command(&self) -> &'static str {
        "zig build\nzig build test"
    }
}

pub struct CsharpEmitter;

impl TargetLanguageEmitter for CsharpEmitter {
    fn name(&self) -> &'static str {
        "csharp"
    }

    fn extension(&self) -> &'static str {
        "cs"
    }

    fn source_dir(&self) -> &'static str {
        "src"
    }

    fn manifest_name(&self) -> &'static str {
        "Project.csproj"
    }

    fn entrypoint_filename(&self) -> &'static str {
        "Program.cs"
    }

    fn ecosystem_container_term(&self) -> &'static str {
        "Project"
    }

    fn generate_entrypoint(
        &self,
        pkg_name: &str,
        archetype: DomainArchetype,
        migrated_module_names: &[String],
    ) -> String {
        let ns = pkg_name.replace('-', ".");
        if archetype == DomainArchetype::CliTool {
            format!(
                "namespace {ns};\n\npublic static class Program {{\n    public static void Main(string[] args) {{\n        System.Console.WriteLine(\"Migrated {pkg_name} CLI\");\n    }}\n}}\n"
            )
        } else {
            let mut content = format!("namespace {ns};\n\n// Migrated exports for {pkg_name}\n");
            for m in migrated_module_names {
                content.push_str(&format!("// export module {m}\n"));
            }
            content
        }
    }

    fn generate_deterministic_module_source(
        &self,
        module_name: &str,
        source_path: &Path,
        _archetype: DomainArchetype,
    ) -> String {
        format!(
            "// Migrated C# module `{module_name}`\n// Migrated from {}\n\nnamespace Migrated;\n\npublic class {}Module {{ }}\n",
            source_path.display(),
            module_name
        )
    }

    fn generate_package_manifest(
        &self,
        pkg: &PackageDescriptor,
        _all_packages: &[PackageDescriptor],
        is_standalone: bool,
    ) -> (String, String) {
        let safe_name = if is_standalone {
            format!("{}-csharp", pkg.name.replace('_', "-"))
        } else {
            pkg.name.replace('_', "-")
        };
        let csproj = format!(
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <TargetFramework>net8.0</TargetFramework>
    <Nullable>enable</Nullable>
    <ImplicitUsings>enable</ImplicitUsings>
    <AssemblyName>{safe_name}</AssemblyName>
  </PropertyGroup>
</Project>
"#
        );
        (format!("{safe_name}.csproj"), csproj)
    }

    fn generate_workspace_manifest(
        &self,
        _packages: &[&PackageDescriptor],
    ) -> Option<(String, String)> {
        None
    }

    fn generate_version_files(&self, _version_str: &str) -> Vec<(String, String)> {
        vec![(
            "global.json".to_string(),
            r#"{"sdk": {"version": "8.0.100"}}"#.to_string(),
        )]
    }

    fn quickstart_command(&self) -> &'static str {
        "dotnet build\ndotnet test"
    }
}

// ----------------------------------------------------------------------------
// Target Language Registry
// ----------------------------------------------------------------------------

/// Central lookup and factory for target language emitters.
pub struct TargetLanguageRegistry;

impl TargetLanguageRegistry {
    /// Resolves the appropriate emitter for a given language identifier or alias.
    pub fn get(lang: &str) -> Box<dyn TargetLanguageEmitter> {
        match lang.to_lowercase().as_str() {
            "typescript" | "ts" => Box::new(TypeScriptEmitter { is_js: false }),
            "javascript" | "js" => Box::new(TypeScriptEmitter { is_js: true }),
            "python" | "py" => Box::new(PythonEmitter),
            "go" | "golang" => Box::new(GoEmitter),
            "dart" => Box::new(DartEmitter),
            "zig" => Box::new(ZigEmitter),
            "csharp" | "c#" | "cs" => Box::new(CsharpEmitter),
            _ => Box::new(RustEmitter),
        }
    }

    /// Returns list of all supported target language names.
    pub fn supported_languages() -> &'static [&'static str] {
        &[
            "rust",
            "go",
            "dart",
            "typescript",
            "javascript",
            "python",
            "zig",
            "csharp",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_registry_resolution_and_terms() {
        let go_emitter = TargetLanguageRegistry::get("go");
        assert_eq!(go_emitter.name(), "go");
        assert_eq!(go_emitter.ecosystem_container_term(), "Package");
        assert_eq!(go_emitter.collection_dir_name(DomainArchetype::SharedLibrary), "packages");
        assert_eq!(go_emitter.collection_dir_name(DomainArchetype::BackendService), "services");

        let dart_emitter = TargetLanguageRegistry::get("dart");
        assert_eq!(dart_emitter.name(), "dart");
        assert_eq!(dart_emitter.ecosystem_container_term(), "Package");
        assert_eq!(dart_emitter.manifest_name(), "pubspec.yaml");

        let rust_emitter = TargetLanguageRegistry::get("rust");
        assert_eq!(rust_emitter.ecosystem_container_term(), "Crate");
        assert_eq!(rust_emitter.collection_dir_name(DomainArchetype::SharedLibrary), "crates");
    }

    #[test]
    fn test_contextual_unit_labels() {
        let emitter = TargetLanguageRegistry::get("go");
        let label = emitter.contextual_unit_label(DomainArchetype::BackendService, "auth_handler");
        assert!(label.contains("API Service Route / Endpoint"));

        let cli_label = emitter.contextual_unit_label(DomainArchetype::CliTool, "migrate_cmd");
        assert!(cli_label.contains("CLI Command"));
    }
}
