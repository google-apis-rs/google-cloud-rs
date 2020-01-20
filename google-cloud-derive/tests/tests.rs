use trybuild::TestCases;

#[test]
fn tests() {
    let tests = TestCases::new();
    tests.pass("tests/01-simple.rs");
    tests.pass("tests/02-nested.rs");
}
