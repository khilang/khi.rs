# dxrs

A recursive Rust parser for the Dx format.

# Dx format

**Dx** is a simple, concise, human-readable and writable textual format for configuration, serialization and hand-coding
of structures and markup. It is a meta-format, like XML, which means that only the syntax is defined, and the semantic
validity of a Dx document is determined by an external source, such as a schema or a program. Dx syntax natively
supports many structures common to programming languages and other formats.

## Example

```
# Example of a Dx document containing article data.

title: Aluminium;
type: chemical-element;
tags: [metal; reactive; common];
key: aluminium;
element-symbol: Al;
atomic-number: 13;
references: {
  wikipedia: "https://en.wikipedia.org/wiki/Aluminium";
  snl: "https://snl.no/aluminium";
};

content: {
  (h1) (title!)   # (title!) is a macro that substitutes in the article title.
  (span class:article-name) {Aluminium} is a
  (@ chemical-element) {chemical element}   # The (@) macro denotes an article link.
  with
  (@ element-symbol) {symbol}
  (span class:chemical-symbol-text) {Al}
  and
  (@ atomic-number) {atomic number}
  (span class:atomic-number) {13}.
  (\\)   # (\\) indicates a line break.
  In pure form, it is a highly reactive metal.
  (\\)
  It constitutes 8.2% of the earth's crust.
};
```

More example documents can be found in the `examples` directory.

## Features

- is human-readable and writable.
- is simple, has few special cases and is easy to parse.
- is concise with minimal syntax noise.
- has comments.
- fully defines syntax, but not semantics. The semantics of a document is fully defined by the user.
- can natively express both structured (such as sequences, dictionaries) and unstructured (such as markup) data.
- support structures common in programming languages and other formats like JSON, XML&HTML and TeX.
- is practical for hand-coding and as a source format.
- can be used for configuration.
- can be used for serialization.

## Syntax

A Dx document consists of expressions and arguments. The root node may either be an open expression, an open sequence or
an open dictionary. Open means that it is not enclosed in brackets.

#### Expression

An *expression* is a sequence of arguments delimited by whitespace. Ex: `arg1 arg2 arg3 ...`.

#### Argument

An *argument* is an element of an expression. There are 6 argument variants.

#### Symbol argument

A *symbol* is a sequence of characters. Symbols are delimited by whitespace. Ex: `Symbol`, `These are four symbols`.
Chevrons/angular brackets `⟨` ,`⟩` (**not** the less than `<` and greater than `>` signs) can be used to insert an
escaped symbol. Any character within the chevrons, even reserved characters, are part of the symbol.
Ex: `⟨This: is a symbol⟩`.

#### Quote argument

A *quote* is a string enclosed in quotes `"`. Ex: `"This is a quote argument"`. Any character within the quotes, even
reserved characters, are part of the quote.

#### Grouping argument

A *grouping* is an expression enclosed in curly brackets `{`, `}`. Ex: `{ expr }`. `{}` is not a grouping, but an empty
dictionary. An empty grouping is inserted as the single standalone character `_`.

#### Sequence argument

A *sequence* is a sequence of expressions delimited by semicolons `;` enclosed in square brackets `[`, `]`.
A trailing semicolon is allowed. Ex: `[expr1; expr2; expr3; ...]`. `[]` is an empty sequence.

#### Dictionary argument

A *dictionary* is a sequence of key-value entries delimited by semicolons `;` enclosed in curly brackets `{`, `}`.
A key and its value is separated by a colon `:`. A key is a string given by either a symbol or a quote argument. A value
is an expression. A trailing semicolon is allowed.
Ex: `{k1: v1; "k2": v2; k3: v3; ...}`.
`{}` is an empty dictionary.

#### Function argument

A *function* is a function expression enclosed in parentheses `(`, `)`. Ex: `( fexpr )`.

#### Function expression

A *function expression* is a sequence of positional arguments, options and flags delimited by whitespace. Positional
arguments are regular arguments. Options are key-value pairs, where the key is a string given as a symbol or a quote,
the value is a single argument and the key and value is separated by a colon `:`. A flag is a string given as a symbol
or a quote terminated by a semicolon `;`.
Ex: `arg1 f1; k1:v1 "f2"; arg2 "k2":v2` is a function expression with 2 positional arguments, 2 options and 2 flags.

