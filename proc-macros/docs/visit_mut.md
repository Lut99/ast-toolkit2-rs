A procedural macro for deriving `VisitMut`.

The resulting implementation will walk your tree by mutable references and call
`VisitorMut::visit_mut()` at set moments.

# Usage
To use this macro, simply slap `#[derive(VisitMut)]` on top of your struct/enum:
```ignore
use ast_toolkit2::loc::Located;
use ast_toolkit2::tree::Node;
use ast_toolkit2::visit::{Visit, VisitMut, VisitorMut};


#[derive(Located, Node, Visit, VisitMut)]
struct Branch {
    leaf1: Leaf,
    leaf2: Leaf,
    leaf3: Leaf,
}

#[derive(Located, Node, Visit, VisitMut)]
struct Leaf;


struct CountLeafsTraversal;
impl Visitor for CountLeafsTraversal {
    type Result = usize;

    fn visit<N: Visit>(&mut self, node: &N) {
        let mut count = 0;
        if node.as_t::<Leaf>().is_some() {
            count += 1;
        } else {
            node.visit(self)
        }
        count
    }
}


assert_eq!(CountLeafsTraversal.visit(Branch { leaf1: Leaf, leaf2: Leaf, leaf3: Leaf }), 3);
```
_Note: Since `VisitMut` is a supertrait of `Visit` and, transitively, `Node` and `Located`, these
need to be implemented too._

## Skipping variants/fields
If you don't want to visit a variant of field of your struct, then use `#[visit(skip)]` to indicate
this:
```ignore
use ast_toolkit2::loc::Located;
use ast_toolkit2::tree::Node;
use ast_toolkit2::visit::{Visit, Visitor};


#[derive(Located, Node, Visit)]
struct Branch {
    leaf1: Leaf,
    #[visit(skip)]
    leaf2: Leaf,
    leaf3: Leaf,
}

#[derive(Located, Node, Visit)]
struct Leaf;


struct CountLeafsTraversal;
impl Visitor for CountLeafsTraversal {
    type Result = usize;

    fn visit<N: Visit>(&mut self, node: &N) -> usize {
        let mut count = 0;
        if node.as_t::<Leaf>().is_some() {
            count += 1;
        } else {
            node.visit(self)
        }
        count
    }
}


assert_eq!(CountLeafsTraversal.visit(Branch { leaf1: Leaf, leaf2: Leaf, leaf3: Leaf }), 2);
```

## Standard implementations
The following [`std`] library types implement `Visit`, to make derivation easier:
- [`Option`];
- [`Vec`].

## Generics
Like most macros, this macro will enforce that `Visit` exists for all of the generic types
mentioned in a type.
