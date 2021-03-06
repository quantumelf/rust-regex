#[macro_use]
mod macros;

pub mod parsers;
pub mod combinators;

#[derive(Debug, PartialEq)]
pub enum ParserError {
    Error(String),
    RepeatUpper(usize),
    RepeatLower(usize),
    UnexpectedMatch(String),
}

type ParserResult<'a, Output> = Result<(Output, &'a str), ParserError>;

pub trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParserResult<'a, Output>;

    fn map<NewOutput, F>(self, f: F) -> BoxedParser<'a, NewOutput>
        where
            Self: Sized + 'a,
            Output: 'a,
            NewOutput: 'a,
            F: Fn(Output) -> NewOutput + 'a,
    {
        BoxedParser::new(combinators::map(self, f))
    }

    fn predicate<F>(self, f: F) -> BoxedParser<'a, Output>
        where
            Self: Sized + 'a,
            Output: 'a,
            F: Fn(&Output) -> bool + 'a,
    {
        BoxedParser::new(combinators::predicate(self, f))
    }
}

impl<'a, T, U> Parser<'a, T> for U where U: Fn(&'a str) -> ParserResult<'a, T> {
    fn parse(&self, input: &'a str) -> ParserResult<'a, T> {
        self(input)
    }
}

pub struct BoxedParser<'a, T> {
    inner: Box<dyn Parser<'a, T> + 'a>
}

impl<'a, T> BoxedParser<'a, T> {
    fn new<P>(p: P) -> Self
        where
            P: Parser<'a, T> + 'a,
    {
        BoxedParser {
            inner: Box::new(p),
        }
    }
}

impl<'a, T> Parser<'a, T> for BoxedParser<'a, T> {
    fn parse(&self, input: &'a str) -> ParserResult<'a, T> {
        self.inner.parse(input)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Expression {
    alt: Alternation,
}

impl Expression {
    fn new(choices: Vec<SubExpression>) -> Self {
        Expression{ alt: Alternation::new(choices) }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Alternation {
    choices: Vec<SubExpression>
}

impl Alternation {
    fn new(choices: Vec<SubExpression>) -> Self {
        Alternation{ choices: choices }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SubExpression {
    tokens: Vec<QuantifiedToken>
}

impl SubExpression {
    fn new(tokens: Vec<QuantifiedToken>) -> Self {
        SubExpression{ tokens: tokens }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum QuantifiedToken {
    Singleton(Token),
    Quantified(Token, Quantifier)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Char(char),
    IndirectMatch(IndirectMatch),
    Group(Group),
    Anchor(Anchor),
    Unimpl,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Anchor {
    Begin,
    End,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Group {
    Capturing(usize, Box<Expression>),
    NonCapturing(Box<Expression>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Quantifier {
    Lazy(RawQuantifier),
    Greedy(RawQuantifier),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RawQuantifier {
    Kleene,
    Plus,
    Possible,
    Exact(usize),
    Range(usize, Option<usize>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum IndirectMatch {
    WildCard,
    Class(CharClass),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CharClass {
    Regular(RawCharClass),
    Inverted(RawCharClass),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RawCharClass {
    CharGroup(Vec<char>),
    CharRange(char, char),
    SpecialSet(CharSet),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CharSet {
    Word,
    WhiteSpace,
    DecimalDigit,
    UnicodeCategory(UnicodeCategory),
}

#[derive(Debug, PartialEq, Eq)]
pub enum UnicodeCategory {
    Punctuation,
}

//TODO: CONSIDER UNRESERVING '-', flip order of char class parsers
fn not_reserved(c: &char) -> bool {
    let reserved = ['[', ']', '(', ')', '*', '?', '+', '.', '\\', '-'];

    reserved.iter().find(|x| *x == c).is_none()
}

//TODO
fn escapable(c: &char) -> bool {
    !not_reserved(c)
}

fn map_literal_charset(c: char) -> Option<CharSet> {
    match c {
        'w' => Some(CharSet::Word),
        's' => Some(CharSet::WhiteSpace),
        'd' => Some(CharSet::DecimalDigit),
         _  => None,
    }
}

fn map_unicode_charset(s: &str) -> Option<UnicodeCategory> {
    match s {
        "P"  => Some(UnicodeCategory::Punctuation),
        "Lt" => Some(UnicodeCategory::Punctuation),
        "Ll" => Some(UnicodeCategory::Punctuation),
        "N"  => Some(UnicodeCategory::Punctuation),
        "S"  => Some(UnicodeCategory::Punctuation),
         _   => None,
    }
}

fn map_anchor(c: char) -> Option<Anchor> {
    match c {
        '^' => Some(Anchor::Begin),
        '$' => Some(Anchor::End),
         _  => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::parsers::*;
    use super::combinators::*;

    #[test]
    fn test_match_literal() {
        let parse_hello = match_literal("Hello");
        assert_eq!(Ok(( (), " World")), parse_hello.parse("Hello World"));
        assert_eq!(Err(ParserError::Error("HeLlO WoRlD".to_owned())), parse_hello.parse("HeLlO WoRlD"));
        assert_eq!(Ok(( (), "")), parse_hello.parse("Hello"));
    }

    #[test]
    fn test_any_char() {
        assert_eq!(Ok(( 'H', "ello" )), any_char.parse("Hello"));
        assert_eq!(Err(ParserError::Error("".to_owned())), any_char.parse(""));
    }

    #[test]
    fn test_map() {
        let any_char_lc = map(any_char, |c| c.to_ascii_lowercase());
        assert_eq!(Ok(( 'h', "ello" )), any_char_lc.parse("Hello"));
        assert_eq!(Err(ParserError::Error("".to_owned())), any_char_lc.parse(""));

        let parse_hello_to_bool = map(match_literal("Hello"), |_unit| true);
        assert_eq!(Ok(( true, "" )), parse_hello_to_bool.parse("Hello"));
        assert_eq!(Err(ParserError::Error("Hola".to_owned())), parse_hello_to_bool.parse("Hola"));
    }

    #[test]
    fn test_pair() {
        let first_char_then_ello = pair(any_char, match_literal("ello"));
        assert_eq!( Ok(( ('H', ()), "")), first_char_then_ello.parse("Hello") );
    }

    #[test]
    fn test_left() {
        let first_char_then_ello_left = left(any_char, match_literal("ello"));
        assert_eq!( Ok(('H', "")), first_char_then_ello_left.parse("Hello") );
    }

    #[test]
    fn test_right() {
        let first_char_then_ello_right = right(any_char, match_literal("ello"));
        assert_eq!( Ok(((), "")), first_char_then_ello_right.parse("Hello") );
    }

    #[test]
    fn test_repeat_range() {
        let match_e = || match_literal("e");
        let s1 = "eeeeeeeeefffffff";
        let s2 = "f";

        let all_parsed_res = Ok((vec![(); 9], "fffffff"));
        let parsed_5_res = Ok((vec![(); 5], "eeeefffffff"));
        let failed_parse = |n| Err(ParserError::RepeatLower(n));

        assert_eq!(all_parsed_res, repeat_range(match_e(), 0, None).parse(s1));
        assert_eq!(all_parsed_res, repeat_range(match_e(), 1, None).parse(s1));
        assert_eq!(all_parsed_res, repeat_range(match_e(), 5, None).parse(s1));
        assert_eq!(all_parsed_res, repeat_range(match_e(), 9, None).parse(s1));
        assert_eq!(all_parsed_res, repeat_range(match_e(), 0, Some(20)).parse(s1));

        assert_eq!(parsed_5_res, repeat_range(match_e(), 0, Some(5)).parse(s1));
        assert_eq!(parsed_5_res, repeat_range(match_e(), 1, Some(5)).parse(s1));
        assert_eq!(parsed_5_res, repeat_range(match_e(), 2, Some(5)).parse(s1));
        assert_eq!(parsed_5_res, repeat_range(match_e(), 3, Some(5)).parse(s1));

        assert_eq!(failed_parse(10), repeat_range(match_e(), 10, None).parse(s1));
        assert_eq!(failed_parse(25), repeat_range(match_e(), 25, None).parse(s1));

    }
    #[test]
    fn test_predicate() {
        let white_space_remover = predicate(any_char, |c| c.is_whitespace());
        
        let leading_space = " hello";
        let no_leading_space = &leading_space[1..];

        assert_eq!(Ok((' ', "hello")), white_space_remover.parse(leading_space));
        assert_eq!(Err(ParserError::Error("hello".to_owned())), white_space_remover.parse(no_leading_space))
    }

    #[test]
    fn test_optional() {
        let maybe_j = optional(match_literal("J"));

        assert_eq!(Ok((Some(()), "")), maybe_j.parse("J"));
        assert_eq!(Ok((None, "K")), maybe_j.parse("K"));
    }

    #[test]
    fn test_quantifier() {
        let quant_parser = parsers::quantifiers::quantifier();

        assert_eq!(Ok((Quantifier::Greedy(RawQuantifier::Kleene), "")), quant_parser.parse("*"));
    }

    #[test]
    fn test_parser() {
        let t_parser = parsers::expression();
        println!("BUILT!");
        assert_eq!(
            Ok((
                Expression{
                    alt: Alternation{
                        choices: vec![
                            SubExpression{ 
                                tokens: vec![
                                    QuantifiedToken::Singleton(Token::Char('J')),
                                    QuantifiedToken::Singleton(Token::Char('T'))
                                ]
                            }
                        ]
                    }
                },

                ""
            )),

            t_parser.parse("JT")
        )
    }

    #[test]
    fn test_parser_print() {
        let par = parsers::expression();
        let expr = par.parse("a*b(?:[a-e])((i?)[\\w][^a-e])");
        println!("{:#?}", expr);
    }

    #[test]
    fn test_number() {
        let t_parser = parsers::number();
        println!("BUILT!");
        assert_eq!(
            Ok((10000,
                ""
            )),

            t_parser.parse("10000")
        )
    }

    #[test]
    fn test_char_token() {
        let t_parser = parsers::token();
        println!("BUILT!");
        assert_eq!(
            Ok((
                Token::Char('J'),
                ""
            )),

            t_parser.parse("J")
        )
    }

    #[test]
    fn test_pq_char_token() {
        let t_parser = parsers::pqtoken();
        println!("BUILT!");
        assert_eq!(
            Ok((
                QuantifiedToken::Singleton(Token::Char('J')),
                ""
            )),

            t_parser.parse("JT")
        )
    }

    #[test]
    fn test_subexpression() {
        let t_parser = parsers::sub_expression();
        println!("BUILT!");
        assert_eq!(
            Ok((
                SubExpression{
                    tokens: vec![QuantifiedToken::Singleton(Token::Char('J')), QuantifiedToken::Singleton(Token::Char('J'))],
                },
                ""
            )),

            t_parser.parse("JT")
        )
    }
}

pub struct UnimplementedParser<'a, T> {
    phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Parser<'a, T> for UnimplementedParser<'a, T> {
    fn parse(&self, _input: &'a str) -> ParserResult<'a, T> {
        unimplemented!()
    }
}