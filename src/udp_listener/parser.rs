use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, multispace1},
    multi::fold_many1,
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

use crate::db_api::{ClientOrder, FinalPiece};

fn parse_euros_and_cents(input: &str) -> anyhow::Result<i64> {
    let (euros_str, cents_str) = match input.split_once(',') {
        Some((euros, cents)) => (euros, cents),
        None => match input.split_once('.') {
            Some((euros, cents)) => (euros, cents),
            None => anyhow::bail!("Invalid currency separator"),
        },
    };

    let cents = match cents_str.len() {
        1 => cents_str.parse::<i64>()? * 10,
        2 => cents_str.parse::<i64>()?,
        _ => anyhow::bail!("Invalid number of cents"),
    };

    let euros = match euros_str.parse::<i64>() {
        Ok(e) => e * 100,
        Err(e) => anyhow::bail!("{}", e),
    };

    Ok(euros + cents)
}

fn parse_money(input: &str) -> anyhow::Result<i64> {
    let input = input.replace('€', "");
    let has_cents = input.contains('.') || input.contains(',');

    if has_cents {
        return match parse_euros_and_cents(&input) {
            Ok(money) => Ok(money),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        };
    }

    match input.parse::<i64>() {
        Ok(money) => Ok(money * 100),
        Err(e) => Err(anyhow::anyhow!("{}", e)),
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
        Ok(pen) => pen,
        Err(e) => {
            tracing::error!("{} while parsing late penalty", e);
            return None;
        }
    };

    let early_pen = match parse_money(early_pen) {
        Ok(pen) => pen,
        Err(e) => {
            tracing::error!("{} while parsing late penalty", e);
            return None;
        }
    };

    Some(ClientOrder {
        client_name: name.to_string(),
        order_number: number.parse().ok()?,
        work_piece: FinalPiece::try_from(piece).ok()?,
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
    use rstest::rstest;

    use super::*;
    use crate::db_api::{ClientOrder, FinalPiece};

    struct Money {
        euros: i64,
        cents: i64,
    }

    impl Money {
        const fn new(euros: i64, cents: i64) -> Self {
            Self { euros, cents }
        }
    }

    struct OrderParams {
        name: &'static str,
        number: i32,
        piece: FinalPiece,
        quantity: i32,
        due_date: i32,
        late_pen: Money,
        early_pen: Money,
    }

    impl OrderParams {
        const fn new(
            name: &'static str,
            number: i32,
            piece: FinalPiece,
            quantity: i32,
            due_date: i32,
            late_pen: Money,
            early_pen: Money,
        ) -> Self {
            Self {
                name,
                number,
                piece,
                quantity,
                due_date,
                late_pen,
                early_pen,
            }
        }

        fn to_client_order(&self) -> ClientOrder {
            ClientOrder {
                client_name: self.name.to_string(),
                order_number: self.number,
                work_piece: self.piece,
                quantity: self.quantity,
                due_date: self.due_date,
                late_penalty: self.late_pen.euros * 100 + self.late_pen.cents,
                early_penalty: self.early_pen.euros * 100
                    + self.early_pen.cents,
            }
        }

        fn as_mock_input(&self) -> String {
            format!(
                r#"<ClientOrder>
            <Client NameId="{0}"/>
            <Order Number="{1}"
                WorkPiece="{2}"
                Quantity="{3}"
                DueDate="{4}"
                LatePen="€{5},{6}"
                EarlyPen="€{7},{8}"/>
            </ClientOrder>"#,
                self.name,
                self.number,
                self.piece,
                self.quantity,
                self.due_date,
                self.late_pen.euros,
                self.late_pen.cents,
                self.early_pen.euros,
                self.early_pen.cents,
            )
        }
    }

    static MOCK_ORDERS: [OrderParams; 3] = [
        OrderParams::new(
            "John Doe",
            1,
            FinalPiece::P5,
            10,
            31,
            Money::new(0, 10),
            Money::new(50, 32),
        ),
        OrderParams::new(
            "Kling Inc",
            1,
            FinalPiece::P9,
            3,
            4,
            Money::new(5, 74),
            Money::new(66, 32),
        ),
        OrderParams::new(
            "John Doe",
            2,
            FinalPiece::P5,
            10,
            31,
            Money::new(20, 0),
            Money::new(0, 0),
        ),
    ];

    #[rstest]
    #[case(&MOCK_ORDERS[0].as_mock_input(), Some(MOCK_ORDERS[0].to_client_order()))]
    #[case(&MOCK_ORDERS[1].as_mock_input(), Some(MOCK_ORDERS[1].to_client_order()))]
    #[case(&MOCK_ORDERS[2].as_mock_input(), Some(MOCK_ORDERS[2].to_client_order()))]
    fn test_parse(#[case] input: &str, #[case] expected: Option<ClientOrder>) {
        let (_, result) = parse(input).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_many() {
        let input = format!(
            "--I AM PREAMBLE-- {0}{1} --I AM SOME MUMBO JUMBO-- {2} -- I AM POSTAMBLE--",
            MOCK_ORDERS[0].as_mock_input(),
            MOCK_ORDERS[1].as_mock_input(),
            MOCK_ORDERS[2].as_mock_input()
            
        );
        let expected = vec![
            MOCK_ORDERS[0].to_client_order(),
            MOCK_ORDERS[1].to_client_order(),
            MOCK_ORDERS[2].to_client_order()
        ];

        let (_, result) = parse_many(&input).unwrap();
        assert_eq!(result, expected);
    }
}
