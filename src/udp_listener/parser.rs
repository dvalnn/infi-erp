use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{digit1, multispace0, multispace1},
    multi::fold_many1,
    sequence::{delimited, pair, preceded, terminated},
    IResult, Parser,
};

use crate::db_api::{ClientOrder, WorkPiece};

fn parse_money(input: &str) -> IResult<&str, Option<i64>> {
    let (input, _) = tag("€")(input)?;
    let (input, units_str) = terminated(digit1, tag(",").or(tag(".")))(input)?;

    let cents = input.parse::<i64>().ok();
    let units = units_str.parse::<i64>().ok();

    if let (Some(cents), Some(units)) = (cents, units) {
        Ok(("", Some(units * 100 + cents)))
    } else {
        Ok((input, None))
    }
}

fn try_new_client_order(
    name: &str,
    number: &str,
    piece: &str,
    quantity: &str,
    due_date: &str,
    late_pen: &str,
    early_pen: &str,
) -> Option<ClientOrder> {
    let late_pen = match parse_money(late_pen) {
        Ok((_, pen)) => pen?,
        Err(e) => {
            tracing::error!("{} while parsing late penalty", e);
            return None;
        }
    };

    let early_pen = match parse_money(early_pen) {
        Ok((_, pen)) => pen?,
        Err(e) => {
            tracing::error!("{} while parsing late penalty", e);
            return None;
        }
    };

    Some(ClientOrder {
        client_name: name.to_string(),
        order_number: number.parse().ok()?,
        work_piece: WorkPiece::try_from_str(piece)?,
        quantity: quantity.parse().ok()?,
        due_date: due_date.parse().ok()?,
        late_penalty: late_pen,
        early_penalty: early_pen,
    })
}

fn double_quotes(input: &str) -> IResult<&str, &str> {
    delimited(tag("\""), take_until("\""), tag("\""))(input)
}

fn remove_around_tag<'a>(
    input: &'a str,
    t: &'static str,
) -> IResult<&'a str, &'a str> {
    let (input, _) =
        pair(take_until(t), terminated(tag(t), multispace0))(input)?;

    Ok((input, ""))
}

fn parse(input: &str) -> IResult<&str, Option<ClientOrder>> {
    // First get rid of any XML preamble
    let (input, _) = remove_around_tag(input, "<ClientOrder>")?;

    // Now parse the client name
    let (input, name) = terminated(
        delimited(tag("<Client NameId="), double_quotes, tag("/>")),
        multispace0,
    )(input)?;

    // Now parse the order details
    let (input, number) = preceded(
        pair(tag("<Order"), multispace1),
        delimited(tag("Number="), double_quotes, multispace1),
    )(input)?;
    let (input, piece) =
        delimited(tag("WorkPiece="), double_quotes, multispace1)(input)?;
    let (input, quantity) =
        delimited(tag("Quantity="), double_quotes, multispace1)(input)?;
    let (input, due_date) =
        delimited(tag("DueDate="), double_quotes, multispace1)(input)?;
    let (input, late_pen) =
        delimited(tag("LatePen="), double_quotes, multispace1)(input)?;
    let (input, early_pen) =
        delimited(tag("EarlyPen="), double_quotes, multispace0)(input)?;

    // Now parse the closing tag and any trailing whitespace
    let (input, _) = remove_around_tag(input, "</ClientOrder>")?;

    Ok((
        input,
        try_new_client_order(
            name, number, piece, quantity, due_date, late_pen, early_pen,
        ),
    ))
}

pub fn parse_many(input: &str) -> IResult<&str, Vec<ClientOrder>> {
    fold_many1(parse, Vec::new, |mut acc, order| {
        if let Some(order) = order {
            acc.push(order);
        }
        acc
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_api::{ClientOrder, WorkPiece};

    #[test]
    fn test_parse() {
        let input = r#" <ClientOrder>
                        <Client NameId="John Doe"/>
                        <Order Number="1"
                            WorkPiece="P5"
                            Quantity="10"
                            DueDate="31"
                            LatePen="€100,00"
                            EarlyPen="€50,32"/>
                        </ClientOrder>"#;

        let expected = Some(ClientOrder {
            client_name: "John Doe".to_string(),
            order_number: 1,
            work_piece: WorkPiece::P5,
            quantity: 10,
            due_date: 31,
            late_penalty: 10000,
            early_penalty: 5032,
        });

        let (_, result) = parse(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_form_mock_input() {
        let input = include_str!("../../mock_client_orders.xml");
        let expected = ClientOrder {
            client_name: "Kling Inc".to_string(),
            order_number: 1,
            work_piece: WorkPiece::P9,
            quantity: 3,
            due_date: 4,
            late_penalty: 574,
            early_penalty: 6632,
        };

        let (_, result) = parse(input).unwrap();
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn test_parse_many() {
        let input = r#" -- I AM A PREAMBLE --
                        <ClientOrder>
                        <Client NameId="John Doe"/>
                        <Order Number="1"
                            WorkPiece="P5"
                            Quantity="10"
                            DueDate="31"
                            LatePen="€0,10"
                            EarlyPen="€50,32"/>
                        </ClientOrder>
                        -- I AM FURTHER INVALID DATA --
                        <ClientOrder>
                        <Client NameId="John Doe"/>
                        <Order Number="1"
                            WorkPiece="P5"
                            Quantity="10"
                            DueDate="31"
                            LatePen="€0,10"
                            EarlyPen="€50,32"/>
                        </ClientOrder>"#;

        let expected = vec![
            ClientOrder {
                client_name: "John Doe".to_string(),
                order_number: 1,
                work_piece: WorkPiece::P5,
                quantity: 10,
                due_date: 31,
                late_penalty: 10,
                early_penalty: 5032,
            },
            ClientOrder {
                client_name: "John Doe".to_string(),
                order_number: 1,
                work_piece: WorkPiece::P5,
                quantity: 10,
                due_date: 31,
                late_penalty: 10,
                early_penalty: 5032,
            },
        ];

        let (_, result) = parse_many(input).unwrap();
        assert_eq!(result, expected);
    }
}
