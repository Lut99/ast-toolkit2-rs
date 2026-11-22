A procedural macro for automatically deriving the `Node`-, and `NodeMut`-traits.

The former can be used to iterate over a node's children by read-only reference. Because we
also sometimes want mutable access, `NodeMut` is derived in addition.

# Usage
There is three flavours to deriving your type, configurable using attributes.

## Nonterminals
Nonterminals are branches in your tree. Derive them using the `#[node(nonterm)]`-attribute.

## Literals
Literals are leaves in your tree that do not parse a static piece of text, but rather a value.
Derive them using the `#[node(lit = "...")]`-attribute, where `...` is a regular expression
defining the syntax of values parsed.

## Terminals
Terminals are leaves in your tree that parse static pieces of text. Derive them using the
`#[node(term = "...")]` attribute, where `...` is the text to parse.

# Examples
All of them together:
```ignore
use ast_toolkit2::tree::Node;

#[derive(Node)]
#[node(nonterm)]
struct Sentence(LitInt, Dot);

#[derive(Node)]
#[node(lit = "^[0-9]+")]
struct LitInt;

#[derive(Node)]
#[node(term = ".")]
struct Dot;


fn assert_node<T: Node>() {}

assert_node::<Sentence>();
assert_node::<LitInt>();
assert_node::<Dot>();
```

## Non-mutable implementation
Sometimes, your type cannot iterate over its children mutably but it _can_ iterate immutably.
For example, consider [`std::rc::Rc`].
If that's the case, avoid deriving `NodeMut` by using the `#[node(nomut)]`-attribute:
```ignore
use std::rc::Rc;

use ast_toolkit2::tree::{Node, NodeMut};

#[derive(Node)]
struct Mutable(Base);

#[derive(Node)]
#[node(nomut)]
struct Immutable(Rc<Base>);

#[derive(Node)]
struct Base;


fn assert_node<T: Node>() {}
fn assert_node_mut<T: NodeMut>() {}

assert_node::<Mutable>();
assert_node_mut::<Mutable>();
assert_node::<Immutable>();
// assert_node_mut::<Immutable>();
assert_node::<Base>();
assert_node_mut::<Base>();
```

# A note on generics
All generic types of your struct will require `Node` to have `Node` implemented, and `NodeMut`
to have `NodeMut` implemented.