#### Escape character

Backslash `\ ` is the *escape character*, and will insert the next character into the current argument
no matter if it is reserved or not.

#### Reserved characters

Brackets `(`, `)`, `[`, `]`, `{`, `}`, `⟨`, `⟩`, quotes `"`, colons `:` and semicolons `;` are *reserved characters*.
They cannot be used in symbols unless they are escaped.

#### Whitespace equivalence

All whitespace is equivalent to a single space character, unless it is escaped. All non-symbol arguments and the
beginnings and ends of expressions are considered to have implicit whitespace surrounding them.
Ex: `symbol{grouping}` is equivalent to `symbol { grouping }`.

#### Comments

A number sign `#` opens a *comment* which extends to the next newline. `#` must be followed by either whitespace `# ...`
or another number sign `##...`, and it must follow whitespace `... #`, otherwise it will be treated as part of a 
symbol. Ex: `# This is a comment` and `#### Part II` start comments,
while `#2`, `#0FA60F` and `elements#` do not.

#### Empty grouping

`{}` indicates an empty dictionary, not an empty grouping. A single standalone underscore `_` is treated as an empty
grouping instead. To insert a standalone underscore as a symbol, it must be escaped: `\_`.

## Semantics

Dx defines the syntax of expressions and arguments, but it does not define their semantics or dictate how structures
are encoded. The semantics must be defined by a user of the format. This is similar to how XML defines the syntax and
requires a schema to define valid tags and values.

Data structures can be encoded in Dx in many arbitrary ways. Thus, a user must define an encoding for each of them. A
user must also define whether the document root is an open expression, an open sequence or an open dictionary. This can
be done by writing documentation, using a schema, or preferably by implementing serialization and deserialization
procedures in a program. Once this is done, one has a format with well-defined syntax and semantics.

Although there are no definite rules about how a structure should be encoded, there are some best practices when it
comes to what expressions and arguments represent. Following these practices when defining a structure encoding will
make Dx documents more uniform, which makes them more easily understood. Below, the best practices for usage of 
expressions and arguments are described.

#### Expression

An expression represents an encoded data structure. A program evaluates an expression to produce the structure.
Ex: The `Text` expression `This is text` is evaluated to the string *"This is text"*. The `Point3D` expression
`40 -10 9` is evaluated to the struct `Point3D {x: 40, y: -10, z: 9}`.

#### Argument

Data structures vary in complexity. Many simple structures correspond to a single argument, such as strings, numbers,
sequences and dictionaries. Some more complex data structures may require several arguments to be properly encoded. In
such cases, one must split the data structure into multiple parts, and encode each part using the argument variant that
best fits.

An argument on its own can also represent a simple encoded data structure. Such an argument can be part of a greater
expression. More complex structures cannot be encoded as a single argument, so if one wishes to make it part of a
greater expression, one should encode such a structure as an expression, and then wrap it as a grouping argument.

#### Symbol argument

A symbol is the most general type of argument. Its meaning is highly dependent on the type of expression. It can
represent

- data such as text or numbers.
- a static part of an expression. Ex: `Matrix` and `,` in `Matrix [1, 0, 0; 0, 1, 0; 0, 0, 1]` or `Binomial` in `Binomial 20 10%`.

#### Quote argument

A quote usually represents text.

#### Grouping argument

- In markup, a grouping should be used to group text. For example, a grouping can delimit the content of an XML tag or
  a TeX macro argument.

- For structured data, a grouping could be used to insert one structure into another. In this way, nested data
  structures are encoded.

- In general, a grouping can be used to delimit an input argument of an expression.

#### Sequence argument

A sequence trivially represents a collection of multiple values, such as arrays, ordered lists, unordered lists and
tuples.

#### Dictionary argument

A dictionary trivially represents a collection of named values or mappings.

#### Function argument

A function represents something that may

- consume nearby arguments.
- affect nearby arguments (such as adding semantics).
- perform a computation.
- perform a substitution.

For example:

