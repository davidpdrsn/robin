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

/// A macro for verifying that a type implements a certain trait. Normally used to verify that
/// something is either `Send` or `Sync`.
///
/// ```
/// # #[macro_use]
/// # extern crate robin;
/// #
/// # fn main() {
/// struct Foo;
///
/// test_type_impls!(foo_impls_send, Foo, Send);
/// test_type_impls!(foo_impls_sync, Foo, Sync);
/// # }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! test_type_impls {
    ($name:ident, $type:ty, $trait:ty) => {
        #[allow(warnings)]
        fn $name() {
            let x: $type = unimplemented!();
            let _: &$trait = &x;
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! robin_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            setup();
            $body();
            teardown();
        }
    };
}
