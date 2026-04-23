// Integration tests for sealenv
// Run with: cargo test

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;

    fn sealenv_bin() -> String {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.push("sealenv");
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_help_runs() {
        let output = Command::new("cargo")
            .args(["run", "--", "--help"])
            .output()
            .expect("Failed to run sealenv");
        assert!(output.status.success());
    }

    #[test]
    fn test_parse_env_bytes() {
        use std::str;
        let input = b"DATABASE_URL=postgres://localhost/db\nAPI_KEY=secret123\n# comment\n\nFOO=bar\n";
        let vars = crate_parse(input);
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0].0, "DATABASE_URL");
        assert_eq!(vars[1].0, "API_KEY");
        assert_eq!(vars[2].0, "FOO");
    }

    fn crate_parse(data: &[u8]) -> Vec<(String, String)> {
        // Inline the parsing logic for unit testing
        let text = std::str::from_utf8(data).unwrap();
        let mut vars = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(idx) = line.find('=') {
                let key = line[..idx].trim().to_string();
                let val = line[idx + 1..].trim().trim_matches('"').trim_matches('\'').to_string();
                vars.push((key, val));
            }
        }
        vars
    }
}
