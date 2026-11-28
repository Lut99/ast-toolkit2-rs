A procedural macro for deriving `Visit`.

The resulting implementation will walk your tree by read-only references and call
`Visitor::visit()` at set moments.

# Usage
To use this macro, simply slap `#[derive(Visit)]` on top of your struct/enum:
```ignore
use ast_toolkit2::loc::Located;
use ast_toolkit2::tree::Node;
use ast_toolkit2::visit::{Visit, Visitor};


#[derive(Located, Node, Visit)]
struct Branch {
    leaf1: Leaf,
    leaf2: Leaf,
    leaf3: Leaf,
}

#[derive(Located, Node, Visit)]
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
_Note: Since `Visit` is a supertrait of `Node` and, transitively, `Located`, these need to be
implemented too._

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

## Generics
Like most macros, this macro will enforce that `Visit` exists for all of the generic types
mentioned in a type.
