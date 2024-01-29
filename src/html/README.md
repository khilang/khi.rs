# KhiML preprocessor

**Khi**-based **XML/HTML** preprocessor.

## Macro

| Macro        | Example           | Function                          |
|--------------|-------------------|-----------------------------------|
| `<doctype!>` | `<doctype!>:html` | Insert doctype declaration        |
| `<raw!>`     | `<raw!>:"text"`   | Insert exact string (no escaping) |

## Object

A tag sequence is produces upon reading a dictionary. For example, `{a: X; b: Y; c: Z}`
produces `<a>X<a/><b>Y</b><c>Z</c>`.

## HTML example

```
# A frontpage

<doctype!>:html # Macro that inserts <!doctype html>.
<html>:{
  <head>:{
    <title>:{Hello world!}
    <script src:script.js>:{~} # {~} denotes an empty element.
  }
  <body>:{
    <h1 id:main-heading>:{Hello world!}
    <p>:{Hello world!}
    <img src:frontpage.jpg>
    <div class:dark-background>:<>:<p>:{
      This is a paragraph <br>
      with a line break.
      <em class:italic>:{This text is italic.}
    }
    <pre>:<>:<code>:<>:<raw!>:<#>
      def fib(n):
          if n == 0:
              return 0
          elif n == 1:
              return 1
          else:
              return fib(n - 1) + fib(n - 2)
    <#>
  }
}
```

## XML example

```
# A list of fruits

<fruits>:{
  <fruit>:{
    <name>:Apple
    <amount>:30
    <colour>:#ce4531
  }
  <fruit>:{
    <name>:Watermelon
    <amount>:10
    <colour>:#aff10e
  }
  # Dictionaries compile to a sequence of tags.
  <fruit>:{
    > name: Mango
    > amount: 20
    > colour: #f0b902
  }
}
```
