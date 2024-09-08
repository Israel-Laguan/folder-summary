use regex::Regex;

use super::{static_analysis::FunctionAnalysis, CodeAnalysis, LanguageAnalyzer};
use crate::error::FolderSummaryError;
use crate::llm::LLM;
use async_trait::async_trait;

pub struct JavaScriptAnalyzer;

impl JavaScriptAnalyzer {
    fn extract_imports(content: &str) -> Vec<String> {
        let import_regex = Regex::new(r#"(?m)^(?:import\s+(?:(?:\{[^}]*\}|\*\s+as\s+\w+|\w+)(?:\s*,\s*(?:\{[^}]*\}|\*\s+as\s+\w+|\w+))*\s+from\s+)?['"](.+?)['"]|(?:const|let|var)\s+(?:\{[^}]*\}|\w+)\s*=\s*require\s*\(\s*['"](.+?)['"]\s*\))(?:;|\s*$)"#).unwrap();
        import_regex
            .captures_iter(content)
            .filter_map(|cap| cap.get(1).or(cap.get(2)))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_functions(content: &str) -> Vec<FunctionAnalysis> {
        let function_regex = Regex::new(r"(?m)^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)\s*\((.*?)\)(?:\s*:\s*([^{]+))?\s*\{").unwrap();
        let arrow_function_regex = Regex::new(r"(?m)^\s*(?:export\s+)?(?:const|let|var)\s+(\w+)\s*=\s*(?:async\s+)?\((.*?)\)(?:\s*:\s*([^=]+))?\s*=>").unwrap();

        let mut functions = Vec::new();

        for caps in function_regex.captures_iter(content) {
            let name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let params = caps.get(2).map_or("", |m| m.as_str());
            let return_type = caps.get(3).map_or("", |m| m.as_str());
            let signature = format!(
                "const {} = ({}){}=> ",
                name,
                params,
                if return_type.is_empty() {
                    "".to_string()
                } else {
                    format!(": {}", return_type)
                }
            );

            let function_body = Self::extract_function_body(content, caps.get(0).unwrap().end());
            let lines_of_code = function_body.lines().count();

            functions.push(FunctionAnalysis {
                name,
                signature,
                types: "".to_string(),
                body: Some(function_body.clone()),
                lines_of_code,
                cyclomatic_complexity: Self::calculate_cyclomatic_complexity(&function_body),
                parameters: params.split(',').filter(|p| !p.trim().is_empty()).count(),
                returns: !return_type.is_empty(),
                summary: None,
            });
        }

        for caps in arrow_function_regex.captures_iter(content) {
            let name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let params = caps.get(2).map_or("", |m| m.as_str());
            let return_type = caps.get(3).map_or("", |m| m.as_str());
            let signature = format!(
                "const {} = ({}){}=> ",
                name,
                params,
                if return_type.is_empty() {
                    "".to_string()
                } else {
                    ": ".to_string() + return_type
                }
            );
            let function_body = Self::extract_function_body(content, caps.get(0).unwrap().end());
            let lines_of_code = function_body.lines().count();

            functions.push(FunctionAnalysis {
                name,
                signature,
                types: "".to_string(),
                body: Some(function_body.clone()),
                lines_of_code,
                cyclomatic_complexity: Self::calculate_cyclomatic_complexity(&function_body),
                parameters: params.split(',').filter(|p| !p.trim().is_empty()).count(),
                returns: !return_type.is_empty(),
                summary: None,
            });
        }

        functions
    }

    fn extract_types(content: &str) -> Vec<String> {
        let type_regex = Regex::new(r"(?m)^\s*(?:export\s+)?(?:type|interface)\s+(\w+)").unwrap();
        type_regex
            .captures_iter(content)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_exports(content: &str) -> Vec<String> {
        let export_regex =
            Regex::new(r"(?m)^export\s+(?:const|let|var|function|class|type|interface)\s+(\w+)")
                .unwrap();
        export_regex
            .captures_iter(content)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn calculate_cyclomatic_complexity(function_body: &str) -> usize {
        let control_flow_regex =
            Regex::new(r"\b(if|else|for|while|do|switch|case|catch|&&|\|\|)\b").unwrap();
        1 + control_flow_regex.find_iter(function_body).count()
    }
    fn extract_function_body(content: &str, start: usize) -> String {
        let mut brace_count = 0;
        let mut body = String::new();
        let lines: Vec<&str> = content[start..].lines().collect();

        for line in lines {
            body.push_str(line);
            body.push('\n');
            brace_count += line.matches('{').count() as i32;
            brace_count -= line.matches('}').count() as i32;
            if brace_count == 0 {
                break;
            }
        }

        body
    }
}

#[async_trait]
impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        file_path.ends_with(".js") || file_path.ends_with(".ts")
    }

    fn analyze(&self, content: &str) -> Result<CodeAnalysis, FolderSummaryError> {
        Ok(CodeAnalysis {
            imports: Self::extract_imports(content),
            functions: Self::extract_functions(content),
            types: Self::extract_types(content),
            exports: Self::extract_exports(content),
        })
    }

    async fn summarize(&self, analysis: &CodeAnalysis, llm: &Box<dyn LLM>) -> Result<CodeAnalysis, FolderSummaryError> {
        let mut summarized = analysis.clone();
        for func in &mut summarized.functions {
            if func.lines_of_code > 6 {
                let prompt = format!(
                    "Summarize the following JavaScript/TypeScript function:\n\nName: {}\nSignature: {}\nBody: {}",
                    func.name,
                    func.signature,
                    func.body.as_deref().unwrap_or("(Function body not available)")
                );
                func.summary = Some(llm.summarize(&prompt).await?);
            }
        }
        Ok(summarized)
    }
}