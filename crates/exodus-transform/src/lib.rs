//! Deterministic Python-to-Rust transformation engine for Project Exodus.

use exodus_core::{Diagnostic, MigrationOutcome, Result};
use exodus_fallback::{FallbackGenerator, FallbackRecord, FallbackStrategy};
use exodus_parser::{ClassDef, FunctionDef, ParsedModule, ParsedRepository};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Transformation configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOptions {
    pub target_rust_edition: String,
    pub derive_common_traits: bool,
    pub generate_stubs_for_unsupported: bool,
}

impl Default for TransformOptions {
    fn default() -> Self {
        Self {
            target_rust_edition: "2021".to_string(),
            derive_common_traits: true,
            generate_stubs_for_unsupported: true,
        }
    }
}

/// Request to transform a parsed module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRequest {
    pub parsed_module: ParsedModule,
    pub options: TransformOptions,
}

/// Result of transforming a module to Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResult {
    pub rust_source: String,
    pub module_name: String,
    pub file_path: PathBuf,
    pub outcome: MigrationOutcome,
    pub fallbacks: Vec<FallbackRecord>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Core deterministic transformation engine.
pub struct TransformationEngine {
    fallback_gen: FallbackGenerator,
}

impl TransformationEngine {
    pub fn new() -> Self {
        Self {
            fallback_gen: FallbackGenerator::new(),
        }
    }

    /// Maps Python type annotations to idiomatic Rust types.
    pub fn map_type(py_type: &str) -> String {
        let trimmed = py_type.trim();
        if trimmed.is_empty() {
            return "()".to_string();
        }

        if trimmed == "int" {
            return "i64".to_string();
        } else if trimmed == "float" {
            return "f64".to_string();
        } else if trimmed == "str" {
            return "String".to_string();
        } else if trimmed == "bool" {
            return "bool".to_string();
        } else if trimmed == "bytes" {
            return "Vec<u8>".to_string();
        } else if trimmed == "None" || trimmed == "NoneType" {
            return "()".to_string();
        } else if trimmed == "Any" {
            return "serde_json::Value".to_string();
        }

        // Handle Optional[T]
        if let Some(inner) = trimmed
            .strip_prefix("Optional[")
            .and_then(|s| s.strip_suffix(']'))
        {
            return format!("Option<{}>", Self::map_type(inner));
        }

        // Handle List[T] / list[T]
        if let Some(inner) = trimmed
            .strip_prefix("List[")
            .or_else(|| trimmed.strip_prefix("list["))
            .and_then(|s| s.strip_suffix(']'))
        {
            return format!("Vec<{}>", Self::map_type(inner));
        }

        // Handle Dict[K, V] / dict[K, V]
        if let Some(inner) = trimmed
            .strip_prefix("Dict[")
            .or_else(|| trimmed.strip_prefix("dict["))
            .and_then(|s| s.strip_suffix(']'))
        {
            let parts: Vec<&str> = inner.splitn(2, ',').collect();
            if parts.len() == 2 {
                return format!(
                    "std::collections::HashMap<{}, {}>",
                    Self::map_type(parts[0]),
                    Self::map_type(parts[1])
                );
            }
        }

        // Handle Set[T] / set[T]
        if let Some(inner) = trimmed
            .strip_prefix("Set[")
            .or_else(|| trimmed.strip_prefix("set["))
            .and_then(|s| s.strip_suffix(']'))
        {
            return format!("std::collections::HashSet<{}>", Self::map_type(inner));
        }

        // Custom class / identifier
        trimmed.to_string()
    }

