use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{char, u32};
use nom::combinator::map;
use nom::IResult;
use nom::sequence::{delimited, separated_pair, terminated, tuple};
use chrono::{NaiveDate, NaiveTime};
use nom::number::complete::float;
use crate::data::{DrawType, EntryError, PlaceEntry};

pub(crate) fn parse_entry(input: &str) -> IResult<&str, PlaceEntry> {
    let (i, (timestamp, uid_hash, color, location)) = tuple((
        map(
            tuple((
                terminated(u32, char('-')),
                terminated(u32, char('-')),
                terminated(u32, char(' ')),
                terminated(u32, char(':')),
                terminated(u32, char(':')),
                terminated(float, tag(" UTC,")),
            )),
            |x| NaiveDate::from_ymd(x.0 as i32, x.1, x.2).and_time(
                NaiveTime::from_hms_milli(x.3, x.4, x.5 as u32, ((x.5 % 1.0) * 1000.0) as u32)),
        ),
        terminated(map(take_until(","), |x: &str| x.to_string()), char(',')),
        terminated(map(take_until(","), |x: &str| x.to_string()), char(',')),
        delimited(char('"'), parse_draw_type, char('"')),
    ))(input)?;
    IResult::Ok((i, PlaceEntry { timestamp, uid_hash, color, location }))
}

fn parse_draw_type(input: &str) -> IResult<&str, DrawType> {
    alt((
        map(
            tuple((u32, char(','), u32, char(','), u32, char(','), u32)),
            |x| DrawType::Rect(x.0, x.2, x.4, x.6),
        ),
        map(
            separated_pair(u32, char(','), u32),
            |x| DrawType::Pixel(x.0, x.1),
        ),
    ))(input)
}
