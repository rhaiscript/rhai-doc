# rhai-doc
rhai-doc is a tool for auto-generating documentation for rhai source code. It
supports writing markdown documentation in source comments and creating general
purpose documentation pages with markdown.

## Usage
### CLI Interface
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

### Creating `rhai.toml`
To get started, you need to create a `rhai.toml` file where your rhai source
files are located. An example of what a `rhai.toml` file would look like goes as
follows:

```
name = "My Rhai Project"
colour = [246, 119, 2]
index = "home.md"
root = "https://example.com/docs/"
icon = "logo.svg"
extension = "rhai"

[[links]]
name = "Blog"
link = "https://example.com/blog"

[[links]]
name = "Tools"
link = "https://example.com/tools"
```

### The `rhai.toml` parameters
- `name`: The name of your rhai project. It's the title that shows up on the
  documentation pages.
- `colour`: The RGB value of the theme colour for the generated docs.
- `index`: The markdown file that will become the `index.html`.
- `root`: The root part of the URLs generated as part of the documentation.
- `icon`: The location of a custom icon file.
- `extension`: The extension of the source files `rhai-doc` will look for.
- `links`: Any external links to other sites of relevance.

### Setting up pages
By default, `rhai-doc` will generate documentation pages from a `pages` folder
in the directory your `rhai.toml` file; alternatively, you can specify another
location with the `--pages` option in the CLI. To ensure that that the generated
documents have an index page you *must* specify the `index` markdown file in
`rhai.toml`.

## Features
- [x] Generate documentation from markdown in Rhai source files.
- [x] Create general purpose documentation pages.
- [ ] Search documentation for functions.
- [ ] Create warnings for undocumented functions, parameters, and etc.

## License
Licensed under either of the following, at your choice:

* [Apache License, Version 2.0](https://github.com/semirix/rhai-doc/blob/master/LICENSE-APACHE.txt), or
* [MIT license](https://github.com/semirix/rhai-doc/blob/master/LICENSE-MIT.txt)

Unless explicitly stated otherwise, any contribution intentionally submitted
for inclusion in this crate, as defined in the Apache-2.0 license, shall
be dual-licensed as above, without any additional terms or conditions.