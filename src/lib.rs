#![deny(
    missing_debug_implementations,
    rust_2018_idioms,
    unused_imports,
    dead_code
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

// macros used internally
#[macro_use]

mod byte_stream;