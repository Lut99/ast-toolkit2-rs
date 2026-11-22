A procedural macro for deriving the `Parsable` trait.

This will derive parsers for your types. If you are writing an AST that closely follows the syntax
users can write, then this trait will give you parsers almost for free.

# Usage
There are three variations of parsers to write:
- Parsers for nonterminals are the most derive-y. The macro will generate a parser that simply
  tries all of the fields in your type one-by-one, in-order. If it's a struct, **all** of them need
  to succeed and the result is placed in the struct; if it's an enum, **any** of them needs to
  succeed and the result is placed in the succeeding variant.
- Parsers for literals are implemented by giving a regex representing the complex expression. The
  procedural macro relies on nibble's `Convert`-trait to turn the result into the value you are
  attempting to parse.
- Parsers for terminal are implemented by parsing fixed sequences of input. This can be subdivided
  into e.g. keywords (which may be e.g. identifiers, secretly, instead of keywords, and as such
  require termination conditions) and punctuation (they do not require termination conditions;
  seeing them is always enough).

Below, each of these three is elaborated on.

## Nonterminals
Nonterminals are derived in a much more usual fashion.

Simply define your type and then derive `Parsable` on it:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
struct CommaDot {
  comma: Comma,
  dot: Dot,
}

#[derive(Parsable)]
#[term = ","]
struct Comma(Loc);

#[derive(Parsable)]
#[term = "."]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b",.");
assert_eq!(CommaDot::parse(input1), Ok((
    CommaDot {
        comma: Comma(Loc::encapsulate_range(0, ..1)),
        dot: Dot(Loc::encapsulate_range(0, 1..2)),
    },
    input1.slice(2..),
)));
```
The above is used for parsing a fixed number of things in sequence; but you can also have the
parser branch by defining an enum:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
enum CommaOrDot {
  Comma(Comma),
  Dot(Dot),
}

#[derive(Parsable)]
#[term = ","]
struct Comma(Loc);

#[derive(Parsable)]
#[term = "."]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b",");
let input2 = Slice::with_raw_id(0, b".");
assert_eq!(CommaOrDot::parse(input1), Ok((
    CommaOrDot::Comma(Comma(Loc::encapsulate_range(0, ..1))),
    input1.slice(1..),
)));
assert_eq!(CommaOrDot::parse(input2), Ok((
    CommaOrDot::Dot(Dot(Loc::encapsulate_range(0, ..1))),
    input2.slice(1..),
)));
```

### Skipping variants/fields
If your type carries additional information that needn't be parsed, you can use the
`#[parse(skip)]`-attribute on a field or variant.

On fields, the field is not parsed but instead constructed by its
[`Default`](std::default::Default) implementation. For example:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
struct NamedDot {
  #[parse(skip)]
  name: String,
  dot: Dot,
}

#[derive(Parsable)]
#[term = "."]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b".");
assert_eq!(NamedDot::parse(input1), Ok((
    NamedDot {
        name: String::new(),
        dot: Dot(Loc::encapsulate_range(0, ..1)),
    },
    input1.slice(1..),
)));
```

For variants, using `skip` means that it can simply never be constructed via parsing:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
enum CommaOrDot {
  Comma(Comma),
  #[parse(skip)]
  Dot(Dot),
}

#[derive(Parsable)]
#[term = ","]
struct Comma(Loc);

#[derive(Parsable)]
#[term = "."]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b",");
let input2 = Slice::with_raw_id(0, b".");
assert_eq!(CommaOrDot::parse(input1), Ok((
    CommaOrDot::Comma(Comma(Loc::encapsulate_range(0, ..1))),
    input1.slice(1..),
)));
assert_eq!(CommaOrDot::parse(input2), Err(NibbleError::Unmatched(",", input2)));
```

### Parsing as another type
Sometimes a field makes an exception to the rule of the type it has. In those cases, it may make
sense to first parse as another type and then convert to the field's type.

