# KhiTeX preprocessor

**Khi**-based **LaTeX** preprocessor.

## Macro

| Macro    | Example                               | Function                                                         |
|----------|---------------------------------------|------------------------------------------------------------------|
| `<$>`    | `<$>:x`                               | Inline math                                                      |
| `<n>`    | `<n>`                                 | Insert newline command (`\\`)                                    |
| `<p>`    | `<p>`                                 | Insert paragraph separator (two newlines or equivalently `\par`) |
| `<def!>` | `<def!>:<Log>:1:{<operatorname>:Log}` | Define a LaTeX command                                           |
| `<raw!>` | `<raw!>:"text"`                       | Insert exact string (no escaping)                                |

## Optional argument

An optional argument is an argument enclosed in a pair of square brackets `[`, `]`.
To indicate an optional argument, end the command with an apostrophe. For example,
`<sqrt'>:3:27` produces `\sqrt[3]{27}`.

## Tabulation

Tabulation is performed upon reading a table. For example, `[1|0|0; 0|2|0]` produces
`1&0&0\\0&2&0\\`.

## Whitespace after command

After a command with no arguments, TeX consumes all whitespace. Thus, there is no
difference between `\dots {a}` and `\dots{a}`. To rectify this, `{}` is inserted after
an empty command preceding whitespace. For example, `<dots> a` produces `\dots{} a`
and `<dots>a` produces `\dots a`.

## Example

```
# Math equations

<documentclass>:article

<usepackage>:amsmath

<setlength>:<jot>:1em # Controls math line spacing

<begin>:document

A document containing some
equations and matrices.

<section>:Equations

  # Define a sum-range command.
  <def!>:<SumRn>:4:{ # def! is a macro that defines a LaTeX command.
    <sum>_{#1}^{[`#2:`#3]`} #4
  }

  <def!>:<Log>:0: <operatorname>:Log

  <begin>:equation* <begin>:split

    <sqrt>:5 <times> <sqrt>:5 = 5
    <n>
    # A trailing apostrophe indicates that the first argument is optional.
    <sqrt'>:3:4 <times> <sqrt'>:3:16 = 4

  <end>:split <end>:equation*

  <begin>:equation* <Log>(1 + 2 + 3) = <Log>:1 + <Log>:2 + <Log>:3 <end>:equation*

  <begin>:align* [
    | <SumRn>:k:0:100:k | = 0 + 1 + 2 + <dots> + 99 + 100                  |
    |                 ~ | = (0 + 100) + (1 + 99) + <dots> + (49 + 51) + 50 |
    |                 ~ | = 5050                                           |
  ] <end>:align*

  <begin>:align* [
    | <SumRn>:k:0:n:k                    |
    | = 0 + 1 + 2 + <dots> + (n - 1) + n |
    | = n <cfrac>:n:2 + <cfrac>:n:2      |
    | = <cfrac>:n^2:2 + <cfrac>:n:2      |
    | = n <cdot> <cfrac>:{n + 1}:2       |
  ] <end>:align*

<section>:Matrices

  <begin>:math
    <mathbf>:X = <begin>:bmatrix [
      |1|0|0|
      |0|1|0|
      |0|0|1|
    ] <end>:bmatrix
    =
    <begin>:bmatrix [1|~|~; ~|1|~; ~|~|1] <end>:bmatrix
  <end>:math

<end>:document
```