    /// Indentation-aware Python statement transformer.
    fn transform_body_code(body: &str, _is_async: bool) -> String {
        let mut rust_lines = Vec::new();
        let lines: Vec<&str> = body.lines().collect();

        let print_re = Regex::new(r#"print\((.*)\)"#).unwrap();
        let len_re = Regex::new(r#"len\(([^)]+)\)"#).unwrap();
        let range_re = Regex::new(r#"range\(([^,]+),\s*([^)]+)\)"#).unwrap();
        let range_one_re = Regex::new(r#"range\(([^)]+)\)"#).unwrap();
        let append_re = Regex::new(r#"([a-zA-Z0-9_]+)\.append\(([^)]+)\)"#).unwrap();

        let mut indent_stack: Vec<usize> = Vec::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || (trimmed.starts_with("\"\"\"") && trimmed.ends_with("\"\"\""))
            {
                continue;
            }

            let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();

            // Close blocks if indent decreased
            while let Some(&top_indent) = indent_stack.last() {
                if current_indent < top_indent {
                    indent_stack.pop();
                    let close_indent = " ".repeat(indent_stack.len() * 4 + 4);
                    rust_lines.push(format!("{close_indent}}}"));
                } else {
                    break;
                }
            }

            let mut trans = trimmed.to_string();

            // Boolean literals & None
            trans = trans.replace("True", "true");
            trans = trans.replace("False", "false");
            trans = trans.replace(" and ", " && ");
            trans = trans.replace(" or ", " || ");
            trans = trans.replace("not ", "!");

            // Python builtins to Rust
            trans = print_re
                .replace_all(&trans, "println!(\"{}\", $1)")
                .to_string();
            trans = len_re.replace_all(&trans, "$1.len()").to_string();
            trans = range_re.replace_all(&trans, "$1..$2").to_string();
            trans = range_one_re.replace_all(&trans, "0..$1").to_string();
            trans = append_re.replace_all(&trans, "$1.push($2)").to_string();

            // List literal: result = [] -> let mut result = Vec::new()
            if trans.contains(" = []") {
                trans = trans.replace(" = []", " = Vec::new()");
                if !trans.starts_with("let ") && !trans.starts_with("self.") {
                    trans = format!("let mut {trans}");
                }
            } else if trans.contains(" = ")
                && !trans.starts_with("let ")
                && !trans.starts_with("self.")
                && !trans.starts_with("if ")
            {
                trans = format!("let mut {trans}");
            }

            let effective_indent = " ".repeat(indent_stack.len() * 4 + 4);

            if trans.starts_with("elif ") {
                if let Some(top_indent) = indent_stack.pop() {
                    let close_indent = " ".repeat(indent_stack.len() * 4 + 4);
                    let cond = trans.strip_prefix("elif ").unwrap().trim_end_matches(':');
                    rust_lines.push(format!("{close_indent}}} else if {cond} {{"));
                    indent_stack.push(top_indent);
                }
            } else if trans == "else:" {
                if let Some(top_indent) = indent_stack.pop() {
                    let close_indent = " ".repeat(indent_stack.len() * 4 + 4);
                    rust_lines.push(format!("{close_indent}}} else {{"));
                    indent_stack.push(top_indent);
                }
            } else if trans.starts_with("if ") {
                let cond = trans.strip_prefix("if ").unwrap().trim_end_matches(':');
                rust_lines.push(format!("{effective_indent}if {cond} {{"));
                indent_stack.push(current_indent + 4);
            } else if trans.starts_with("for ") && trans.contains(" in ") {
                let stripped = trans.strip_prefix("for ").unwrap().trim_end_matches(':');
                rust_lines.push(format!("{effective_indent}for {stripped} {{"));
                indent_stack.push(current_indent + 4);
            } else if trans.starts_with("while ") {
                let cond = trans.strip_prefix("while ").unwrap().trim_end_matches(':');
                rust_lines.push(format!("{effective_indent}while {cond} {{"));
                indent_stack.push(current_indent + 4);
            } else if trans.starts_with("return ") {
                let expr = trans.strip_prefix("return ").unwrap().trim_end_matches(';');
                rust_lines.push(format!("{effective_indent}return {expr};"));
            } else if trans == "pass" {
                rust_lines.push(format!("{effective_indent}// pass"));
            } else {
                let stmt = trans.trim_end_matches(';');
                rust_lines.push(format!("{effective_indent}{stmt};"));
            }
        }

        // Close remaining open blocks
        while indent_stack.pop().is_some() {
            let close_indent = " ".repeat(indent_stack.len() * 4 + 4);
            rust_lines.push(format!("{close_indent}}}"));
        }

        if rust_lines.is_empty() {
            "    // Default implementation\n".to_string()
        } else {
            let joined = rust_lines.join("\n");
            format!("{joined}\n")
        }
    }

    /// Transforms a single FunctionDef to Rust code.
    fn transform_function(&self, func: &FunctionDef) -> (String, Option<FallbackRecord>) {
        let is_async = func.is_async;
        let async_prefix = if is_async { "async " } else { "" };
        let ret_type = func
            .return_type
            .as_deref()
            .map(Self::map_type)
            .unwrap_or_else(|| "()".to_string());

        let mut param_strs = Vec::new();
        if func.is_method {
            param_strs.push("&mut self".to_string());
        }

        for p in &func.parameters {
            let p_type = p
                .type_annotation
                .as_deref()
                .map(Self::map_type)
                .unwrap_or_else(|| "serde_json::Value".to_string());
            param_strs.push(format!("{}: {}", p.name, p_type));
        }

        let params_joined = param_strs.join(", ");
        let ret_clause = if ret_type == "()" {
            String::new()
        } else {
            format!(" -> {ret_type}")
        };

        let mut doc = String::new();
        if let Some(d) = &func.docstring {
            doc.push_str(&format!("/// {}\n", d.replace('\n', "\n/// ")));
        }

        let body_code = Self::transform_body_code(&func.body_snippet, is_async);

        let code = format!(
            "{doc}pub {async_prefix}fn {}({}){} {{\n{}}}\n",
            func.name, params_joined, ret_clause, body_code
        );

        (code, None)
    }

    /// Transforms a ClassDef to a Rust struct and impl block.
    fn transform_class(&self, class: &ClassDef) -> (String, Vec<FallbackRecord>) {
        let mut out = String::new();
        let mut fallbacks = Vec::new();

        if let Some(doc) = &class.docstring {
            out.push_str(&format!("/// {}\n", doc.replace('\n', "\n/// ")));
        }

        out.push_str(
            "#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]\n",
        );
        out.push_str(&format!("pub struct {} {{\n", class.name));

        // Fields
        for field in &class.fields {
            out.push_str(&format!("    pub {}: serde_json::Value,\n", field));
        }
        if class.fields.is_empty() {
            let mut init_fields = HashSet::new();
            if let Some(init_m) = class.methods.iter().find(|m| m.name == "__init__") {
                for p in &init_m.parameters {
                    let p_type = p
                        .type_annotation
                        .as_deref()
                        .map(Self::map_type)
                        .unwrap_or_else(|| "serde_json::Value".to_string());
                    init_fields.insert((p.name.clone(), p_type));
                }
            }
            if init_fields.is_empty() {
                out.push_str("    // No explicit fields\n");
            } else {
                for (fname, ftype) in init_fields {
                    out.push_str(&format!("    pub {}: {},\n", fname, ftype));
                }
            }
        }
        out.push_str("}\n\n");

        // Impl block
        out.push_str(&format!("impl {} {{\n", class.name));

        for method in &class.methods {
            if method.name == "__init__" {
                let mut params = Vec::new();
                let mut field_inits = Vec::new();
                for p in &method.parameters {
                    let p_type = p
                        .type_annotation
                        .as_deref()
                        .map(Self::map_type)
                        .unwrap_or_else(|| "serde_json::Value".to_string());
                    params.push(format!("{}: {}", p.name, p_type));
                    field_inits.push(format!("            {}: {},", p.name, p.name));
                }
                out.push_str(&format!(
                    "    pub fn new({}) -> Self {{\n        Self {{\n{}\n        }}\n    }}\n\n",
                    params.join(", "),
                    field_inits.join("\n")
                ));
            } else {
                let (m_code, fb) = self.transform_function(method);
                if let Some(fb) = fb {
                    fallbacks.push(fb);
                }
                let indented = m_code
                    .lines()
                    .map(|l| format!("    {l}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                out.push_str(&format!("{indented}\n\n"));
            }
        }

        out.push_str("}\n");

        (out, fallbacks)
    }

    /// Transforms a parsed module to Rust source code.
    pub fn transform_module(&self, request: &TransformRequest) -> Result<TransformResult> {
        let module = &request.parsed_module;
        let mut rust_source = String::new();
        let mut fallbacks = Vec::new();
        let diagnostics = module.diagnostics.clone();

        rust_source.push_str(&format!(
            "//! Transformed Rust module `{}` generated by Project Exodus.\n\n",
            module.module_name
        ));

        // Common imports
        rust_source.push_str("use std::collections::{HashMap, HashSet};\n\n");

        // Transform unsupported constructs into explicit fallback stubs
        for unsupp in &module.unsupported_constructs {
            let fb = self.fallback_gen.generate_stub(
                &format!("{}::{}", module.module_name, unsupp.construct_kind),
                &format!("{}_fallback", unsupp.construct_kind),
                "/* unsupported dynamic arguments */",
                "Result<(), String>",
                &unsupp.reason,
                Some(unsupp.evidence.span.clone()),
            );
            rust_source.push_str(&fb.generated_code);
            rust_source.push('\n');
            fallbacks.push(fb);
        }

        // Transform classes
        for class in &module.classes {
            let (cls_code, cls_fallbacks) = self.transform_class(class);
            rust_source.push_str(&cls_code);
            rust_source.push('\n');
            fallbacks.extend(cls_fallbacks);
        }

        // Transform top-level functions
        for func in &module.functions {
            let (fn_code, fn_fallback) = self.transform_function(func);
            rust_source.push_str(&fn_code);
            rust_source.push('\n');
            if let Some(fb) = fn_fallback {
                fallbacks.push(fb);
            }
        }

        let outcome = if !fallbacks.is_empty() {
            if fallbacks
                .iter()
                .any(|f| f.strategy == FallbackStrategy::TypedFailureStub)
            {
                MigrationOutcome::Degraded
            } else {
                MigrationOutcome::Compatible
            }
        } else {
            MigrationOutcome::Verified
        };

        Ok(TransformResult {
            rust_source,
            module_name: module.module_name.clone(),
            file_path: module.file_path.clone(),
            outcome,
            fallbacks,
            diagnostics,
        })
    }

    /// Transforms an entire parsed repository.
    pub fn transform_repository(
        &self,
        parsed_repo: &ParsedRepository,
    ) -> Result<Vec<TransformResult>> {
        let mut results = Vec::new();
        let options = TransformOptions::default();

        for module in &parsed_repo.modules {
            let req = TransformRequest {
                parsed_module: module.clone(),
                options: options.clone(),
            };
            results.push(self.transform_module(&req)?);
        }

        Ok(results)
    }
}

impl Default for TransformationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use exodus_parser::{PythonParser, SourceParser};
    use std::path::Path;

    #[test]
    fn test_map_types() {
        assert_eq!(TransformationEngine::map_type("int"), "i64");
        assert_eq!(TransformationEngine::map_type("str"), "String");
        assert_eq!(
            TransformationEngine::map_type("Optional[int]"),
            "Option<i64>"
        );
        assert_eq!(TransformationEngine::map_type("List[str]"), "Vec<String>");
        assert_eq!(
            TransformationEngine::map_type("Dict[str, int]"),
            "std::collections::HashMap<String, i64>"
        );
    }

    #[test]
    fn test_transform_module_clean() {
        let py_code = r#"
def add(a: int, b: int) -> int:
    return a + b

class Counter:
    def __init__(self, start: int):
        self.count = start

    def get_count(self) -> int:
        return self.count
"#;
        let parser = PythonParser::new();
        let parsed = parser
            .parse_source(Path::new("counter.py"), py_code)
            .unwrap();

        let engine = TransformationEngine::new();
        let req = TransformRequest {
            parsed_module: parsed,
            options: TransformOptions::default(),
        };

        let result = engine.transform_module(&req).unwrap();
        assert!(result
            .rust_source
            .contains("pub fn add(a: i64, b: i64) -> i64"));
        assert!(result.rust_source.contains("pub struct Counter"));
        assert!(result
            .rust_source
            .contains("pub fn new(start: i64) -> Self"));
        assert_eq!(result.outcome, MigrationOutcome::Verified);
    }
}
