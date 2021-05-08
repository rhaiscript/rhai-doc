// This is a Rhai script containing many functions.
//
// Functions within the same script file can cross-link to each other via
// the [`func_name`] format.

/// Another test
private fn thing() {}

// This is not a doc-comment so this should not show up in the documentation.
fn foo() {}

/// # Thing
/// Test
/// Oh wow! *one* **two** _three_ __four__
/// This is the last line of the paragraph.
///
/// Check out this function: [`baz`]
///
/// - list
/// - of
/// - things
///
/// 1. List
/// 2. Of
/// 3. Things
fn foo(bar, baz) {}

/// > This is a quote for
/// > that spans two lines.
///
/// These functions are great: [`thing`] and [`foo`]
///
/// # Example
/// ```
/// // Embedded comment
/// foo(1, 2);
///
/// let x = 42;
/// let y = [1, 2, 3, x, x + 1];
/// x += y.len.abs();
/// print(`Result: ${x}`);
/// ```
fn baz() {}

/// [This is a link](http://example.com)
///
/// ![Logo](logo.png)
fn foo(x) {}
