use super::reporter::Reporter;
use crate::checker::types::diagnostics::{Diagnostic, DiagnosticCategory};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> std::io::Result<Self> {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("tsz_cli_reporter_test_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent directory");
    }
    std::fs::write(path, contents).expect("failed to write file");
}

#[test]
fn reporter_formats_diagnostic_with_location() {
    let temp = TempDir::new().expect("temp dir");
    let file_path = temp.path.join("src/main.ts");
    write_file(
        &file_path,
        "let x = 1;\nlet y = 2;\n",
    );

    let diagnostic = Diagnostic {
        file: file_path.to_string_lossy().into_owned(),
        start: 11,
        length: 1,
        message_text: "Cannot find name 'y'.".to_string(),
        category: DiagnosticCategory::Error,
        code: 2304,
        related_information: Vec::new(),
    };

    let mut reporter = Reporter::new(false);
    let output = reporter.format_diagnostic(&diagnostic);

    let expected_prefix = format!("{}:2:1 - error TS2304: ", diagnostic.file);
    assert!(output.starts_with(&expected_prefix), "{output}");
}

#[test]
fn reporter_omits_code_when_missing() {
    let diagnostic = Diagnostic {
        file: "missing.ts".to_string(),
        start: 0,
        length: 0,
        message_text: "Parse error".to_string(),
        category: DiagnosticCategory::Error,
        code: 0,
        related_information: Vec::new(),
    };

    let mut reporter = Reporter::new(false);
    let output = reporter.format_diagnostic(&diagnostic);

    assert!(output.contains("- error: Parse error"), "{output}");
}
