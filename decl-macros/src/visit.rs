//  VISIT.rs
//    by Lut99
//
//  Description:
//!   Defines declarative macros for the `visit` module in `ast-toolkit2`.
//


/***** LIBRARY *****/
/// Convenience macro for creating a group of nodes.
///
/// It is shorthand for defining an enum with the given visibility, name and variants that always
/// has the same shape. Then, for that shape, `Group` is implemented.
///
/// This allows one to quickly define groups of nodes for usage in visiting.
///
/// # Syntax
/// The syntax is:
/// ```plain
/// $(#[$($attrs:tt)*])* $vis:vis group $name:ident <'a> { $($(#[$($var_attrs:tt)*])* $var:ident(&'a $ty:ty)),* $(,)? }
/// ```
/// Verbosely:
/// - First, give any attribute for the new enum (incl. docstrings);
/// - Then, given the visibility (e.g., nothing or `pub`), the `group` keyword and the new enum's
///   name;
/// - Continue with a lifetime `<'a>`;
/// - Proceed to open braces, wherein you:
///   - Give a comma-separated list of attributes, variant names and the types part of this group.
///     Note all types _must_ be references with the `'a` lifetime.
///
/// # Examples
/// For example, the following:
/// ```ignore
/// use ast_toolkit2::visit::group;
///
/// group! {
///     /// An example group.
///     pub group Ex1<'a> {
///         /// The first variant.
///         First(&'a String),
///         /// The second variant.
///         Second(&'a u64),
///     }
/// }
/// assert_eq!(Ex1::as_group::<String>(&"Hello, world!".to_string()), Some(Ex1::First(&"Hello, world!".to_string())));
/// assert_eq!(Ex1::as_group::<u64>(&42), Some(Ex1::Second(&42)));
/// assert_eq!(Ex1::as_group::<()>(&()), None);
/// ```
///
/// This generates a type similar to:
/// ```ignore
/// /// An example group.
/// pub enum Ex1<'a> {
///     /// The first variant.
///     First(&'a String),
///     /// The second variant.
///     Second(&'a u64),
/// }
/// impl<'a> Group<'a> for Ex1<'a> {
///     #[inline]
///     fn as_group<N: Node>(node: &'a N) -> Option<Self> {
///         if let Some(node) = node.as_t::<String>() {
///             Some(Self::First(node))
///         } else if let Some(node) = node.as_t::<String>() {
///             Some(Self::Second(node))
///         } else {
///             None
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! group {
    ($(#[$($attrs:tt)*])* $vis:vis group $name:ident <'a> { $($(#[$($var_attrs:tt)*])* $var:ident(&'a $ty:ty)),* $(,)? }) => {
        $(#[$($attrs)*])*
        $vis enum $name<'a> {
            $($(#[$($var_attrs)*])* $var(&'a $ty),)*
        }
        impl<'a> ::ast_toolkit2::visit::Group<'a> for $name<'a> {
            #[inline]
            fn as_group<N: ::ast_toolkit2::tree::Node>(node: &'a N) -> Option<Self> {
                $(if let ::std::option::Option::Some(node) = ::ast_toolkit2::tree::Node::as_t::<$ty>(node) {
                    return ::std::option::Option::Some(Self::$var(node));
                })*
                ::std::option::Option::None
            }
        }
    };
}



/// Convenience macro for creating a group of nodes.
///
/// It is shorthand for defining an enum with the given visibility, name and variants that always
/// has the same shape. Then, for that shape, `Group` is implemented.
///
/// This allows one to quickly define groups of nodes for usage in visiting.
///
/// # Syntax
/// The syntax is:
/// ```plain
/// $(#[$($attrs:tt)*])* $vis:vis group $name:ident <'a> { $($(#[$($var_attrs:tt)*])* $var:ident(&'a mut $ty:ty)),* $(,)? }
/// ```
/// Verbosely:
/// - First, give any attribute for the new enum (incl. docstrings);
/// - Then, given the visibility (e.g., nothing or `pub`), the `group` keyword and the new enum's
///   name;
/// - Continue with a lifetime `<'a>`;
/// - Proceed to open braces, wherein you:
///   - Give a comma-separated list of attributes, variant names and the types part of this group.
///     Note all types _must_ be mutable references with the `'a` lifetime.
///
/// # Examples
/// For example, the following:
/// ```ignore
/// use ast_toolkit2::visit::group_mut;
///
/// group_mut! {
///     /// An example group.
///     pub group Ex1<'a> {
///         /// The first variant.
///         First(&'a mut String),
///         /// The second variant.
///         Second(&'a mut u64),
///     }
/// }
/// assert_eq!(Ex1::as_group_mut::<String>(&mut "Hello, world!".to_string()), Some(Ex1::First(&mut "Hello, world!".to_string())));
/// assert_eq!(Ex1::as_group_mut::<u64>(&mut 42), Some(Ex1::Second(&mut 42)));
/// assert_eq!(Ex1::as_group_mut::<()>(&mut ()), None);
/// ```
///
/// This generates a type similar to:
/// ```ignore
/// /// An example group.
/// pub enum Ex1<'a> {
///     /// The first variant.
///     First(&'a String),
///     /// The second variant.
///     Second(&'a u64),
/// }
/// impl<'a> GroupMut<'a> for Ex1<'a> {
///     #[inline]
///     fn as_group_mut<N: NodeMut>(node: &'a mut N) -> Option<Self> {
///         if let Some(node) = node.as_t_mut::<String>() {
///             Some(Self::First(node))
///         } else if let Some(node) = node.as_t_mut::<String>() {
///             Some(Self::Second(node))
///         } else {
///             None
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! group_mut {
    ($(#[$($attrs:tt)*])* $vis:vis group $name:ident <'a> { $($(#[$($var_attrs:tt)*])* $var:ident(&'a mut $ty:ty)),* $(,)? }) => {
        $(#[$($attrs)*])*
        $vis enum $name<'a> {
            $($(#[$($var_attrs)*])* $var(&'a $ty),)*
        }
        impl<'a> ::ast_toolkit2::visit::GroupMut<'a> for $name<'a> {
            #[inline]
            fn as_group_mut<N: ::ast_toolkit2::tree::NodeMut>(node: &'a mut N) -> Option<Self> {
                $(if let ::std::option::Option::Some(node) = ::ast_toolkit2::tree::NodeMut::as_t_mut::<$ty>(node) {
                    return ::std::option::Option::Some(Self::$var(node));
                })*
                ::std::option::Option::None
            }
        }
    };
}



/// Declarative macro for efficiently implementing `Visitor`s.
///
/// This makes implementing visitors *almost* ergonomic. Simply define handle functions for
/// specific nodes in your tree and then use this macro to implement `Visitor::visit()` for all of
/// those nodes.
///
/// # Syntax
/// The syntax of this macro looks like:
/// ```plain
/// ($ty:ty $(as $ret:ty)? { $($({group})? $node:ty $(=> $handler:ident)?),* $(,)? })
/// ```
/// Verbosely:
/// - First, specify the type that you wish to implement `Visitor` for;
/// - Then, optionally, define the `Visitor::Result`-type (after the `as`-keyword);
/// - And open curly braces, for within those:
///   - Write a comma-separated list of all types you have handles for.
///     If you precede any type with `{group}`, then it will be matched on `Group::as_group()`
///     instead of as a `Node`.
///     You can give the name of the handler after a `=>`, or omit it to default to `visit_TY`,
///     where `TY` is the snake-case version of the given type (thanks
///     [paste](https://docs.rs/paste)!).
///
/// # Examples
/// For example, consider the following derived implementation:
/// ```ignore
/// use ast_toolkit2::loc::Located;
/// use ast_toolkit2::tree::Node;
/// use ast_toolkit2::visit::{Visit, Visitor, visitor_impl};
///
///
/// struct Branch {
///     leaf1: Leaf,
///     leaf2: Leaf,
/// }
/// impl Located for Branch { /* ... */ }
/// impl Node for Branch { /* ... */ }
/// impl Visit for Branch { /* ... */ }
///
/// struct Leaf;
/// impl Located for Leaf { /* ... */ }
/// impl Node for Leaf { /* ... */ }
/// impl Visit for Leaf { /* ... */ }
///
///
/// struct CountLeafsTraversal(usize);
/// impl CountLeafsTraversal {
///     fn visit_leaf(&mut self, _leaf: &Leaf) {
///         self.0 += 1;
///     }
/// }
/// visitor_impl!(CountLeafsTraversal as usize {
///     Leaf => visit_leaf,
///     // Alternatively:
///     Leaf,
/// });
///
///
/// assert_eq!(CountLeafsTraversal.visit(&Branch { leaf1: Leaf, leaf2: Leaf }), 2);
/// ```
///
/// The macro call `visitor_impl!(...)` expands to something like:
/// ```ignore
/// impl Visitor for CountLeafsTraversal {
///     type Result = usize;
///
///     fn visit<N: Visit>(&mut self, node: &N) -> usize {
///         if let Some(leaf) = node.as_t::<Leaf>() {
///             self.visit_leaf(leaf)
///         } else {
///             node.visit(self)
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! visitor_impl {
    /* Private processing */
    // Case where we inject the default name
    ({ __inject_default($ty:ty $(as $ret:ty)? {$({$first_group:ident})? $first:ty $(, $({$group:ident})? $node:ty $(=> $handler:ident)?)*}, {$($({$done_group:ident})? $done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::__private::paste! {
            ::ast_toolkit2::visit::visitor_impl!({ __inject_default( $ty $(as $ret)? { $($({group})? $node $(=> $handler)?),* }, { $($({$done_group})? $done => $done_handler,)* $({$first_group})? $first => [<visit_ $first:snake>] }) });
        }
    };
    // Case where we keep the given name
    ({ __inject_default($ty:ty $(as $ret:ty)? {$({$first_group:ident})? $first:ty => $first_handler:ident $(, $({$group:ident})? $node:ty $(=> $handler:ident)?)*}, {$($({$done_group:ident})? $done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __inject_default($ty $(as $ret)? { $($({$group})? $node $(=> $handler)?),* }, { $($({$done_group})? $done => $done_handler,)* $({$first_group})? $first => $first_handler }) });
    };
    // Base case
    ({ __inject_default($ty:ty $(as $ret:ty)? {}, {$($({$done_group:ident})? $done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __default_ret($ty $(as $ret)? { $($({$done_group})? $done => $done_handler),* }) });
    };

    // Case where we inject the default return type
    ({ __default_ret($ty:ty {$($({$group:ident})? $node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __handle_groups($ty as () { $($({$group})? $node => $handler),* }, {}) });
    };
    // Case where we keep the given return type
    ({ __default_ret($ty:ty as $ret:ty {$($({$group:ident})? $node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __handle_groups($ty as $ret { $($({$group})? $node => $handler),* }, {}) });
    };

    // Case where we inject normal match
    ({ __handle_groups($ty:ty as $ret:ty {$first:ty => $first_handler:ident $(, $({$group:ident})? $node:ty => $handler:ident)* }, { $({$($done_match_fn:tt)*} => $done_handler:ident),* }) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __handle_groups($ty as $ret {$($({$group})? $node => $handler),* }, { $({$($done_match_fn)*} => $done_handler,)* {::ast_toolkit2::tree::Node::as_t::<$first>} => $first_handler }) });
    };
    // Case where we inject group match
    ({ __handle_groups($ty:ty as $ret:ty {{group} $first:ty => $first_handler:ident $(, $({$group:ident})? $node:ty => $handler:ident)* }, { $({$($done_match_fn:tt)*} => $done_handler:ident),* }) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __handle_groups($ty as $ret {$($({$group})? $node => $handler),* }, { $({$($done_match_fn)*} => $done_handler,)* {<$first as ::ast_toolkit2::visit::Group>::as_group} => $first_handler }) });
    };
    // Base case
    ({ __handle_groups($ty:ty as $ret:ty {}, { $({$($done_match_fn:tt)*} => $done_handler:ident),* }) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __impl($ty as $ret { $({$($done_match_fn)*} => $done_handler),* }) });
    };


    /* Impl */
    ({ __impl($ty:ty as $ret:ty { $({$($match_fn:tt)*} => $handler:ident),* }) }) => {
        impl ::ast_toolkit2::visit::Visitor for $ty {
            type Result = $ret;

            #[inline]
            fn visit<N: ::ast_toolkit2::visit::Visit>(&mut self, node: &N) -> Self::Result {
                // Given special cases
                $(
                    if let ::std::option::Option::Some(node) = $($match_fn)*(node) {
                        return self.$handler(node);
                    }
                )*

                // Base case
                ::ast_toolkit2::visit::Visit::visit(node, self)
            }
        }
    };


    /* Public interface */
    ($ty:ty $(as $ret:ty)? { $($({$group:ident})? $node:ty $(=> $handler:ident)?),* $(,)? }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __inject_default($ty $(as $ret)? { $($({$group})? $node $(=> $handler)?),* }, {}) });
    };
}



/// Declarative macro for efficiently implementing `VisitorMut`s.
///
/// This makes implementing visitors *almost* ergonomic. Simply define handle functions for
/// specific nodes in your tree and then use this macro to implement `VisitorMut::visit_mut()` for
/// all of those nodes.
///
/// # Syntax
/// The syntax of this macro looks like:
/// ```plain
/// ($ty:ty $(as $ret:ty)? { $($({group})? $node:ty $(=> $handler:ident)?),* $(,)? })
/// ```
/// Verbosely:
/// - First, specify the type that you wish to implement `VisitorMut` for;
/// - Then, optionally, define the `VisitorMut::Result`-type (after the `as`-keyword);
/// - And open curly braces, for within those:
///   - Write a comma-separated list of all types you have handles for.
///     If you precede any type with `{group}`, then it will be matched on
///     `GroupMut::as_group_mut()` instead of as a `NodeMut`.
///     You can give the name of the handler after a `=>`, or omit it to default to `visit_TY_mut`,
///     where `TY` is the snake-case version of the given type (thanks
///     [paste](https://docs.rs/paste)!).
///
/// # Examples
/// For example, consider the following derived implementation:
/// ```ignore
/// use ast_toolkit2::loc::Located;
/// use ast_toolkit2::tree::Node;
/// use ast_toolkit2::visit::{Visit, VisitMut, VisitorMut, visitor_mut_impl};
///
///
/// struct Branch {
///     leaf1: Leaf,
///     leaf2: Leaf,
/// }
/// impl Located for Branch { /* ... */ }
/// impl Node for Branch { /* ... */ }
/// impl Visit for Branch { /* ... */ }
/// impl VisitMut for Branch { /* ... */ }
///
/// struct Leaf;
/// impl Located for Leaf { /* ... */ }
/// impl Node for Leaf { /* ... */ }
/// impl Visit for Leaf { /* ... */ }
/// impl VisitMut for Leaf { /* ... */ }
///
///
/// struct CountLeafsTraversal(usize);
/// impl CountLeafsTraversal {
///     fn visit_leaf_mut(&mut self, _leaf: &Leaf) {
///         self.0 += 1;
///     }
/// }
/// visitor_mut_impl!(CountLeafsTraversal as usize {
///     Leaf => visit_leaf_mut,
///     // Alternatively:
///     Leaf,
/// });
///
///
/// assert_eq!(CountLeafsTraversal.visit_mut(&mut Branch { leaf1: Leaf, leaf2: Leaf }), 2);
/// ```
///
/// The macro call `visitor_mut_impl!(...)` expands to something like:
/// ```ignore
/// impl VisitorMut for CountLeafsTraversal {
///     type Result = usize;
///
///     fn visit_mut<N: VisitMut>(&mut self, node: &mut N) -> usize {
///         if let Some(leaf) = node.as_t_mut::<Leaf>() {
///             self.visit_leaf_mut(leaf)
///         } else {
///             node.visit_mut(self)
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! visitor_mut_impl {
    /* Private processing */
    // Case where we inject the default name
    ({ __inject_default($ty:ty $(as $ret:ty)? {$({$first_group:ident})? $first:ty $(, $({$group:ident})? $node:ty $(=> $handler:ident)?)*}, {$($({$done_group:ident})? $done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::__private::paste! {
            ::ast_toolkit2::visit::visitor_mut_impl!({ __inject_default( $ty $(as $ret)? { $($({group})? $node $(=> $handler)?),* }, { $($({$done_group})? $done => $done_handler,)* $({$first_group})? $first => [<visit_ $first:snake _mut>] }) });
        }
    };
    // Case where we keep the given name
    ({ __inject_default($ty:ty $(as $ret:ty)? {$({$first_group:ident})? $first:ty => $first_handler:ident $(, $({$group:ident})? $node:ty $(=> $handler:ident)?)*}, {$($({$done_group:ident})? $done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __inject_default($ty $(as $ret)? { $($({$group})? $node $(=> $handler)?),* }, { $($({$done_group})? $done => $done_handler,)* $({$first_group})? $first => $first_handler }) });
    };
    // Base case
    ({ __inject_default($ty:ty $(as $ret:ty)? {}, {$($({$done_group:ident})? $done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __default_ret($ty $(as $ret)? { $($({$done_group})? $done => $done_handler),* }) });
    };

    // Case where we inject the default return type
    ({ __default_ret($ty:ty {$($({$group:ident})? $node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __handle_groups($ty as () { $($({$group})? $node => $handler),* }, {}) });
    };
    // Case where we keep the given return type
    ({ __default_ret($ty:ty as $ret:ty {$($({$group:ident})? $node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __handle_groups($ty as $ret { $($({$group})? $node => $handler),* }, {}) });
    };

    // Case where we inject normal match
    ({ __handle_groups($ty:ty as $ret:ty {$first:ty => $first_handler:ident $(, $({$group:ident})? $node:ty => $handler:ident)* }, { $({$($done_match_fn:tt)*} => $done_handler:ident),* }) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __handle_groups($ty as $ret {$($({$group})? $node => $handler),* }, { $({$($done_match_fn)*} => $done_handler,)* {::ast_toolkit2::tree::NodeMut::as_t_mut::<$first>} => $first_handler }) });
    };
    // Case where we inject group match
    ({ __handle_groups($ty:ty as $ret:ty {{group} $first:ty => $first_handler:ident $(, $({$group:ident})? $node:ty => $handler:ident)* }, { $({$($done_match_fn:tt)*} => $done_handler:ident),* }) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __handle_groups($ty as $ret {$($({$group})? $node => $handler),* }, { $({$($done_match_fn)*} => $done_handler,)* {<$first as ::ast_toolkit2::visit::GroupMut>::as_group_mut} => $first_handler }) });
    };
    // Base case
    ({ __handle_groups($ty:ty as $ret:ty {}, { $({$($done_match_fn:tt)*} => $done_handler:ident),* }) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __impl($ty as $ret { $({$($done_match_fn)*} => $done_handler),* }) });
    };


    /* Impl */
    ({ __impl($ty:ty as $ret:ty { $({$($match_fn:tt)*} => $handler:ident),* }) }) => {
        impl ::ast_toolkit2::visit::VisitorMut for $ty {
            type Result = $ret;

            #[inline]
            fn visit_mut<N: ::ast_toolkit2::visit::VisitMut>(&mut self, node: &mut N) -> Self::Result {
                // Given special cases
                $(
                    if let ::std::option::Option::Some(node) = $($match_fn)*(node) {
                        return self.$handler(node);
                    }
                )*

                // Base case
                ::ast_toolkit2::visit::VisitMut::visit_mut(node, self)
            }
        }
    };


    /* Public interface */
    ($ty:ty $(as $ret:ty)? { $($({$group:ident})? $node:ty $(=> $handler:ident)?),* $(,)? }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __inject_default($ty $(as $ret)? { $($({$group})? $node $(=> $handler)?),* }, {}) });
    };
}
