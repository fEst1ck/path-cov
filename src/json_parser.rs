use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub entry_block: u32,
    pub exit_blocks: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub module_name: String,
    pub functions: Vec<Function>,
}

pub fn parse_json_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<ModuleInfo>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut buffer = String::new();
    let mut modules = Vec::new();
    let mut depth = 0;

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            continue;
        }

        buffer.push_str(&line);
        buffer.push('\n');

        // Count opening and closing braces
        for c in trimmed.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }

        // When we reach depth 0, we have a complete JSON object
        if depth == 0 && !buffer.trim().is_empty() {
            match serde_json::from_str(&buffer) {
                Ok(module) => {
                    modules.push(module);
                    buffer.clear();
                }
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("JSON parsing error: {}", e),
                    ));
                }
            }
        }
    }

    Ok(modules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_json_file() {
        let json_content = r#"{
            "module_name": "test.c",
            "functions": [
                {
                    "name": "func1",
                    "entry_block": 1,
                    "exit_blocks": [2]
                }
            ]
        }
        {
            "module_name": "test2.c",
            "functions": [
                {
                    "name": "func2",
                    "entry_block": 3,
                    "exit_blocks": [4, 5]
                }
            ]
        }"#;

        let temp_file = NamedTempFile::new().unwrap();
        write(temp_file.path(), json_content).unwrap();

        let result = parse_json_file(temp_file.path()).unwrap();
        assert_eq!(result.len(), 2);

        // Check first module
        let first_module = &result[0];
        assert_eq!(first_module.module_name, "test.c");
        assert_eq!(first_module.functions.len(), 1);
        let first_func = &first_module.functions[0];
        assert_eq!(first_func.name, "func1");
        assert_eq!(first_func.entry_block, 1);
        assert_eq!(first_func.exit_blocks, vec![2]);

        // Check second module
        let second_module = &result[1];
        assert_eq!(second_module.module_name, "test2.c");
        assert_eq!(second_module.functions.len(), 1);
        let second_func = &second_module.functions[0];
        assert_eq!(second_func.name, "func2");
        assert_eq!(second_func.entry_block, 3);
        assert_eq!(second_func.exit_blocks, vec![4, 5]);
    }
} 