#![allow(dead_code)]
#![allow(clippy::needless_return)]

use crate::Operator::{And, Not, Or};

#[derive(Debug, PartialEq)]
enum Token {
    Operator(Operator),
    OpenGroup,
    CloseGroup,
    Label(String),
}

#[derive(Debug, PartialEq)]
enum Operator {
    And,
    Or,
    Not,
}

#[derive(Debug, PartialEq)]
enum Expression {
    Label(String),
    Inverse(Box<Expression>),
    Operation(Box<Expression>, Operator, Box<Expression>),
}

type TokenIterator<'a> = std::iter::Peekable<std::vec::IntoIter<Token>>;

fn tokenize(input: &str) -> Vec<Token> {
    // Add some extra whitespace not required by spec, to ease splitting up
    let expanded = input
        .replace("(", "( ")
        .replace(")", " )")
        .replace("!", "! ");

    expanded
        .split(' ')
        .filter(|word| { word != &""})
        .map(|word| match word {
            "!" => Token::Operator(Not),
            "&&" => Token::Operator(And),
            "||" => Token::Operator(Or),
            "(" => Token::OpenGroup,
            ")" => Token::CloseGroup,
            label => Token::Label(label.to_string()),
        }).collect()
}

fn parse_single(iter: &mut TokenIterator) -> Expression {
    match iter.next().unwrap() {
        Token::Label(label) => Expression::Label(label),
        Token::Operator(Not) => Expression::Inverse(parse_single(iter).into()),
        Token::OpenGroup => { 
            let expr = parse_expression(iter); 
            assert_eq!(iter.next(), Some(Token::CloseGroup)); 
            expr 
        },
        _ => unimplemented!()
    }
}

fn parse_expression(iter: &mut TokenIterator) -> Expression {
    let first = parse_single(iter);

    match iter.peek() {
        None | Some(Token::CloseGroup) => {
            // End of expression
            return first;
        },
        _ => {}
    };

    let operator = match iter.next().unwrap() {
        Token::Operator(oper) => oper,
        _ => panic!("Non-operator following a token")
    };

    return Expression::Operation(first.into(), operator, parse_expression(iter).into())
}

fn parse(tokens: Vec<Token>) -> Expression {
    let mut iter = tokens.into_iter().peekable();
    parse_expression(&mut iter)
}

fn main() {
    //let test_query = "tag:tag1 && (tag:tag2 || tag:tag3 || tag:tag4) && !bad";
    let test_query = "tag:tag1 && (tag:tag2 || (tag:tag3 && !asdf))";
    let tokens = tokenize(test_query);
    let parsed = parse(tokens);
    println!("{:#?}", parsed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        assert_eq!(tokenize(""), Vec::<Token>::new());
        assert_eq!(tokenize("a b c"), vec![
            Token::Label("a".into()),
            Token::Label("b".into()),
            Token::Label("c".into()),
        ]);
        assert_eq!(tokenize("tag:test && hello-world"), vec![
            Token::Label("tag:test".into()),
            Token::Operator(And),
            Token::Label("hello-world".into()),
        ]);
        assert_eq!(tokenize("a && !b"), vec![
            Token::Label("a".into()),
            Token::Operator(And),
            Token::Operator(Not),
            Token::Label("b".into()),
        ]);
        assert_eq!(tokenize("((a b) c)"), vec![
            Token::OpenGroup,
            Token::OpenGroup,
            Token::Label("a".into()),
            Token::Label("b".into()),
            Token::CloseGroup,
            Token::Label("c".into()),
            Token::CloseGroup,
        ]);
    }

    #[test]
    fn test_parse_single() {
        let mut test1 = vec![
            Token::Label("asd".into()),
        ].into_iter().peekable();
        assert_eq!(parse_single(&mut test1), Expression::Label("asd".into()));

        let mut test2 = vec![
            Token::Operator(Not),
            Token::Label("asd".into()),
        ].into_iter().peekable();
        assert_eq!(parse_single(&mut test2), Expression::Inverse(Expression::Label("asd".into()).into()));

        let mut test3 = vec![
            Token::OpenGroup,
            Token::Label("asd".into()),
            Token::CloseGroup,
        ].into_iter().peekable();
        assert_eq!(parse_single(&mut test3), Expression::Label("asd".into()));
    }
}
