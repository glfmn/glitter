use nom::{error, error::ParseError, Err, IResult};

/// Apply a function to the error returned by a parser
pub fn map_err<I: Clone, E1, E2, M, F, O>(f: F, map_err: M) -> impl Fn(I) -> IResult<I, O, E2>
where
    F: Fn(I) -> IResult<I, O, E1>,
    M: Fn(I, E1) -> E2,
    E1: error::ParseError<I>,
    E2: error::ParseError<I>,
{
    move |i: I| match f(i.clone()) {
        Ok(o) => Ok(o),
        Err(Err::Failure(e)) => Err(Err::Failure(map_err(i, e))),
        Err(Err::Error(e)) => Err(Err::Error(map_err(i, e))),
        Err(Err::Incomplete(i)) => Err(Err::Incomplete(i)),
    }
}

/// Apply a function to the error returned by a parser, coverting errors to failures
pub fn map_fail<I: Clone, E1, E2, M, F, O>(f: F, on_err: M) -> impl Fn(I) -> IResult<I, O, E2>
where
    F: Fn(I) -> IResult<I, O, E1>,
    M: Fn(I, E1) -> E2,
    E1: ParseError<I>,
    E2: ParseError<I>,
{
    move |i: I| match map_err(&f, &on_err)(i) {
        Err(Err::Error(e)) => Err(Err::Failure(e)),
        rest => rest,
    }
}

///
pub fn delimited_many0<I, O1, O2, O3, E: error::ParseError<I>, F, G, H>(
    left: F,
    item: G,
    right: H,
) -> impl Fn(I) -> IResult<I, Vec<O2>, E>
where
    I: Clone + PartialEq + nom::InputLength,
    F: Fn(I) -> IResult<I, O1, E>,
    G: Fn(I) -> IResult<I, O2, E>,
    H: Fn(I) -> IResult<I, O3, E>,
{
    use error::ErrorKind::Many0;

    move |input: I| {
        let (input, _) = left(input.clone())?;

        let mut i = input.clone();
        let mut items = Vec::new();
        loop {
            match item(i.clone()) {
                Err(Err::Error(e)) => match right(i.clone()) {
                    Ok((i, _)) => return Ok((i, items)),
                    Err(Err::Error(e2)) => {
                        if i.input_len() == 0 {
                            return Err(Err::Failure(e2));
                        } else {
                            return Err(Err::Failure(E::add_context(input, "expression", e)));
                        }
                    }
                    Err(e) => return Err(e),
                },
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    if i == i1 {
                        return Err(Err::Error(E::from_error_kind(i, Many0)));
                    }

                    i = i1;
                    items.push(o);
                }
            }
        }
    }
}