- an XML tag should be represented by a function, because the tag affects its content by adding semantics.
- a TeX macro should be represented by a function, because the macro consumes nearby arguments and substitutes them
  and itself for a new computed value.
- a variable should be represented by a function, because it substitutes itself for its value.

Options and flags should be used to add metadata to a function. For example, XML tag attributes should be represented by
function options.

## Suggested encodings of common data structures

It should be obvious how to encode most data structures. Here are some examples and suggestions.

### Text

Text is primarily given as a sequence of symbol arguments `This is text` or as a quote argument `"This is text."`.

### Numbers

A number is primarily given as a symbol. Ex: `400`, `2.45`, `True` or `50%`.

### Sequences and dictionaries

Trivially, a sequence or a dictionary can be encoded as a sequence argument or a dictionary argument.
A sequence could alternatively be encoded as varargs expression. Ex: `1 3 5 7`.

### Structs

Structs could optionally encode type name in the first argument.

| Variant           | Examples                                                                 |
|-------------------|--------------------------------------------------------------------------|
| Named fields      | `Point3D { x: 10; y: 30; z: 5 }` or<br> `{ x: 10; y: 30; z: 5 }`         |
| Positional fields | `Point3D 10 30 5`, `Point3D [10, 30, 5]`,<br> `10 30 5` or `[10, 30, 5]` |

### Enums

Enums encode their variant in the first argument.

| Variant           | Example                             |
|-------------------|-------------------------------------|
| Named fields      | `Binomial { n: 50; p: 10% }`        |
| Positional fields | `Uniform 0 10` or `Uniform [0; 10]` |

### XML-like markup

Here it is assumed that the structure can consist of text and tags with attributes and content.

Symbols and quotes encode text and functions encode tags with attributes. Groupings are used to encode the content of a
tag.

Ex: `(p class:front-paragraph) {Hello world!} Text.` encodes the HTML `<p class="front-paragraph">Hello world!</p>Text.`.

This encoding is expanded upon in the HTML preprocessor subcrate.

### TeX-like markup

Here it is assumed that the structure can consist of text, macro
commands and groupings.

Symbols and quotes encode text, groupings trivially encode groupings, and functions encode macros.

Ex: `This is text. (italic) {This is a grouping}` encodes the TeX `This is text.\italic{This is a grouping}`.

This encoding is expanded upon in the TeX preprocessor subcrate.

## Purpose

The goal is to design a textual format that satisfy the requirements below. It is also considered how other formats that
already exist satisfy these requirements. The most important requirements are **1**, **2**, **7** and **10**, while
**8** and **9** are of lesser importance. The reason for designing this new format is indeed the lack of a format
satisfying requirements **7** and **10**. Keep in mind that some requirements may be subjective.

