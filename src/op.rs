//! Implements preprocessing according to the operator convention.

use std::rc::Rc;
use crate::{Element, Text, TextType, Value};
use crate::parse::pdm::{ParsedCatenation, ParsedTag, ParsedTaggedTuple, ParsedText, ParsedValue, Position};

pub enum TakeBefore {
    None, Character, Word
}

pub enum TakeAfter {
    None, Character, WordAndBracket
}

pub struct Operator<'b> {
    pub symbol: &'b str,
    pub command: &'b str,
    pub take_before: TakeBefore,
    pub take_after: TakeAfter,
}

/// Preprocess the operators in a catenation.
pub fn preprocess_operators(value: &ParsedValue, operators: &[Operator]) -> Result<ParsedValue, String> {
    let mut result: Vec<Element<ParsedValue>> = vec![];
    let mut iter = value.iter_as_catenation();
    // If this value is a singular bracketed text, a nested catenation is being processed.
    let bracketed_text = if let ParsedValue::Text(ParsedText { text_type, .. }, ..) = value {
        *text_type == TextType::Bracket
    } else {
        false
    };
    while let Some(element) = iter.next() {
        if let Element::Element(element) = element {
            match element {
                ParsedValue::Text(text, _, _) => {
                    if !bracketed_text && text.text_type != TextType::Plain {
                        result.push(Element::Element(element.clone()));
                        continue;
                    }
                    let mut textbuf: Vec<char> = vec![];
                    let mut t = text.as_str();
                    'charloop: loop {
                        if t.is_empty() {
                            break;
                        }
                        for operator in operators {
                            let symbol = operator.symbol;
                            if t.starts_with(symbol) {
                                let first = match operator.take_before {
                                    TakeBefore::None => {
                                        if !textbuf.is_empty() {
                                            let text = textbuf.iter().collect::<String>();
                                            let text = ParsedValue::Text(ParsedText { str: Rc::from(text.as_str()), text_type: TextType::Plain }, element.from(), element.to());
                                            result.push(Element::Element(text));
                                            textbuf.clear();
                                        }
                                        None
                                    }
                                    TakeBefore::Character => {
                                        if !textbuf.is_empty() {
                                            if *textbuf.last().unwrap() == ' ' {
                                                return Err(format!("Operator {} at {}:{} is missing an argument before.", operator.symbol, element.from().line, element.from().column)); // Todo position
                                            } else {
                                                let c = textbuf.pop().unwrap();
                                                let text = String::from(c);
                                                let t = ParsedValue::Text(ParsedText { str: Rc::from(text.as_str()), text_type: TextType::Plain }, element.from(), element.to());
                                                if !textbuf.is_empty() {
                                                    let text = textbuf.iter().collect::<String>();
                                                    let text = ParsedValue::Text(ParsedText { str: Rc::from(text.as_str()), text_type: TextType::Plain }, element.from(), element.to());
                                                    result.push(Element::Element(text));
                                                    textbuf.clear();
                                                }
                                                Some(t)
                                            }
                                        } else {
                                            match result.pop() {
                                                Some(Element::Element(element)) => Some(element),
                                                _ => return Err(format!("Operator {} at {}:{} is missing a character argument after.", operator.symbol, element.from().line, element.from().column)),// Todo position
                                            }
                                        }
                                    }
                                    TakeBefore::Word => {
                                        if !textbuf.is_empty() {
                                            if *textbuf.last().unwrap() == ' ' {
                                                return Err(format!("Operator {} at {}:{} is missing a word argument before.", operator.symbol, element.from().line, element.from().column)); // Todo position
                                            } else {
                                                let mut text = String::new();
                                                loop {
                                                    if textbuf.is_empty() || *textbuf.last().unwrap() == ' ' {
                                                        break;
                                                    }
                                                    let c = textbuf.pop().unwrap();
                                                    text.insert(0, c);
                                                }
                                                let t = ParsedValue::Text(ParsedText { str: Rc::from(text.as_str()), text_type: TextType::Plain }, element.from(), element.to());
                                                if !textbuf.is_empty() {
                                                    let text = textbuf.iter().collect::<String>();
                                                    let text = ParsedValue::Text(ParsedText { str: Rc::from(text.as_str()), text_type: TextType::Plain }, element.from(), element.to());
                                                    result.push(Element::Element(text));
                                                    textbuf.clear();
                                                }
                                                Some(t)
                                            }
                                        } else {
                                            match result.pop() {
                                                Some(Element::Element(element)) => Some(element),
                                                _ => return Err(format!("Operator {} at {}:{} is missing a word argument before.", operator.symbol, element.from().line, element.from().column)),// Todo position
                                            }
                                        }
                                    }
                                };
                                t = t.strip_prefix(symbol).unwrap();
                                let last = match operator.take_after {
                                    TakeAfter::None => None,
                                    TakeAfter::Character => {
                                        if !t.is_empty() {
                                            let c = t.chars().next().unwrap();
                                            t = t.strip_prefix(c).unwrap();
                                            if c != ' ' {
                                                let cstr = String::from(c);
                                                Some(ParsedValue::Text(ParsedText { text_type: TextType::Plain, str: Rc::from(cstr.as_str()) }, element.from(), element.to()))
                                            } else {
                                                return Err(format!("Operator {} at {}:{} is missing a character argument after.", operator.symbol, element.from().line, element.from().column)); // Todo position
                                            }
                                        } else {
                                            let next = iter.next();
                                            if let Some(Element::Element(next)) = next {
                                                Some(next.clone())
                                            } else {
                                                return Err(format!("Operator {} at {}:{} is missing a character argument after.", operator.symbol, element.from().line, element.from().column)); // Todo position
                                            }
                                        }
                                    }
                                    TakeAfter::WordAndBracket => {
                                        if !t.is_empty() {
                                            // Read the next word.
                                            let mut w = String::new();
                                            let mut witer = t.chars();
                                            loop {
                                                if let Some(c) = witer.next() {
                                                    if c != ' ' {
                                                        w.push(c);
                                                    } else {
                                                        break;
                                                    }
                                                } else {
                                                    break;
                                                }
                                            }
                                            // Consume the read word.
                                            t = t.strip_prefix(w.as_str()).unwrap();
                                            let e1 = ParsedValue::Text(ParsedText { text_type: TextType::Plain, str: Rc::from(w.as_str()) }, element.from(), element.to());
                                            if t.is_empty() { // Might have bracket.
                                                match iter.next() {
                                                    Some(Element::Element(next)) if !next.is_nil() => {
                                                        let e2 = next.clone();
                                                        let from = e1.from(); let to = e2.to();
                                                        let cat = ParsedValue::Catenation(ParsedCatenation { components: vec![Element::Element(e1), Element::Element(e2)] }, from, to);
                                                        Some(cat)
                                                    }
                                                    _ => {
                                                        Some(e1)
                                                    }
                                                }
                                            } else { // Ended on whitespace.
                                                Some(e1)
                                            }
                                        } else {
                                            let next = iter.next();
                                            if let Some(Element::Element(next)) = next {
                                                Some(next.clone())
                                            } else {
                                                return Err(format!("Operator {} at {}:{} is missing an argument after.", operator.symbol, element.from().line, element.from().column)); // Todo position
                                            }
                                        }
                                    }
                                };
                                let mut elements = vec![];
                                if let Some(first) = first {
                                    elements.push(first);
                                }
                                if let Some(last) = last {
                                    elements.push(last);
                                }
                                result.push(Element::Element(
                                    ParsedValue::TaggedTuple(ParsedTaggedTuple {
                                        tag: Some(ParsedTag {name: Rc::from(operator.command), attributes: vec![]}),
                                        elements,
                                    }, value.from(), value.to()) // TODO Fix positions
                                ));
                                continue 'charloop;
                            }
                        }
                        // If none of the ops matched:
                        let first = t.chars().next().unwrap();
                        textbuf.push(first); // Todo: whitespace to separator
                        t = t.strip_prefix(first).unwrap();
                    }
                    if !textbuf.is_empty() {
                        let text = textbuf.iter().collect::<String>();
                        result.push(Element::Element(ParsedValue::Text(ParsedText {
                            str: Rc::from(text.as_str()),
                            text_type: TextType::Plain
                        }, element.from(), element.to() ))); // Todo: fix positions
                        textbuf.clear();
                    }
                }
                v => {
                    result.push(Element::Element(v.clone()));
                }
            }
        } else {
            unreachable!() // TODO Remove
        }
    }
    if result.len() == 0 {
        Ok(ParsedValue::Nil(value.from(), value.to()))
    } else if result.len() == 1 {
        if let Element::Element(e) = &result[0] {
            Ok(e.clone())
        } else {
            unreachable!()
        }
    } else {
        Ok(ParsedValue::Catenation(ParsedCatenation { components: result }, value.from(), value.to()))
    }
}
