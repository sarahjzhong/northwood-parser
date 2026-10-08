use std::process::Command;

#[test]
fn executable_matches_example() {
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/example.txt");
    let output = Command::new(env!("CARGO_BIN_EXE_parser"))
        .arg(example)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"aaa 4 12\nbbb 8\nzzz 0\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_arguments_and_missing_file_fail_cleanly() {
    let executable = env!("CARGO_BIN_EXE_parser");
    for args in [
        vec![],
        vec!["/definitely-missing-northwood-input"],
        vec!["fixtures/example.txt", "unexpected"],
    ] {
        let output = Command::new(executable).args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}
