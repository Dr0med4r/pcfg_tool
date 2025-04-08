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

impl<'a> From<&'a str> for ParseTree<&'a str> {
    fn from(val: &'a str) -> Self {
        ParseTree {
            root: val,
            children: vec![],
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
    many1(delimited(space0, element, space0)).parse(input)
}

fn element(input: &str) -> IResult<&str, ParseTree<&str>> {
    let (input, (name, elements)) = delimited(
        char('('),
        (atom, alt((elements, map(atom, str_to_parsetree_vec)))),
        char(')'),
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

// fn parse_element(input: &str) -> IResult<&str, ParseTree<String>> {
//     let ()
//     Ok((input, ParseTree {}))
// }

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
        assert_eq!(
            element("(ROOT hallo)"),
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
        )
    }
}
