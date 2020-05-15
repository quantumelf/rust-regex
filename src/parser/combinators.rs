use super::{Parser, ParserError, ParseElement};

pub fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B> 
    where
        P: Parser<'a, A>,
        F: Fn(A) -> B,
{
    move |input: &'a str|
        parser.parse(input).map( |(a, remain)| (map_fn(a), remain) )
}

pub fn pair<'a, P1, P2, R1, R2>(p1: P1, p2: P2) -> impl Parser<'a, (R1, R2)>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
{
    move |input|
        p1.parse(input).and_then( |(res1, remain1)|
            p2.parse(remain1).map( |(res2, remain2)|
                ((res1, res2), remain2)))
}

pub fn left<'a, P1, P2, R1, R2>(p1: P1, p2: P2) -> impl Parser<'a, R1>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
{
    map(pair(p1, p2), |(r1, _r2)| r1)
}

pub fn right<'a, P1, P2, R1, R2>(p1: P1, p2: P2) -> impl Parser<'a, R2>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
{
    map(pair(p1, p2), |(_r1, r2)| r2)
}

//TODO: make assertions about start and stop for valid range
pub fn repeat_range<'a, P, R>(p: P, start: usize, stop: Option<usize>) -> impl Parser<'a, Vec<R>>
    where
        P: Parser<'a, R>,
{
    move |mut input| {
        let mut result = Vec::new();

        while let Ok((res, rem)) = p.parse(input).and_then(|res| if stop.map_or(true, |stp| result.len() < stp) {Ok(res)} else {Err(ParserError::RepeatUpper(stop.expect("should never be none")))}) {
            result.push(res);
            input = rem;
        }

        if start <= result.len() {Ok((result, input))} else {Err(ParserError::RepeatLower(start))}
    }
}

pub fn zero_or_more<'a, P, R>(p: P) -> impl Parser<'a, Vec<R>>
    where
        P: Parser<'a, R>,
{
    repeat_range(p, 0, None)
}

pub fn one_or_more<'a, P, R>(p: P) -> impl Parser<'a, Vec<R>>
    where
        P: Parser<'a, R>,
{
    repeat_range(p, 1, None)
}

pub fn predicate<'a, P, R, F>(parser: P, pred: F) -> impl Parser<'a, R>
    where
        P: Parser<'a, R>,
        F: Fn(&R) -> bool,
{
    move |input|
        parser.parse(input).and_then( |(res, rem)| if pred(&res) {Ok((res, rem))} else {Err(ParserError::Error(input.to_owned()))})
}

pub fn and_then<'a, P, A, NextP, B, F>(p: P, f: F) -> impl Parser<'a, B>
    where
        P: Parser<'a, A>,
        NextP: Parser<'a, B>,
        F: Fn(A) -> NextP,
{
    move |input|
        p.parse(input).and_then(|(a, rem)| f(a).parse(rem))
}

pub fn either<'a, P1, P2, R>(p1: P1, p2: P2) -> impl Parser<'a, R>
    where
        P1: Parser<'a, R>,
        P2: Parser<'a, R>,
{
    move |input| p1.parse(input).or(p2.parse(input))
}

pub fn not<'a, P, R>(p: P) -> impl Parser<'a, ()>
    where P: Parser<'a, R>,
{
    move |input|
        if p.parse(input).is_err() {Ok(( (), input ))} else {Err(ParserError::UnexpectedMatch(input.to_owned()))}
}

pub fn not_but<'a, P1, R1, P2, R2>(p1: P1, p2: P2) -> impl Parser<'a, R2>
    where
        P1: Parser<'a, R1>,
        P2: Parser<'a, R2>,
{
    right(not(p1), p2)
}

//TODO
pub fn quantified<'a, P>(p: P) -> impl Parser<'a, ParseElement>
    where
        P: Parser<'a, ParseElement>
{
    p
}