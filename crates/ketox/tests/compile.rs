#[test]
fn export_contracts() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/valid.rs");
    tests.compile_fail("tests/ui/unsupported.rs");
    tests.compile_fail("tests/ui/async.rs");
    tests.compile_fail("tests/ui/arguments.rs");
}
