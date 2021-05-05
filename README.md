rhai-doc
========

`rhai-doc` is a tool for auto-generating documentation for rhai source code.

It supports writing [MarkDown] documentation in source comments and creating general purpose
documentation pages with [MarkDown].

See an example [here](https://rhai.rs/rhai-doc).


CLI Interface
-------------

```text
USAGE:
    rhai-doc [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
        --verbose    Prints diagnostic messages

OPTIONS:
        --config <FILE>        Sets the configuration file (default rhai.toml)
    -D, --dest <DIRECTORY>     Sets the destination for the documentation output
    -d, --dir <DIRECTORY>      Sets the Rhai scripts (*.rhai) directory
    -p, --pages <DIRECTORY>    Sets the directory where MarkDown (*.md) page files are located
```

Install the tool using `cargo`:

```sh
cargo install --path .
```


Configuration File
------------------

To get started, you need a configuration file.

It is usually named `rhai.toml`, or you can specify one via the `-config` option.

An example of what a `rhai.toml` file should look like:

```toml
name = "My Rhai Project"                # project name
color = [246, 119, 2]                   # theme color
index = "home.md"                       # this file becomes 'index.html`
root = "https://example.com/docs/"      # root URL for generated site
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

### Configuration parameters

- `name`: The name of the project, if any. It's the title that shows up on the documentation pages.
- `color`: RGB values of the theme color for the generated docs, if any.
- `index`: The main [MarkDown] file that will become `index.html`.
- `root`: The root part of the URLs generated as part of the documentation.
- `icon`: The location of a custom icon file, if any.
- `stylesheet`: The location of a custom stylesheet, if any.
- `extension`: The extension of the source files `rhai-doc` will look for.
- `google_analytics`: Google Analytics ID, if any.
- `[[links]]`: External links to other sites of relevance.
  - `name`: Title of external link.
  - `link`: URL of external link.


Pages
-----

By default, `rhai-doc` will generate documentation pages from a `pages` sub-directory
under the current directory.

Alternatively, you can specify another location with the `--pages` option in the CLI.

To ensure that that the generated documents have an index page, you *must* specify the `index`
[MarkDown] file in `rhai.toml`, and that file will be renamed to `index.html`.


Features
--------

- [x] Generate documentation from [MarkDown] in Rhai source files.
- [x] Create general purpose documentation pages.
- [ ] Search documentation for functions.
- [ ] Create warnings for undocumented functions, parameters, and etc.


License
-------

Licensed under either of the following, at your choice:

- [Apache License, Version 2.0](https://github.com/semirix/rhai-doc/blob/master/LICENSE-APACHE.txt), or
- [MIT license](https://github.com/semirix/rhai-doc/blob/master/LICENSE-MIT.txt)

Unless explicitly stated otherwise, any contribution intentionally submitted
for inclusion in this crate, as defined in the Apache-2.0 license,
shall be dual-licensed as above, without any additional terms or conditions.


[Markdown]: https://en.wikipedia.org/wiki/Markdown
