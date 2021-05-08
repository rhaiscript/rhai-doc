`rhai-doc` - Generates HTML documentation from Rhai script files
==============================================================

[![License](https://img.shields.io/crates/l/rhai)](https://github.com/license/rhaiscript/rhai-doc)
[![crates.io](https://img.shields.io/crates/v/rhai-doc?logo=rust)](https://crates.io/crates/rhai-doc/)
[![crates.io](https://img.shields.io/crates/d/rhai-doc?logo=rust)](https://crates.io/crates/rhai-doc/)

`rhai-doc` is a tool for auto-generating documentation for [Rhai] scripts.

It supports writing [MarkDown] documentation in [doc-comments] of [Rhai] scripts and creating
general-purpose documentation pages.

See an example [here](https://rhai.rs/rhai-doc).


CLI Interface
-------------

```text
USAGE:
    rhai-doc [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -a, --all        Generates documentation for all functions, including private ones
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v               Use multiple to set the level of verbosity: 1 = silent, 2,3 (default) = full

OPTIONS:
    -c, --config <FILE>        Sets the configuration file (default rhai.toml)
    -D, --dest <DIRECTORY>     Sets the destination for the documentation output
    -d, --dir <DIRECTORY>      Sets the Rhai scripts (*.rhai) directory
    -p, --pages <DIRECTORY>    Sets the directory where MarkDown (*.md) pages files are located

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand(s)
    new     Generates a new 'rhai.toml' configuration file
```


Installation
------------

### Install from `crates.io`

```sh
cargo install rhai-doc
```

### Install from source

```sh
cargo install --path .
```


Configuration File
------------------

To get started, you need a configuration file.

It is usually named `rhai.toml`, or you can specify one via the `-config` option.

To generate a skeleton `rhai.toml`, use the `new` command:

```sh
rhai-doc new
```

### Example

```toml
name = "My Rhai Project"                # project name
color = [246, 119, 2]                   # theme color
root = "/docs/"                         # root URL for generated site
index = "home.md"                       # this file becomes 'index.html`
icon = "logo.svg"                       # project icon
stylesheet = "my_stylesheet.css"        # custom stylesheet
extension = "rhai"                      # script extension
google_analytics = "G-ABCDEF1234"       # Google Analytics ID

[[links]]                               # external link for 'Blog'
name = "Blog"
link = "https://example.com/blog"

[[links]]                               # external link for 'Tools'
name = "Tools"
link = "https://example.com/tools"
```

### Configuration options

- `name`: The name of the project, if any. It's the title that shows up on the documentation pages.
- `color`: RGB values of the theme color for the generated docs, if any.
- `root`: The root URL generated as part of the documentation, if any.
- `index`: The main [MarkDown] file, if any, that will become `index.html`.
- `icon`: The location of a custom icon file, if any.
- `stylesheet`: The location of a custom stylesheet, if any.
- `extension`: The extension of the script files `rhai-doc` will look for (default `.rhai`).
- `google_analytics`: Google Analytics ID, if any.
- `[[links]]`: External links, if any, to other sites of relevance.
  - `name`: Title of external link.
  - `link`: URL of external link.


Doc-Comments
------------

[Rhai] supports [doc-comments] in [MarkDown] format on script-defined
[functions](https://rhai.rs/book/language/functions.html).

```rust
/// This function calculates a **secret number**!
///
/// Formula provided from this [link](https://secret_formula.com/calc_secret_number).
///
/// # Scale Factor
/// Uses a scale factor obtained by calling [`get_contribution_factor`].
///
/// # Parameters
/// `seed` - random seed to start the calculation
///
/// # Returns
/// The secret number!
///
/// # Exceptions
/// Throws when the seed is not positive.
///
/// # Example
/// ```
/// let secret = calc_secret_number(42);
/// ```
fn calc_secret_number(seed) {
    if seed <= 0 {
        throw "the seed must be positive!";
    }

    let factor = get_contribution_factor(seed);

    // Some very complex code skipped ...
    // ...
}

/// This function is private and will not be included
/// unless the `-a` flag is used.
private fn get_multiply_factor() {
    42
}

/// This function calculates a scale factor for use
/// in the [`calc_secret_number`] function.
fn get_contribution_factor(x) {
    x * get_multiply_factor()
}
```


Inter-Script Links
------------------

Functions documentation can cross-link to each other within the same script file.

A link in the format ``[`my_func`]`` is automatically expanded to link to the documentation of
the target function (in this case `my_func`).


Pages
-----

By default, `rhai-doc` will generate documentation pages from [MarkDown] documents within a
`pages` sub-directory under the scripts directory.

Alternatively, you can specify another location via the `--pages` option.


Features
--------

- [x] Generate documentation from [MarkDown] [doc-comments] in [Rhai] script files.
- [x] Create general-purpose documentation pages.
- [ ] Text search.
- [ ] Linter for undocumented functions, parameters, etc.


License
-------

Licensed under either of the following, at your choice:

- [Apache License, Version 2.0](https://github.com/semirix/rhai-doc/blob/master/LICENSE-APACHE.txt), or
- [MIT license](https://github.com/semirix/rhai-doc/blob/master/LICENSE-MIT.txt)

Unless explicitly stated otherwise, any contribution intentionally submitted
for inclusion in this crate, as defined in the Apache-2.0 license,
shall be dual-licensed as above, without any additional terms or conditions.


[MarkDown]: https://en.wikipedia.org/wiki/Markdown
[Rhai]: https://rhai.rs
[doc-comments]: https://rhai.rs/book/language/doc-comments.html
