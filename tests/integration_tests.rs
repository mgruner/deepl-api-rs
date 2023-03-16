use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

#[test]
// Check that help is shown if invoked without any arguments.
fn test_help() {
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.assert()
        .code(2)
        .stdout(predicate::eq(""))
        .stderr(predicate::str::contains("Usage"));
}

// Check success/failure depending on available DEEPL_API_KEY.
#[test]
fn test_auth() {
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("usage-information").assert().success();

    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.env("DEEPL_API_KEY", "")
        .arg("usage-information")
        .assert()
        .code(1)
        .stdout(predicate::eq(""))
        .stderr(predicate::eq(
            "Error: no DEEPL_API_KEY found. Please provide your API key in this environment variable.\n",
        ));

    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.env("DEEPL_API_KEY", "false")
        .arg("usage-information")
        .assert()
        .code(1)
        .stdout(predicate::eq(""))
        .stderr(predicate::eq(
            "Error: Authorization failed, is your API key correct?\n",
        ));
}

#[test]
fn test_usage_information() {
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("usage-information")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Available characters per billing period:",
        ))
        .stderr(predicate::eq(""));
}

#[test]
fn test_languages() {
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("languages")
        .assert()
        .success()
        .stdout(predicate::str::contains("RU    (Russian)"))
        .stderr(predicate::eq(""));
}

#[test]
fn test_translate() {
    // Missing target language
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("translate")
        .write_stdin("Please go home.")
        .assert()
        .code(2)
        .stdout(predicate::eq(""))
        .stderr(predicate::str::contains(
            "following required arguments were not provided:",
        ));

    // STDIN/STDOUT
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("translate")
        .arg("--source-language")
        .arg("EN")
        .arg("--target-language")
        .arg("DE")
        .write_stdin("Please go home.")
        .assert()
        .success()
        .stdout(predicate::eq("Bitte gehen Sie nach Hause.\n"))
        .stderr(predicate::eq(""));

    // Invalid target language
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("translate")
        .arg("--source-language")
        .arg("EN")
        .arg("--target-language")
        .arg("FALSE")
        .write_stdin("Please go home.")
        .assert()
        .code(1)
        .stdout(predicate::eq(""))
        .stderr(predicate::eq("Error: An error occurred while communicating with the DeepL server: \'Value for \'target_lang\' not supported.: \'.\n"));

    // Via valid files
    let tempdir = assert_fs::TempDir::new().unwrap();
    let input_file = tempdir.child("input.txt");
    input_file.write_str("Please go home.").unwrap();
    let output_file = tempdir.child("output.txt");

    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("translate")
        .arg("--source-language")
        .arg("EN")
        .arg("--target-language")
        .arg("DE")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--output-file")
        .arg(output_file.path())
        .write_stdin("Please go home.")
        .assert()
        .success()
        .stdout(predicate::eq(""))
        .stderr(predicate::eq(""));

    output_file.assert("Bitte gehen Sie nach Hause.");

    // Invalid input file path.
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("translate")
        .arg("--source-language")
        .arg("EN")
        .arg("--target-language")
        .arg("DE")
        .arg("--input-file")
        .arg("nonexisting/file/path")
        .arg("--output-file")
        .arg(output_file.path())
        .write_stdin("Please go home.")
        .assert()
        .code(1)
        .stdout(predicate::eq(""))
        .stderr(predicate::eq(
            "Error: No such file or directory (os error 2)\n",
        ));

    // Invalid output file path.
    let mut cmd = Command::cargo_bin("deepl").unwrap();
    cmd.arg("translate")
        .arg("--source-language")
        .arg("EN")
        .arg("--target-language")
        .arg("DE")
        .arg("--input-file")
        .arg(input_file.path())
        .arg("--output-file")
        .arg("nonexisting/file/path")
        .write_stdin("Please go home.")
        .assert()
        .code(1)
        .stdout(predicate::eq(""))
        .stderr(predicate::eq(
            "Error: No such file or directory (os error 2)\n",
        ));
}
