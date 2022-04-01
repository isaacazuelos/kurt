# Testing Plan

> A description of how tests are structured, and a rationale.

You can run all the tests with `cargo test --all`. We'll have a different,
non-cargo test runner eventually for end-to-end testing.

This `tests` directory is for tests of the binary produced by the `bin`
directory. While we try to lean what `cargo test` can do for us, Rust currently
doesn't have great end-to-end binary testing so we'll have to come up with
something soon.

Most rust files should have [unit tests][ut] at the bottom of the file. The test
names should describe what's being tested.

Anything public for a crate should have [documentation tests][dt] which show
intended use, but like other functions there should be normal unit tests for
other behaviour (failure states, edge cases, etc.) as well.

I've largely moved away from having multiple examples or doing a lot of doc
tests because they're noticeable _much_ slower to run. They make good
documentation, and we should make sure our documentation is correct, but we can
keep most of the tests as regular unit tests most of the time to make tests run
quicker. For example, the 14 doc tests in syntax take 1.95 seconds to run but
the 45 unit tests take 0.00

Crates should also have [integration tests] for their public behaviour. For
example, the `syntax` module has integration tests for the lexer and parser
which outline a bunch of their behaviour that might be unintuitive or bug prone.
If you find a bug, write an integration test!

[ut]: https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html
[dt]: https://doc.rust-lang.org/rust-by-example/testing/doc_testing.html
[it]: https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html
