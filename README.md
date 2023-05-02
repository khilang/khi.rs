# udl-rs

A Rust parser for **UDL** (Universal Data Language).

# Universal Data Language (UDL)

**UDL** is a textual [metaformat](https://www.w3.org/standards/webarch/metaformats) primarily purposed to defining data
formats that are read and hand-coded by users. Such formats are mainly configuration and markup formats, or a mix
thereof.

**UDL** natively supports the universal data structures found in other programming languages and formats, such as
**XML**, **JSON** and **LaTeX**. It can express both structured data (dictionaries, sequences, hierarchies, values) and
unstructured data (text, markup), and it can express complex structures composed of arbitrary combinations of such data.

**UDL** is a textual format focused on being human-readable and writable. A well formatted **UDL**-document is easy to
read, understand and edit. The format is concise and has minimal syntax noise. Few characters are needed to structure a
document. It is practical and convenient for hand-coding and thus as a source format. Therefore, the format is suitable
as a basis for configuration and markup formats.

**UDL** is simple: there are few special cases and exceptions and there are few reserved characters. This makes it
easy to reason about, generate and parse. At the expense of readability, it can be compactified. Although not designed
for these purposes, it is viable for serialization, data storage and data interchange, though here other formats may be
more optimal.

## Comparisons

Compared to **XML**, **UDL** has native support for the universal data structures sequences and dictionaries. **UDL**'s
tag notation syntax is based on **XML**'s tag syntax, but with some modifications. In **UDL**, there is also support
for encoding commands and actions.

Compared to **JSON**, **UDL** has lesser syntax noise; it does not require quotes around strings. **UDL** has native
support for markup, and importantly, comments. **UDL**'s syntax for sequences and dictionaries is inspired by **JSON**.

Compared to (regular) **LaTeX**, **UDL** has support for structured data. They are similar in terms of syntax noise and
conciseness. It may be argued that command notation in **UDL** is more readable than command notation in **LaTeX**,
since it can be seen clearly from syntax which arguments a command applies to. Additionally, commands can take
structured data as arguments, which is convenient for certain applications.

## Examples & showcase

Here are some examples of **UDL**-based formats and documents written in them. It is demonstrated how structured and
unstructured data can coexist and form more complex structures, and how it can be used for markup and configuration.

### Wiki article example

This is an example of a wiki article written in a **UDL**-based wiki article format.

This example exhibits complex hierarchical structures consisting of both structured data (values, dictionaries and
sequences) and unstructured data (markup).

The purpose of this example is to show the capabilities of **UDL** when it is used to its full extent. In particular, a
wiki article usually contains both structured data and unstructured data. Thus, this is a good example of how **UDL**
can compose both types into more complex hierarchical structures.

Additionally, this example showcases the **UDL** syntax. The readability, conciseness and simplicity of the format
should be compared to other formats encoding the same data.

Notes:
- Macro application looks like this: `<macro>:arg1:arg2:...:argN`. Arguments are appended with a colon.
- The `@` macro inserts a link. It takes two arguments: the first argument is the article to link to, and the second is
  the link label that will appear in the article.
- The `title` macro takes no arguments and is substituted for the article title.

```
title: Aluminium;
shortdesc: The <@>:element:{chemical element} aluminium.;
uuid: 0c5aacfe-d828-43c7-a530-12a802af1df4;
type: chemical-element;
tags: [metal; common];
key: aluminium;

chemical-symbol: Al;
atomic-number: 13;
stp-phase: solid;
melting-point: 933.47;
boiling-point: 2743;
density: 2.7;
electron-shells: [2; 8; 3];

# External references
ext-refs: {
  wikipedia: "https://en.wikipedia.org/wiki/Aluminium";
  snl: "https://snl.no/aluminium";
};

# Intra-wiki references
refs: {
  element: 740097ea-10fa-4203-b086-58632f099167;
  chemsym: 6e2f634c-f180-407a-b9ce-2138b412b248;
  atomnum: 1a5e1974-a78c-4820-afeb-79bef6974814;
  react: ab7d8a1f-c028-4466-9bb2-41a39d153241;
  aloxide: c1ff08e7-a88f-42d5-83c3-6adc4835a07b;
  stab: b3b13474-4fe3-4556-9568-925c066916a5;
  purity: 40786551-85c4-461c-ba6e-4d54d5863820;
  ion: effd5c7a-da31-4357-a94c-91343e9a05eb;
  metal: 84333088-cfcc-4e78-8d3f-7307dcab144b;
};

content: {

  <@>:self:<title> is a <@>:element:{chemical element} with
  <@>:chemsym:{chemical symbol} <chemsym> and <@>:atomnum:{atomic number}
  <atomnum>.

  <p>

  In <@>:purity:pure form, it is a highly <@>:react:reactive <@>:metal:{metal},
  but normally a thin coat of <@>:aloxide:{aluminium oxide} forms on its
  surface, keeping it highly <@>:stab:{stable}.

  <p>

  In nature, it occurs as the <@>:ion:ion <$>:{Al^{3+}}. It constitutes 8.2%
  of the earth's crust, making it the most common <@>:metal:metal found there.

  ...

};
```

### HTML preprocessor example

This is an example of a document written in a **UDL**-based **HTML** preprocessor input format. The preprocessor can
compile this document to **HTML**.

The purpose of this example is to exhibit a **UDL**-based encoding of markup and **XML**-like structures.

Compare this document to the corresponding **HTML** document. In terms of verbosity, the **UDL** document does not
require closing tags. In terms of syntax noise, the **UDL** document does not require quotes around attribute values.

Notes:
- In this format, tags and macros are distinguished with the `@` symbol. Macros start with `@` while regular tags only
  consist of letters.
- Tags can take zero or one argument. A tag with zero arguments is a self-closing tag, and a tag with an argument uses
  the argument as its inner content.
- The `@doctype` macro substitutes for `<!doctype html>`.

```
<@doctype>
<html:> # <tag:> is an opening tag and </tag> or </> is a closing tag.
  <head:>
    <title:><@title></>
    <script: src:script.js></>
  </head>
  <body:>
    <h1: id:main-heading><@title></>
    <p:>Hello world!</> # These two paragraph tags are equivalent.
    <p>:{Hello world!}
    <img src:frontpage.jpg>
    <div: class:dark-background><p:>
      This is a paragraph <br>
      with a line break.
      <em: class:italic-text>This text is italic.</>
    </></>
  </body>
</html>
```

### TeX preprocessor example

This is an example of a document written in a **UDL**-based **LaTeX** preprocessor input format. The preprocessor can
compile this document to **LaTeX**.

The purpose of this example is to exhibit a **UDL**-based encoding of **LaTeX**-like markup.

Compare this document to the corresponding **LaTeX** document. They are similar, but one benefit of the **UDL** document
is that the arguments applied to a command can be determined from syntax alone.

As an application, this encoding could possibly have a use-case in the wiki article example. Articles may contain
mathematical notation, and this encoding could be used to encode **LaTeX**-math, that is later displayed by **MathJax**.

Notes:
- Preprocessor macros start with `@` and regular commands consist only of letters.
- The `@tabulate-sq` automatically tabulates a square grid, such as a matrix. It takes a number and a sequence of
  the tabulated values.

```
<documentclass>:article

<usepackage>:amsmath

<begin>:document

<section>:Equations

  # Define a sum-range command.
  <newcommand>:<SumRn>:*:4:{
    <sum>_{#1}^{#2 <dots> #3} #4
  }

  <begin>:math
    <SumRn>:k:0:100:k
    = 0 + 1 + 2 + <dots> + 99 + 100
    = (0 + 100) + (1 + 99) + <dots> (49 + 51) + 50
    = 5050
  <end>:math

  <begin>:math
    <SumRn>:k:0:n:k
    = 0 + 1 + 2 + <dots> + (n - 1) + n
    = n <cfrac>:n:2 + <cfrac>:n:2
    = <cfrac>:n^2:2 + <cfrac>:n:2
    = n <cdot> <cfrac>:{n + 1}:2
  <end>:math

<section>:Matrices

  <begin>:math
    <mathbf>:X = <begin>:bmatrix <@tabulate-sq>:3:[
      1;0;0;
      0;1;0;
      0;0;1;
    ] <end>:bmatrix
  <end>:math

<end>:document
```

### Material configuration example

This is an example of a **UDL**-based configuration.

The purpose of this example is to showcase a **UDL**-based configuration file and to compare it to the corresponding
**JSON** configuration file.

In terms of syntax noise, the corresponding **JSON** document requires quotes around all keys, quotes around all text
values, does not allow comments, and requires the root level element to be wrapped in brackets. Evidently, **UDL** has
lesser syntax noise. Both formats have a minimal amount of verbosity, and both formats are simple.

```yaml
oak-planks: {
  name: Oak planks;
  description: Planks made from oak wood.;
  tags: [wood];
  price: 200;
};
birch-planks: {
  name: Birch planks;
  description: Planks made from birch wood.;
  tags: [wood];
  price: 200;
};
stone: {
  name: Stone;
  description: A solid material, but does not insulate well.;
  price: 100;
  tags: [heavy; stone];
};
marble: {
  name: Marble;
  price: 450;
  beauty: 2;
  tags: [heavy; stone; wealth];
};
# This material is not available yet.
glass: {
  disabled;
  name: Glass;
  price: 400;
};
```

## Syntax

A **UDL** document consists of expressions, which consist of arguments. Some arguments may in turn contain nested
expressions themselves.

### Expressions and arguments

An *expression* is a sequence of arguments.

**Example:** `arg1 arg2 arg3 ...`.

An *argument* is an element of an expression. There are 6 argument variants: *empty*, *text*, *sequence*, *dictionary*,
*directive* and *compound*.

### Grouping

Brackets `{` `}` are used to group and delimit arguments.

**Example:** `{Text 1} {Text 2}` is an expression with 2 text arguments. Brackets are used to delimit the text
arguments, to prevent them from merging into one text argument.

By grouping arguments, an arbitrary number of them can be given as a single argument. An empty grouping represents an
empty argument. A grouping of one argument simply represents the argument itself. A grouping of multiple (2 or more)
arguments represents a compound argument.

**Example:** `{ arg }` is a grouping of a single argument. This could be useful for delimiting text or delimiting
directive arguments. As arguments, `arg` is equal to `{ arg }`, which is equal to `{ { arg } }`. Indeed, enclosing a
single argument in brackets has no structural effect, but it could improve readability in some cases.

**Example:** `{ arg1 arg2 arg3 }` is a grouping of 3 arguments, which yields a compound argument with 3 arguments.

### Empty argument

An *empty* argument is represented by an empty expression enclosed in brackets: `{}`.

### Text argument

A *text* argument is simply a sequence of words or quoted text.

**Example:** `This is a text argument`.

**Example:** `"Text argument 1" Text argument 2 {Text argument 3} {Text argument 4} Text argument 5` is an
expression consisting of 5 text arguments.

**Example:** `"Quotes allow insertion of arbitrary whitespace and reserved characters, such as : or ]"`.

Unquoted text cannot contain reserved characters, unless they are escaped with backslash `\ `.

**Example:** `Some reserved characters\: \:, \;, \<, \}, etc.`.

Colons `:` can be inserted into unquoted text by repetition.

**Example:** `Some text:: More text` parses to the text `Some text: More text`.

Furthermore, any whitespace in unquoted text is reduced to a single space character. **UDL** is a whitespace-equivalent
format, where all whitespace is equal to a space character, unless it is escaped or within a quote.

### Dictionary argument

A *dictionary* argument is a sequence of key-value entries delimited by semicolons `;` enclosed in curly brackets
`{` `}`. The key and value in an entry is separated by a colon `:`. A key is given by a word or a quote; it cannot be
given as multiple words. A value is an expression.

**Example:** `{ k1: v1; "key 2": v2; k3: v3; ... }`.

An empty dictionary argument must contain a colon to distinguish it from an empty expression.

**Example:** `{:}` is an empty dictionary.

A key followed by a semicolon `;` indicates that its value is an empty expression.

**Example:** `{k1; k2: v2; k3;}` contains the keys `k1` and `k3` which are followed by semicolons `;`. This means that
their values are empty expressions.

A trailing semicolon is allowed.

**Example:** `{k1: v1; k2: v2;}` and `{k1: v1; k2: v2}` are equal.

### Sequence argument

A *sequence* argument is a sequence of expressions delimited by semicolons `;` enclosed in square brackets `[` `]`.

**Example:** `[expr1; expr2; expr3; ...]`.

**Example:** `[]` is an empty sequence.

A trailing semicolon is allowed.

**Example:** `[expr1; expr2;]` and `[expr1; expr2]` are equal.

### Directive argument

A *directive expression* is a directive applied to a number of arguments. There are two notations that produce directive
expressions: command notation and tag notation.

#### Command notation

In command notation, a *directive expression* is encoded as a *directive* enclosed in angular brackets, followed by
arguments applied to it which are appended with colons `:` where there is no surrounding whitespace.

**Example:** `<dir>:arg1:arg2:...:argN` is a directive expression with N arguments.

**Example:** `<dir>` is a directive with no arguments.

**Example:** `<text-weight>:600:{This is bold text}` is the directive `text-weight` applied to 2 text arguments.

The directive, which is the part enclosed in angular brackets, consists of a label followed by attributes. The label is
given by a word or a quote. Following the label, it is possible to insert attributes. An attribute is a
key-value pair. The key and value is delimited by a colon `:`.

**Example:** `<p id:opening class:fancy>` encodes the directive `p` with attributes `id:opening` and `class:fancy`.

An attribute key not followed by a colon is allowed. The value of such an attribute is considered to be an empty
argument.

**Example:** `<input type:checkbox checked>` has the label `input`. It has two attributes: `type` with value `checkbox`
and `checked` with value `{}`.

Directives can be inserted as arguments in a directive expression. They are interpreted as directive expressions that
have zero arguments.

**Example:** In `<cmd0>:arg1:arg2:<cmd3>:arg4:arg5`, `<cmd3>` is a directive expression with zero arguments. `<cmd0>`
is a directive expression with 5 arguments.

The *precedence operator* `<>` is a special operator that can be used in directive expressions. It applies the directive
expression on the right-hand side as an argument to the directive expression on the left-hand side.

**Example:** `<bold>:<>:<italic>:text` is equivalent to `<bold>:{ <italic>:text }`.

#### Tag notation

In tag notation, tags are used to produce directive expressions with one argument. Tag notation is intended to be used
for markup. Tags are used as opening and closing brackets.

Tag notation encodes a directive with a single argument.

**Example:** `<tag:> arg </tag>`.

### Compound argument

A *compound* argument is simply an expression containing multiple (2 or more) arguments enclosed in curly brackets
`{` `}`.

**Example:** `{ {Text} Some more text [1; 2; 3] {k1: v1; k2: v2} {} }` is a compound argument that consists of 2 text
arguments, 1 sequence, 1 dictionary and finally 1 empty argument.

### Root node

The root node of a **UDL** document is either an expression, a sequence or a dictionary. The root node is not an
argument, thus is not enclosed in brackets.

### Reserved characters

The brackets `<`, `>`, `[`, `]`, `{`, `}`, quotes `"`, colons `:` and semicolons `;` are *reserved characters*. They
cannot be used in text unless they are escaped.

### Escape character

Backslash `\ ` is the *escape character*. The character following it is inserted as text no matter if it is reserved or
not.

**Example:** `\[` parses to the text `[`.

### Special escape sequences

Colons `:` are sometimes used in regular text, therefore it could be inconvenient that they are reserved. Therefore,
some special escape sequences are allowed: `::` inserts a colon as text, instead of being parsed as a reserved
character.

**Example:** `Price:: 300€` parses to the text `Price: 300€`.

### Whitespace equivalence and significance

Every sequence of whitespace is equivalent to a single space character, unless the whitespace is escaped or within a
quote. Whitespace between arguments in an expression is significant, but whitespace at the beginning or the end of an
expression is insignificant.

**Example:** `arg1 {arg2}` is not equal to `arg1{arg2}`, because there is a difference in significant whitespace.

**Example:** `arg1{ arg2 }` is equal to `arg1{arg2}`, because there is no difference in significant whitespace.

### Comments

A number sign `#` at the beginning of a word may open a comment, depending on which character follows it. If it is
followed by whitespace or another `#`, then a comment opens that ends at the next newline. Otherwise, if it is followed
by a text glyph, the word is parsed as text as normal.

**Example:** `# This is a comment` is a comment, because `#` is followed by whitespace.

**Example:** `#### Configuration ####` is a comment since `#` is followed by `#`.

**Example:** `#2`, `#0FA60F` and `#elements` are not comments since `#` is followed by a text glyph.

**Example:** A comment is not opened in `This is text# Is this a comment?` since `#` is not at the beginning of a word.

## Semantics

**UDL** dictates the syntax of expressions and arguments, but it does not dictate their semantics or how data structures
are encoded. The semantics, such as the validity of directive expressions, dictionary keys and expression composition,
are determined when a **UDL**-based format is defined. This is similar to how **XML** and **JSON** are metalanguages. On
their own, they only determine if a document is syntactically well-formed, but leave questions of validity to a format
implementer.

A set of data structures can be encoded in **UDL** in many arbitrary ways. Thus, an implementer must define a specific
encoding for each of them. An implementer must also define whether the document root is an expression, a sequence or
dictionary. This can be done by writing documentation, using a schema, or preferably by implementing deserialization
procedures in a program. Once this is done, one has a format with well-defined syntax and semantics.

Although there are no definite rules regarding how a data structure should be encoded, there are some best practices
when it comes to what expressions and variants represent. Following these practices while implementing an encoding
makes **UDL**-based formats more uniform, which makes them more easily understood. Below, the best practices regarding
encodings of expressions and arguments are described.

### Expression and argument semantics

An expression encodes a data structure in a canonical or default way. Arguments of an expression provide the information
required by the expression. Arguments themselves may recursively consist of inner expressions.

This simply means that data structures vary in complexity, and complex structures are made up from simpler structures.
For complex structures, one must split the data structure into multiple parts, and encode each part using the variant
that best fits. One must also decide which variant best composes the arguments.

### Variant semantics

This is a summary of what kind of structures the 6 structural variants encodes.

| Variant                   | Use                                                     |
|---------------------------|---------------------------------------------------------|
| Text                      | Encodes a primitive value.                              |
| Expression/Compound/Empty | Encodes a data structure in a canonical or default way. |
| Sequence                  | Encodes multiple data structures.                       |
| Dictionary                | Encodes multiple named data structures.                 |
| Directive                 | Encodes a data structure in a specific way.             |

### Directive semantics

As an encoding of a data structure, a directive expression encodes a structure in a specific way. However, there are
two dimensions to a directive:

- First, a directive describes how input encodes a data structure.
- Secondly, a directive describes how input encodes an action (An action is a side effect or stateful change or query to
  the environment). A directive is pure if it does not depend on the environment, and impure otherwise.

Given this, a directive expression represents either an encoded data structure, an encoded action, or a mix of both. A
directive could be seen as a generalization of **XML**-tags, **LaTeX**-commands/macros and text placeholders/tokens.

**Example:** In **XML**-like markup, tags are used to mark up and add semantics to text. Tags do not encode any action.
In **UDL**, tags can be encoded as directives, which when applied to text, encodes semantic text. For example, **HTML**
`<span class="italic">text</span>` corresponds to **UDL** `<span: class:italic>text</>`.

**Example:** In `<sender> sent <amount> to <recipient>.`, directives are used to represent tokens/placeholders.

**Example:** In **LaTeX**-like markup, commands/macros are used to perform substitutions, computations and stateful
actions (for example incrementing a section count, or including a package). In **UDL**, commands can be trivially
encoded as directives. For example, **LaTeX** `\frac{2a}{b}` corresponds to **UDL** `<frac>:2a:b`.

**Example:** `<set>:x:100` encodes an action which sets the variable `x` to `100`. It encodes an empty data structure,
since this is purely a command.

There are two notations for directive expressions: command notation and tag notation. Command notation is the default.
Tag notation is purely purposed to markup. It produces a directive expression with one argument which encodes semantic
text.

**Example:** `<bold:>Bold<br>text</>` is a conventional use of tag notation, since `Bold<br>text` is markup and the
expression returns semantic text.

**Example:** `<var:>x</>` is an unconventional use of tag notation, since `x` is not markup, `var` encodes an action
retrieving a variable and the expression does not return semantic text.

### Primitive encoding

Primitive values are trivially encoded as text.

- Strings are encoded as text.
- Numbers, including booleans, are encoded as text. Valid encodings are further determined by number type.

### Markup encoding

Markup is encoded as an expression containing an arbitrary number of text and directive expression arguments.
Directives that produce semantic text and which do not represent any action could be encoded as tags, but this is not
required. Other types of macros, which may represent actions, is encoded in command notation.

**Example:** The preprocessor examples above demonstrate markup encoding.

### Struct / product type encoding

Structs that have no fields are encoded as an empty expression. Structs that have fields are either encoded as a
dictionary or a sequence, depending on if they are named or positional.

| Variant           | Example                  |
|-------------------|--------------------------|
| Named fields      | `{ x: 10; y: 30; z: 5 }` |
| Positional fields | `[10; 30; 5]`            |
| No fields         | `{}`                     |

### Enum / sum type encoding

Enums are encoded as 1 or 2 arguments. The first argument is a text argument that specifies the enum variant. If the
enum has no fields, it does not have a second argument. Otherwise, the second argument is either a sequence or a
dictionary, depending on if the enum has named or positional fields.

| Variant           | Example                      |
|-------------------|------------------------------|
| Named fields      | `Binomial { n: 50; p: 10% }` |
| Positional fields | `Uniform [0; 10]`            |
| No fields         | `StandardNormal`             |

## Purpose

The goal is to design a textual format that satisfy the requirements below. It is also considered how other formats that
already exist satisfy these requirements. The most important requirements are **1**, **2**, **6**, **7**, **8** and
**10**, while **9** is of lesser importance. The primary reason for designing this new format is indeed the lack of a
format satisfying requirements **6** and **10**. Keep in mind that some requirements may be subjective.

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
  <td><b>6</b> The format can natively express both structured and unstructured data, such as:
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
  <td><b>7</b> The format is suitable for markup.</td>
  <td>❌</td>
  <td>✔️ </td>
  <td>❌</td>
  <td>❌</td>
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
  <td><b>9</b> The format is viable for serialization, data storage and data interchange.</td>
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

#### Whitespace equivalence

Whitespace equivalence gives users the flexibility to format a document however they like. For simple expressions, this
flexibility is not needed, but for complex expressions that span multiple lines, it is appreciated.

#### Whitespace indentation

Whitespace indentation is simple and works great when expressions span one line. In many whitespace-indented formats and
languages, this is the case most of the time. However, when an expression has to span multiple lines, whitespace
indentation requires complex rules that feel like special cases to the user. Keeping track of whitespace and indentation
level also adds complexity to the parser. Thus, it was decided to stick with bracket delimited scopes.

#### Argument variants

Looking at modern programming languages and ubiquitous formats such as **JSON**, **XML** and **LaTeX**, the following
structures are universally used: numbers, text, structs/product types, enums/sum types, dictionaries, sequences and
markup consisting of text and commands/tags.

The implemented argument variants are able to natively support these structures with concise and convenient syntax.

#### Metaformat

The XML approach is taken, where the semantics (such as the types of contained expressions) of a document must be
defined externally. A user must define semantics by using a schema, writing documentation or implementing
serialization/deserialization procedures in a program.

This approach is taken simply because it gives format implementers a lot of flexibility. Furthermore, normally a
document is not read blindly. A user or a program already has expectations about the types of encoded expressions. Thus,
it is not necessary to add syntax typed expression either.