<table>
 <tr><th>Goal</th><th>JSON</th><th>XML&HTML</th><th>YAML</th><th>TOML</th></tr>
 <tr>
  <td>
   <b>1</b> The format is human-readable. Assuming that best formatting practices are followed, the format should be easy to read
   and understand.
  </td>
  <td>✔️</td>
  <td>✔️</td>
  <td>✔️</td>
  <td>✔️</td>
 </tr>
 <tr>
  <td><b>2</b> The format is human-writable. Here, ease of writing or convenience is not taken into account.</td>
  <td>✔️️</td>
  <td>✔️</td>
  <td>✔️</td>
  <td>✔️</td>
 </tr>
 <tr>
  <td>
   <b>3</b> The format is simple. There are few special cases. An advantage of a simpler format is that it is easier to
   parse.
  </td>
  <td>✔️</td>
  <td>✔️ There is sometimes minor confusion about whether to encode data as tags or as attributes.</td>
  <td>❌ YAML is complex. There are many special cases and values may yield surprising results.</td>
  <td>✔️</td>
 </tr>
 <tr>
  <td><b>4</b> The format is concise and contains minimal syntax noise.</td>
  <td>➖ JSON is concise, but does not minimize syntax noise. It requires quotes around keys even when there is no ambiguity.</td>
  <td>❌ XML does not minimize syntax noise. It is extremely verbose.</td>
  <td>✔️</td>
  <td>✔️</td>
 </tr>
 <tr>
  <td><b>5</b> The format has comments.️</td>
  <td>❌</td>
  <td>✔️</td>
  <td>✔️</td>
  <td>✔️</td>
 </tr>
 <tr>
  <td><b>6</b> The format fully defines syntax, but not semantics. Semantics (such as data types of expressions) are defined externally.</td>
  <td>❌ JSON fully defines syntax and data types.</td>
  <td>✔️</td>
  <td>❌ YAML fully defines syntax and data types.</td>
  <td>❌ TOML fully defines syntax and data types.</td>
 </tr>
 <tr>
  <td><b>7</b> The format can natively express both structured and unstructured data, such as:
    <ul>
      <li>numbers, text, structs, enums, sequences and dictionaries.</li>
      <li>markup consisting of text and tags with attributes and content, like HTML.</li>
      <li>markup consisting of text, groupings and macro commands, like TeX.</li>
    </ul>
  </td>
  <td>❌ JSON does not support markup, and it is not entirely clear how to represent sum types.</td>
  <td>➖️ XML can represent these structures thanks to its flexibility, but it has no native support for sequences and dictionaries. Yet, it is obvious how to model them.</td>
  <td>❌ YAML does not support markup, and it is not entirely clear how to represent sum types.</td>
  <td>❌ TOML does not support markup, and it is not entirely clear how to represent sum types.</td>
 </tr>
 <tr>
  <td><b>8</b> The format is suitable for configuration.</td>
  <td>➖️ JSON can be used for configuration, but it lacks comments, which is a big downside.</td>
  <td>
   ➖️ XML can be used for configuration, but its verbosity makes it inconvenient as a universal configuration format.
  </td>
  <td>✔️ </td>
  <td>✔️ </td>
 </tr>
 <tr>
  <td><b>9</b> The format is suitable for serialization.</td>
  <td>✔️ </td>
  <td>✔️ </td>
  <td>➖️ YAML can be used for serialization, but is not optimal.</td>
  <td>❌ TOML is not intended for serialization.</td>
 </tr>
 <tr>
  <td>
   <b>10</b> The format is suitable for hand-coding. It lends itself well as a source format. It can conveniently encode structured data and markup.
  </td>
  <td>➖️️ JSON can be hand-coded easily, but its lack of comments makes it impractical as a source format.</td>
  <td>❌️ XML is not suitable as a source format because of its verbosity.</td>
  <td>✔️ YAML is easy to hand-code in most cases, but when YAML documents get large or complex, they may get hard to
      manage, especially given the whitespace indentation.</td>
  <td>✔️ </td>
 </tr>
</table>

## Design

Here are some of the decisions made during the design process. The reasons behind these decisions may be subjective.

#### Name

Dx comes from "data expressions". This makes sense since expressions represent encoded data structures.

#### Whitespace delimited

Arguments are primarily delimited by whitespace, because arguments are frequent, and whitespace creates the least
possible amount of syntax noise.

#### Why whitespace equivalent?

Whitespace equivalence gives users the flexibility to format a document however they like. For simple expressions, this
flexibility is not needed, but for complex expressions that span multiple lines, it is appreciated.

#### Why not whitespace indentation?

Whitespace indentation is simple and works great when expressions span one line. In most whitespace-indented formats and
languages, this is the case most of the time. However, when an expression has to span multiple lines, whitespace
indentation requires complex rules that feel like special cases to the user. Keeping track of whitespace and indentation
level also adds complexity to the parser. Thus, it was decided to stick with bracket delimited scopes.

#### Argument variants

Looking at modern programming languages and ubiquitous formats such as JSON, XML&HTML and TeX, the following structures
are commonly used: numbers, text, structs/product types, enums/sum types, dictionaries, sequences, markup with text and
tags with attributes and content (such as XML) and markup with text, groupings and macro commands (such as TeX).

The described argument variants are able to natively support these structures with concise and convenient syntax.

#### User defined semantics

The XML approach is taken, where the semantics (such as the types of the contained expressions) of a document must be
defined externally. A user must define semantics by using a schema, writing documentation or implementing
serialization/deserialization procedures in a program.

This approach is taken because normally a document is not read blindly. A user or a program already has expectations
about the types of the encoded expressions. This also makes the format more flexible and extensible, like XML.
