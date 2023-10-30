use rustc_test as test;

mod punycode;
mod uts46;

fn main() {
    let mut tests = Vec::new();
    {
        let mut add_test = |name, run| {
            tests.push(test::TestDescAndFn {
                desc: test::TestDesc::new(test::DynTestName(name)),
                testfn: run,
            })
        };
        punycode::collect_tests(&mut add_test);
        uts46::collect_tests(&mut add_test);
    }
    test::test_main(&std::env::args().collect::<Vec<_>>(), tests)
}
