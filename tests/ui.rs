#[test]
pub fn ui() {
	use trybuild::TestCases;
	let t: TestCases = TestCases::new();
	t.compile_fail("tests/f_*");
	t.compile_fail("tests/p_*");
}
