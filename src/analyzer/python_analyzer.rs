use super::{static_analysis::FunctionAnalysis, CodeAnalysis, LanguageAnalyzer};
use crate::error::FolderSummaryError;
use crate::llm::LLM;
use async_trait::async_trait;
use regex::Regex;
// use rustpython_parser::{parser, ast};

pub struct PythonAnalyzer;

impl PythonAnalyzer {
    fn extract_imports(content: &str) -> Vec<String> {
        let import_regex = Regex::new(r"(?m)^(?:from\s+(\S+)\s+)?import\s+(.+)$").unwrap();
        import_regex
            .captures_iter(content)
            .map(|cap| {
                let from = cap.get(1).map_or("", |m| m.as_str());
                let import = cap.get(2).map_or("", |m| m.as_str());
                if from.is_empty() {
                    import.to_string()
                } else {
                    format!("{} from {}", import, from)
                }
            })
            .collect()
    }

    fn extract_functions(content: &str) -> Vec<FunctionAnalysis> {
        let function_regex =
            Regex::new(r"(?m)^(\s*)def\s+(\w+)\s*\((.*?)\)(?:\s*->\s*([^:]+))?\s*:").unwrap();
        let mut functions = Vec::new();

        for caps in function_regex.captures_iter(content) {
            let indentation = caps.get(1).map_or("", |m| m.as_str());
            let name = caps.get(2).map_or("", |m| m.as_str()).to_string();
            let params = caps.get(3).map_or("", |m| m.as_str());
            let return_type = caps.get(4).map_or("", |m| m.as_str());
            let signature = format!(
                "def {}({}){}:",
                name,
                params,
                if return_type.is_empty() {
                    "".to_string()
                } else {
                    " -> ".to_string() + return_type
                }
            );

            let function_body = Self::extract_function_body(content, indentation, caps.get(0).unwrap().end());
            let lines_of_code = function_body.lines().count();

            functions.push(FunctionAnalysis {
                name,
                signature,
                types: return_type.to_string(),
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

    fn extract_function_body(content: &str, indentation: &str, start: usize) -> String {
        let lines: Vec<&str> = content[start..].lines().collect();
        let mut body = Vec::new();
        let mut in_body = false;

        for line in lines {
            if !in_body && line.trim().is_empty() {
                continue;
            }
            if !in_body {
                in_body = true;
            }
            if in_body && (!line.starts_with(indentation) || line.trim().is_empty()) {
                break;
            }
            body.push(line);
        }

        body.join("\n")
    }

    fn extract_types(content: &str) -> Vec<String> {
        let class_regex = Regex::new(r"(?m)^\s*class\s+(\w+)").unwrap();
        class_regex
            .captures_iter(content)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
    }

    fn extract_exports(_content: &str) -> Vec<String> {
        // Python doesn't have explicit exports, so we'll leave this empty
        Vec::new()
    }

    fn calculate_cyclomatic_complexity(function_body: &str) -> usize {
        let control_flow_regex = Regex::new(r"\b(if|elif|for|while|except|and|or)\b").unwrap();
        1 + control_flow_regex.find_iter(function_body).count()
    }

    // fn extract_function_info(func: &ast::StmtFunctionDef) -> FunctionAnalysis {
    //     let name = func.name.to_string();
    //     let params = func.args.args.iter().map(|arg| arg.arg.to_string()).collect::<Vec<_>>().join(", ");
    //     let signature = format!("def {}({})", name, params);
    //     let types = func.args.args.iter()
    //         .filter_map(|arg| arg.annotation.as_ref().map(|ann| ann.to_string()))
    //         .collect::<Vec<_>>()
    //         .join(", ");
    //     let body = func.body.iter().map(|stmt| stmt.to_string()).collect::<Vec<_>>().join("\n");
    //     let lines_of_code = body.lines().count();
    
    //     FunctionAnalysis {
    //         name,
    //         signature,
    //         types,
    //         body: Some(body),
    //         lines_of_code,
    //         cyclomatic_complexity: Self::calculate_cyclomatic_complexity(body), // Simplified, you may want to implement a proper calculation
    //         parameters: func.args.args.len(),
    //         returns: func.returns.is_some(),
    //         summary: None,
    //     }
    // }
}

#[async_trait]
impl LanguageAnalyzer for PythonAnalyzer {
    fn can_analyze(&self, file_path: &str) -> bool {
        file_path.ends_with(".py")
    }

    fn analyze(&self, content: &str) -> Result<CodeAnalysis, FolderSummaryError> {
        Ok(CodeAnalysis {
            imports: Self::extract_imports(content),
            functions: Self::extract_functions(content),
            types: Self::extract_types(content),
            exports: Self::extract_exports(content),
        })
    }

    async fn summarize(
        &self,
        analysis: &CodeAnalysis,
        llm: &Box<dyn LLM>,
    ) -> Result<CodeAnalysis, FolderSummaryError> {
        let mut summarized = analysis.clone();
        for func in &mut summarized.functions {
            if func.lines_of_code > 6 {
                let prompt = format!(
                    "Summarize the following Python function:\n\nName: {}\nSignature: {}\nTypes: {}\nBody: {}",
                    func.name,
                    func.signature,
                    func.types,
                    func.body.as_deref().unwrap_or("(Function body not available)")
                );
                func.summary = Some(llm.summarize(&prompt).await?);
            }
        }
        Ok(summarized)
    }
}