To do so, the `Parsable` macro supports `#[parse(as = ...)]` and `#[parse(try_as = ...)]` on fields
(and on types; see [below](#Type-shadowing)) to first parse as type `...`, and then use that type's
[`From`] and [`TryFrom`](std::convert::TryFrom) implementation, respectively, to do the conversion.

For example:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
struct CommaDot {
  comma: Comma,
  #[parse(as = Comma)]
  dot: Dot,
}

#[derive(Parsable)]
#[term = ","]
struct Comma(Loc);

#[derive(Parsable)]
#[term = "."]
struct Dot(Loc);
impl From<Comma> for Dot {
    #[inline]
    fn from(value: Comma) -> Self {
        Self(value.0)
    }
}

let input1 = Slice::with_raw_id(0, b",,");
assert_eq!(CommaDot::parse(input1), Ok((
    CommaDot {
        comma: Comma(Loc::encapsulate_range(0, ..1)),
        dot: Dot(Loc::encapsulate_range(0, 1..2)),
    },
    input1.slice(2..),
)));
```

### Auxillary type generation
In order to properly implement parsers for complex types, additional types are needed to generate -
in particular, a custom error type and custom formatter are necessary for `Parsable`.

These are, by default, placed in a module that has the same name as your struct but in
`snake_case`, and are then called `snake_case::Error` and `snake_case::Formatter`, respectively. If
you want to change the name of the generate module, use `#[parse(mod = ...)]` instead.

For enum variants, a nested module _within_ the main module is placed for its additionally
generated error type and formatter. The naming scheme of this module is the same, but now it's
`snake_case` for the variant name; and, likewise, it can be changed by adding `#[parse(mod = ...)]`
on the variant.
> Note: when generating complex enum variants, there is even a new type generated that implements
  `Parsable` for the whole variant, which is then later put in the actual variant.

Note, however, that the macro attempts to avoid generating auxillary types when it can to keep the
name space- and set of generated tokens as small as possible.

## Literals
To parse a literal, first accept that your type must have a fixed structure: it must either be in
struct-syntax with precisely one field name `value`, and one field name `loc`; or it must be in
tuple-syntax with precisely two fields, the first with the type of the value and the second with
a type `Loc`.

Then, consider the type your literal's value is stored as. To use it with the automatic derivation,
make sure a `Convert`-implementation exists for it.

Now you can use `#[parse(lit = "...")]`, with `...` a
[regular expression](https://docs.rs/regex/latest/regex/#syntax), to define how your type is
parsed:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
#[parse(lit = "^[0-9]+")]
struct Int(u64, Loc);

let input1 = Slice::with_raw_id(0, b"123");
assert_eq!(Int::parse(input1), Ok((Dot(123, Loc::encapsulate_range(0, 0..3)), input1.slice(3..))));
```

> Tip: Don't forget to use the start-of-haystack anchor `^`, or else the regular expression does
> not match the start of the input.

## Terminals
Parsing terminals is relatively easy. Simply give `#[parse(term = "...")]`, where `...` is the
sequence you want to be able to parse, when deriving for your struct:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
#[parse(term = ".")]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b".");
assert_eq!(Dot::parse(input1), Ok((Dot(Loc::encapsulate_range(0, 0..1)), input1.slice(1..))));
```
As you see, the part that is parsed (encoded by `Loc`) is placed in the struct's only field and can
later be used to refer back to this specific dot in the input stream. Note that, like with
[literals](#Literals), the structure of your type is quite fixed if you want to derive for it.

### Seperators
To implement keywords which may conflict with identifiers or other keywords, use the
`#[parse(sep = "...")]`-attribute, where `...` is some
[regular expression](https://docs.rs/regex/latest/regex/#syntax) describing what must **not** come
after the keyword:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
#[parse(term = "foo", sep = "^[a-z]")]
struct Foo(Loc);

let input1 = Slice::with_raw_id(0, b"foo");
let input2 = Slice::with_raw_id(0, b"foob");
assert_eq!(Foo::parse(input1), Ok((Foo(Loc::encapsulate_range(0, 0..3)), input1.slice(3..))));
assert_eq!(Foo::parse(input2), Err(NibbleError::Unmatched("foo", input1.slice(3..))));
```
The attribute above encodes: "match `foo` UNLESS it is followed by any lowercase alphabetic
character."

## General
### `std` types
Note that `Parsable` is implemented for some [`std`]-types to make derivation easier:
- [`Vec<T>`](Vec): Greedily parses zero or more occurrences of `T`, but always matches.
- [`Option<T>`](Option): Greedily parses zero or one occurrences of `T`, but always matches.

### `expects`-strings
By default, the implementation will generate customized formatters for your type. For example, for
structs it will generate a formatter listing all nested type's formatters in-order; but for
terminals, it is simply only the string describing what is expected.

If you want to deviate from this behaviour (and we recommend that you do for larger constructs like
expressions), give a static string to format instead with the
`#[parse(expected = "...")]`-attribute:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
#[parse(expected = "a period", term = ".")]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b"foo");
assert_eq!(Foo::parse(input1), Err(NibbleError::Unmatched("a period", input1)));
```

### Popping whitespace
In order to pop whitespace from your parser, strategically add `#[parse(whitespace = "...")]` to
your type:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
#[parse(term = ".", whitespace = "^[ ]+")]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b"    .");
assert_eq!(Dot::parse(input1), Ok((Dot(Loc::encapsulate_range(0, 4..5)), input1.slice(5..))));
```
Typically, doing this for your terminals and literals is sufficient to make your whole tree robust
to whitespace. But sometimes, you will need to manually intervene in a parser's implementation to
ensure this is the case.

Note that `#[parse(whitespace)]` is shorthand for `#[parse(whitespace = "^\s+")]`, which matches
all unicode whitespace characters (unless the `regex-unicode` feature is not given, then it
evaluates to all ascii whitespace characters). Technically it is also a bit faster, since the regex
only needs to be compiled once for all types using it instead of compiling the same regex everytime
a type uses it for the first time.

### Type shadowing
Sometimes it is much more convenient to parse your type as another type first and then convert the
one to the other. We already covered doing this [anonymously on fields](#Parsing-as-another-type),
but it is also possible to do it for whole types.

This can be done by using the `#[parse(as = ...)]` and `#[parse(try_as = ...)]` attributes. If
given, then the implementation will first parse type `...` and then use
[`From`](std::convert::From)- and [`TryFrom`](std::convert::TryFrom), respectively, to make the
conversion.

For example:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::{Parsable, Slice};

#[derive(Parsable)]
#[parse(as = Dot)]
struct Foo(Loc);
impl From<Dot> for Foo {
    #[inline]
    fn from(value: Dot) -> Self { Self(value.0) }
}

#[derive(Parsable)]
#[parse(term = ".")]
struct Dot(Loc);

let input1 = Slice::with_raw_id(0, b".");
assert_eq!(Foo::parse(input1), Ok((Foo(Loc::encapsulate_range(0, ..1)), input1.slice(1..))));
```

### Interaction with `Node`
Note that `#[parse(lit = "...")]` and `#[parse(term = "...")]` are actually also read from the
`node`-attribute, requiring you to only give it once:
```ignore
use ast_toolkit2::loc::Loc;
use ast_toolkit2::nibble::Parsable;
use ast_toolkit2::tree::Node;

#[derive(Node, Parsable)]
#[node(lit = ".")]
struct Dot(Loc);

// Works as expected!
```

### Interaction with `Loc`
Note that, any field that specifically requires `Loc`, does not literally need to be `Loc`; any
type `T` for which `Loc` has [`From<T>`](std::convert::From) implemented, is accepted.

### Generics
Like most macros, this macro will enforce that `Parsable` exists for all of the generic types
mentioned in a type.
