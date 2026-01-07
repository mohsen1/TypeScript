use clap::Parser;

use super::args::{CliArgs, Module, Target};

#[test]
fn parses_defaults() {
    let args = CliArgs::try_parse_from(["stc"]).expect("default args should parse");

    assert_eq!(args.target, Target::EsNext);
    assert_eq!(args.module, Module::None);
    assert!(args.out_dir.is_none());
    assert!(!args.strict);
    assert!(!args.no_emit);
    assert!(args.files.is_empty());
}

#[test]
fn parses_common_flags() {
    let args = CliArgs::try_parse_from([
        "stc",
        "--target",
        "es2020",
        "--module",
        "commonjs",
        "--outDir",
        "dist",
        "--strict",
        "--noEmit",
        "src/index.ts",
    ])
    .expect("flagged args should parse");

    assert_eq!(args.target, Target::Es2020);
    assert_eq!(args.module, Module::CommonJs);
    assert_eq!(args.out_dir.as_deref(), Some(std::path::Path::new("dist")));
    assert!(args.strict);
    assert!(args.no_emit);
    assert_eq!(args.files, vec![std::path::PathBuf::from("src/index.ts")]);
}
