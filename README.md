rhai-doc
========

rhai-doc is a tool for auto-generating documentation for rhai source code.

Usage
-----

```
USAGE:
    rhai-doc [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -D, --dest <DIRECTORY>     Set the destination for the documentation output.
    -d, --dir <DIRECTORY>      Set the Rhai source file directory.
    -p, --pages <DIRECTORY>    Set the directory where the markdown files are located.
```

Features
--------
- [x] Generate documentation from markdown in Rhai source files.
- [x] Create general purpose documentation pages.
- [ ] Search documentation for functions.
- [ ] Create warnings for undocumented functions, parameters, and etc.

License
-------

Licensed under either of the following, at your choice:

* [Apache License, Version 2.0](https://github.com/semirix/rhai-doc/blob/master/LICENSE-APACHE.txt), or
* [MIT license](https://github.com/semirix/rhai-doc/blob/master/LICENSE-MIT.txt)

Unless explicitly stated otherwise, any contribution intentionally submitted
for inclusion in this crate, as defined in the Apache-2.0 license, shall
be dual-licensed as above, without any additional terms or conditions.