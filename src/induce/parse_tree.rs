use std::fmt::Display;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::is_not,
    character::complete::{char, space0},
    combinator::map,
    multi::many1,
    sequence::delimited,
};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTree<T> {
    pub root: T,
    pub children: Vec<ParseTree<T>>,
}

impl<T> ParseTree<T> {
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
    /// recursive executes the function f for itself and all children
    pub fn execute_for_nodes<N>(&self, f: &mut N)
    where
        N: FnMut(&Self),
    {
        f(self);
        for child in &self.children {
            child.execute_for_nodes(f);
        }
    }
}

impl<T: Display> Display for ParseTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseTree { root, children } if children.is_empty() => {
                write!(f, "{}", root)
            }
            ParseTree { root, children } => {
                write!(
                    f,
                    "({} {})",
                    root,
                    children
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            }
        }
    }
}

fn atom(input: &str) -> IResult<&str, &str> {
    delimited(space0, is_not(" ()"), space0).parse(input)
}

fn str_to_parsetree_vec(input: &str) -> Vec<ParseTree<&str>> {
    vec![ParseTree {
        root: input,
        children: vec![],
    }]
}

fn elements(input: &str) -> IResult<&str, Vec<ParseTree<&str>>> {
    many1(element).parse(input)
}

/// returns the remainder of the input and the parsed tree for the input
pub fn element(input: &str) -> IResult<&str, ParseTree<&str>> {
    let (input, (name, elements)) = delimited(
        space0,
        delimited(
            char('('),
            (atom, alt((elements, map(atom, str_to_parsetree_vec)))),
            char(')'),
        ),
        space0,
    )
    .parse(input)?;
    Ok((
        input,
        ParseTree {
            root: name,
            children: elements,
        },
    ))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn atom_test() {
        assert_eq!(atom("hallo"), Ok(("", "hallo")));
        assert_eq!(atom("hallo hi"), Ok(("hi", "hallo")));
        assert_eq!(atom(" hallo"), Ok(("", "hallo")));
        assert_eq!(atom("hallo)"), Ok((")", "hallo")));
        assert!(atom("(hallo").is_err());
        assert_eq!(atom(" \t hallo  "), Ok(("", "hallo")));
        assert_eq!(many1(atom).parse("hallo hi"), Ok(("", vec!["hallo", "hi"])));
    }

    #[test]
    fn elements_test() {
        assert_eq!(
            elements("(ROOT hallo) (ROOT hallo)"),
            Ok((
                "",
                vec![
                    ParseTree {
                        root: "ROOT",
                        children: vec![ParseTree {
                            root: "hallo",
                            children: vec![],
                        }],
                    },
                    ParseTree {
                        root: "ROOT",
                        children: vec![ParseTree {
                            root: "hallo",
                            children: vec![],
                        }],
                    },
                ]
            ))
        );
    }

    #[test]
    fn element_test() {
        assert!(element("(ROOT test test)").is_err());
        assert!(element("(ROOT (test))").is_err());
        assert_eq!(
            element(" ( ROOT  \t  hallo ) "),
            Ok((
                "",
                ParseTree {
                    root: "ROOT",
                    children: vec![ParseTree {
                        root: "hallo",
                        children: vec![],
                    }],
                }
            ))
        );
        assert_eq!(
            element("(ROOT (NS hi))"),
            Ok((
                "",
                ParseTree {
                    root: "ROOT",
                    children: vec![ParseTree {
                        root: "NS",
                        children: vec![ParseTree {
                            root: "hi",
                            children: vec![],
                        }],
                    },],
                }
            ))
        );
        assert_eq!(
            element("(ROOT (NS hi) (NS warum))"),
            Ok((
                "",
                ParseTree {
                    root: "ROOT",
                    children: vec![
                        ParseTree {
                            root: "NS",
                            children: vec![ParseTree {
                                root: "hi",
                                children: vec![],
                            }],
                        },
                        ParseTree {
                            root: "NS",
                            children: vec![ParseTree {
                                root: "warum",
                                children: vec![],
                            }],
                        }
                    ],
                }
            ))
        );
    }
}
