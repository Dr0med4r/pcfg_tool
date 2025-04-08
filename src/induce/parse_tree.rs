use nom::{
    IResult, Parser,
    multi::many1,
    branch::alt,
    bytes::complete::is_not,
    character::complete::{char, space0},
    sequence::delimited,
};

pub enum TreeElement<T> {
    Leaf{name: T},
    Node{name: T, children: Vec<TreeElement<T>>},
}

pub struct ParseTree<T> {
    pub item: T,
    pub children: TreeElement<T>,
}

fn string(input: &str) -> IResult<&str, &str> {
    delimited(space0, is_not(" ()"), space0).parse(input)
}

fn atom(input: &str) -> IResult<&str, TreeElement<&str>> {
    let (input,atom) = delimited(space0, is_not(" ()"), space0).parse(input)?;
    Ok((input, TreeElement::Leaf{name: atom}))
}

fn elements(input: &str) -> IResult<&str, Vec<TreeElement<&str>>> {
    let (input, elements) = many1(element).parse(input)?;
    Ok((input, elements))
}

fn element(input: &str) -> IResult<&str, TreeElement<&str>> {
    let (input, (name, test) ) = delimited(
        char('('),
        (string, alt((elements, atom))),
        char(')'),
    )
    .parse(input)?
    Ok((input, TreeElement::Node {name, children: test}))
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
    fn element_test() {
        assert_eq!(element("(ROOT (hi ho))"), Ok(("", "ROOT (hi ho)")));
        assert_eq!(element("(ROOT (NS (NP hi)))"), Ok(("", "ROOT (NS (NP hi))")));
    }
}
