//  VISIT.rs
//    by Lut99
//
//  Description:
//!   Defines declarative macros for the `visit` module in `ast-toolkit2`.
//


/***** LIBRARY *****/
/// Declarative macro for efficiently implementing `Visitor`s.
///
/// This makes implementing visitors *almost* ergonomic. Simply define handle functions for
/// specific nodes in your tree and then use this macro to implement `Visitor::visit()` for all of
/// those nodes.
///
/// # Syntax
/// The syntax of this macro looks like:
/// ```plain
/// ($ty:ty $(as $ret:ty)? { $($node:ty $(=> $handler:ident)?),* $(,)? })
/// ```
/// Verbosely:
/// - First, specify the type that you wish to implement `Visitor` for;
/// - Then, optionally, define the `Visitor::Result`-type (after the `as`-keyword);
/// - And open curly braces, for within those:
///   - Write a comma-separated list of all types you have handles for.
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
    ({ __inject_default($ty:ty $(as $ret:ty)? {$first:ty $(, $node:ty $(=> $handler:ident)?)*}, {$($done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::__private::paste! {
            ::ast_toolkit2::visit::visitor_impl!({ __inject_default( $ty $(as $ret)? { $($node $(=> $handler)?),* }, { $($done => $done_handler,)* $first => [<visit_ $first:snake>] }) });
        }
    };
    // Case where we keep the given name
    ({ __inject_default($ty:ty $(as $ret:ty)? {$first:ty => $first_handler:ident $(, $node:ty $(=> $handler:ident)?)*}, {$($done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __inject_default($ty $(as $ret)? { $($node $(=> $handler)?),* }, { $($done => $done_handler,)* $first => $first_handler }) });
    };
    // Base case
    ({ __inject_default($ty:ty $(as $ret:ty)? {}, {$($done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __default_ret($ty $(as $ret)? { $($done => $done_handler),* }) });
    };

    // Case where we inject the default return type
    ({ __default_ret($ty:ty {$($node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __impl($ty as () { $($node => $handler),* }) });
    };
    // Case where we keep the given return type
    ({ __default_ret($ty:ty as $ret:ty {$($node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __impl($ty as $ret { $($node => $handler),* }) });
    };


    /* Impl */
    ({ __impl($ty:ty as $ret:ty { $($node:ty => $handler:ident),* }) }) => {
        impl ::ast_toolkit2::visit::Visitor for $ty {
            type Result = $ret;

            #[inline]
            fn visit<N: ::ast_toolkit2::visit::Visit>(&mut self, node: &N) -> Self::Result {
                // Given special cases
                $(
                    if let ::std::option::Option::Some(node) = ::ast_toolkit2::tree::Node::as_t::<$node>(node) {
                        return self.$handler(node);
                    }
                )*

                // Base case
                ::ast_toolkit2::visit::Visit::visit(node, self)
            }
        }
    };


    /* Public interface */
    ($ty:ty $(as $ret:ty)? { $($node:ty $(=> $handler:ident)?),* $(,)? }) => {
        ::ast_toolkit2::visit::visitor_impl!({ __inject_default($ty $(as $ret)? { $($node $(=> $handler)?),* }, {}) });
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
/// ($ty:ty $(as $ret:ty)? { $($node:ty $(=> $handler:ident)?),* $(,)? })
/// ```
/// Verbosely:
/// - First, specify the type that you wish to implement `VisitorMut` for;
/// - Then, optionally, define the `VisitorMut::Result`-type (after the `as`-keyword);
/// - And open curly braces, for within those:
///   - Write a comma-separated list of all types you have handles for.
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
    ({ __inject_default($ty:ty $(as $ret:ty)? {$first:ty $(, $node:ty $(=> $handler:ident)?)*}, {$($done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::__private::paste! {
            ::ast_toolkit2::visit::visitor_mut_impl!({ __inject_default( $ty $(as $ret)? { $($node $(=> $handler)?),* }, { $($done => $done_handler,)* $first => [<visit_ $first:snake _mut>] }) });
        }
    };
    // Case where we keep the given name
    ({ __inject_default($ty:ty $(as $ret:ty)? {$first:ty => $first_handler:ident $(, $node:ty $(=> $handler:ident)?)*}, {$($done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __inject_default($ty $(as $ret)? { $($node $(=> $handler)?),* }, { $($done => $done_handler,)* $first => $first_handler }) });
    };
    // Base case
    ({ __inject_default($ty:ty $(as $ret:ty)? {}, {$($done:ty => $done_handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __default_ret($ty $(as $ret)? { $($done => $done_handler),* }) });
    };

    // Case where we inject the default return type
    ({ __default_ret($ty:ty {$($node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __impl($ty as () { $($node => $handler),* }) });
    };
    // Case where we keep the given return type
    ({ __default_ret($ty:ty as $ret:ty {$($node:ty => $handler:ident),*}) }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __impl($ty as $ret { $($node => $handler),* }) });
    };


    /* Impl */
    ({ __impl($ty:ty as $ret:ty { $($node:ty => $handler:ident),* }) }) => {
        impl ::ast_toolkit2::visit::VisitorMut for $ty {
            type Result = $ret;

            #[inline]
            fn visit_mut<N: ::ast_toolkit2::visit::VisitMut>(&mut self, node: &mut N) -> Self::Result {
                // Given special cases
                $(
                    if let ::std::option::Option::Some(node) = ::ast_toolkit2::tree::NodeMut::as_t_mut::<$node>(node) {
                        return self.$handler(node);
                    }
                )*

                // Base case
                ::ast_toolkit2::visit::VisitMut::visit_mut(node, self)
            }
        }
    };


    /* Public interface */
    ($ty:ty $(as $ret:ty)? { $($node:ty $(=> $handler:ident)?),* $(,)? }) => {
        ::ast_toolkit2::visit::visitor_mut_impl!({ __inject_default($ty $(as $ret)? { $($node $(=> $handler)?),* }, {}) });
    };
}
