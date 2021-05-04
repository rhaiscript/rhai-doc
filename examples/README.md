Examples
========


Install `rhai-doc`
------------------

Before trying out the examples, the `rhai-doc` tool must be installed.

```sh
cargo install --path .
```


Generate Documentation Site
---------------------------

To generate documentation for any example set, do the following:

```sh
cd examples/basic

rhai-doc
```

The documentation site will be generated in the `dist` sub-directory (unless a different directory
is provided via the `-D` option).
