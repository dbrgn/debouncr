# Simple Debounce

[![Build status][workflow-badge]][workflow]
[![Crates.io Version][crates-io-badge]][crates-io]
[![Crates.io Downloads][crates-io-download-badge]][crates-io-download]
![No Std][no-std-badge]

A simple and efficient `no_std` input debouncer that uses integer bit shifting
to debounce inputs. The algorithm can detect rising and falling edges and only
requires 1 byte of RAM for detecting up to 8 consecutive high/low states or 2
bytes of RAM for detecting up to 16 consecutive high/low states.

The algorithm is based on the [Ganssle Guide to
Debouncing](http://www.ganssle.com/debouncing-pt2.htm) (section "An
Alternative").

Docs: https://docs.rs/debouncr

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.


<!-- Badges -->
[workflow]: https://github.com/dbrgn/debouncr/actions?query=workflow%3ACI
[workflow-badge]: https://img.shields.io/github/workflow/status/dbrgn/debouncr/CI/master
[crates-io]: https://crates.io/crates/debouncr
[crates-io-badge]: https://img.shields.io/crates/v/debouncr.svg?maxAge=3600
[crates-io-download]: https://crates.io/crates/debouncr
[crates-io-download-badge]: https://img.shields.io/crates/d/debouncr.svg?maxAge=3600
[no-std-badge]: https://img.shields.io/badge/no__std-yes-blue
