macro_rules! test_filter {
    ($args: expr, $stdin: expr, $stdout: expr) => {
        test_filter!($args, $stdin, $stdout, "")
    };
    ($args: expr, $stdin: expr, $stdout: expr, $stderr: expr) => {
        let mut cmd = ::assert_cmd::Command::cargo_bin("mddux")?;
        let assert = cmd.args($args).write_stdin($stdin).assert();
        assert.success().stdout($stdout).stderr($stderr);
    };
}

#[test]
fn test_run() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["run", "tests/fixtures/example.spec.md"],
        "",
        include_str!("fixtures/example.md")
    );
    Ok(())
}


#[test]
fn test_run_console() -> Result<(), Box<dyn std::error::Error>> {
    test_filter!(
        ["run-console", "tests/fixtures/example.console"],
        "",
        include_str!("fixtures/example.expected.console")
    );
    Ok(())
}
