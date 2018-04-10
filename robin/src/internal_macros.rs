/// A useful macro for doing "puts debugging".
///
/// This:
///
/// ```
/// # #[macro_use]
/// # extern crate robin;
/// #
/// # fn main() {
/// #[derive(Debug)]
/// struct Foo;
///
/// let a = Foo;
///
/// p!(a);
/// # }
/// ```
///
/// Is the same as:
///
/// ```
/// # #[macro_use]
/// # extern crate robin;
/// #
/// # fn main() {
/// #[derive(Debug)]
/// struct Foo;
///
/// let a = Foo;
///
/// println!("a = {:?}", a);
/// # }
/// ```
///
/// The macro also accepts multiple arguments and will print each on its own line:
///
/// ```
/// # #[macro_use]
/// # extern crate robin;
/// #
/// # fn main() {
/// #[derive(Debug)]
/// struct Foo;
///
/// let a = Foo;
/// let b = Foo;
/// let c = Foo;
///
/// p!(a, b, c);
/// # }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! p {
    ( $($var:ident),* ) => {
        $(
            println!(concat!(stringify!($var), " = {:?}"), $var);
        )*
    }
}
