use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, multispace1},
    multi::fold_many1,
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

use crate::db_api::{ClientOrder, FinalPiece};

#[derive(Debug)]
struct OrderParams {
    number: i32,
    piece: FinalPiece,
    quantity: i32,
    due_date: i32,
    late_penalty: i64,
    early_penalty: i64,
}

impl OrderParams {
    fn into_client_order(self, client_name: impl ToString) -> ClientOrder {
        ClientOrder {
            client_name: client_name.to_string(),
            order_number: self.number,
            work_piece: self.piece,
            quantity: self.quantity,
            due_date: self.due_date,
            late_penalty: self.late_penalty,
            early_penalty: self.early_penalty,
        }
    }
}

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

fn try_new_order_params(
    number: &str,
    piece: &str,
    quantity: &str,
    due_date: &str,
    late_pen: &str,
    early_pen: &str,
) -> Option<OrderParams> {
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

    Some(OrderParams {
        number: number.parse().ok()?,
        piece: FinalPiece::try_from(piece).ok()?,
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

fn parse_orders(input: &str) -> IResult<&str, Option<OrderParams>> {
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
    let (input, _) = terminated(tag("/>"), multispace0)(input)?;

    Ok((
        input,
        try_new_order_params(
            number, piece, quantity, due_date, late_pen, early_pen,
        ),
    ))
}

pub fn parse_command(input: &str) -> IResult<&str, Vec<ClientOrder>> {
    // First get rid of any XML preamble
    let (input, _) = remove_around_tag(input, "<DOCUMENT>")?;

    // Now parse the client name
    let (input, name) = terminated(
        delimited(tag("<Client NameId="), double_quotes, tag("/>")),
        multispace0,
    )(input)?;

    let (input, orders) =
        fold_many1(parse_orders, Vec::new, |mut acc, order| {
            if let Some(order) = order {
                acc.push(order);
            }
            acc
        })(input)?;

    // Now parse the closing tag and any trailing whitespace
    let (input, _) = remove_around_tag(input, "</DOCUMENT>")?;

    let client_orders = orders
        .into_iter()
        .map(|order| order.into_client_order(name))
        .collect::<Vec<ClientOrder>>();

    Ok((input, client_orders))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::db_api::ClientOrder;

    #[rstest]
    #[case("tests/mock_commands/command1.xml",
        vec![
            ClientOrder::new("Client AA".to_string(), 18, FinalPiece::P5, 8, 7, 1000, 500),
            ClientOrder::new("Client AA".to_string(), 19, FinalPiece::P6, 1, 4, 1000, 1000)
        ]
    )]
    #[case("tests/mock_commands/command2a.xml",
        vec![
            ClientOrder::new("Client AA".to_string(), 41, FinalPiece::P5, 3, 10, 1000, 500),
            ClientOrder::new("Client AA".to_string(), 42, FinalPiece::P9, 2, 8, 1000, 500)
        ]
    )]
    #[case("tests/mock_commands/command2b.xml",
        vec![
            ClientOrder::new("Client BB".to_string(), 47, FinalPiece::P6, 8, 6, 1000, 500),
            ClientOrder::new("Client BB".to_string(), 46, FinalPiece::P7, 2, 6, 1000, 500)
        ]
    )]
    #[case("tests/mock_commands/command3.xml",
        vec![
            ClientOrder::new("Client CC".to_string(), 905, FinalPiece::P5, 3, 8, 1000, 500),
            ClientOrder::new("Client CC".to_string(), 906, FinalPiece::P6, 2, 8, 1000, 200)
        ]
    )]
    #[case("tests/mock_commands/command4.xml",
        vec![
            ClientOrder::new("Client BB".to_string(), 42, FinalPiece::P7, 3, 12, 1000, 500),
            ClientOrder::new("Client BB".to_string(), 43, FinalPiece::P7, 2, 12, 1500, 500),
            ClientOrder::new("Client BB".to_string(), 44, FinalPiece::P6, 4, 12, 1500, 200),
            ClientOrder::new("Client BB".to_string(), 45, FinalPiece::P5, 8, 11, 1500, 200),
        ]
    )]
    #[case("tests/mock_commands/command5.xml",
        vec![
            ClientOrder::new("Client CC".to_string(), 991, FinalPiece::P9, 5, 10, 2000, 500),
        ]
    )]
    fn parse_mock_command(
        #[case] filepath: String,
        #[case] expected: Vec<ClientOrder>,
    ) {
        let input = std::fs::read_to_string(filepath).expect("File not found");

        let (_, orders) = parse_command(&input).unwrap();

        assert_eq!(orders, expected);
    }
}
