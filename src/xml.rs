use serde::{Deserialize, Serialize};
use serde_xml_rs::from_str;
use sqlx::{error::BoxDynError, postgres::types::PgMoney};

use crate::db::ClientOrder;

#[derive(Debug, Deserialize)]
struct Dataset {
    #[serde(rename = "$value")]
    orders: Vec<RawClientOrder>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RawClientOrder {
    pub ordernumber: i64,
    pub clientnameid: String,
    pub workpiece: String, //TODO: Change to enum if possible
    pub quantity: i32,
    pub duedate: i32,
    pub latepen: String,
    pub earlypen: String,
}

impl TryInto<ClientOrder> for RawClientOrder {
    type Error = BoxDynError;
    fn try_into(self) -> Result<ClientOrder, Self::Error> {
        let mut latepen = self.latepen.clone();
        let mut earlypen = self.earlypen.clone();

        for c in ['$', '.'] {
            if let Some(index) = latepen.find(c) {
                latepen.remove(index);
            }
            if let Some(index) = earlypen.find(c) {
                earlypen.remove(index);
            }
        }

        let latepen = PgMoney::from(latepen.parse::<i64>()?);
        let earlypen = PgMoney::from(earlypen.parse::<i64>()?);

        Ok(ClientOrder {
            ordernumber: self.ordernumber,
            clientnameid: self.clientnameid,
            workpiece: self.workpiece,
            quantity: self.quantity,
            duedate: self.duedate,
            latepen,
            earlypen,
        })
    }
}

pub async fn parse_xml(data: &str) -> Result<Vec<ClientOrder>, BoxDynError> {
    let file = std::fs::read_to_string(data)?;
    let dataset: Dataset = from_str(file.as_str())?;
    let orders = dataset
        .orders
        .into_iter()
        .map(|o| o.try_into())
        .collect::<Vec<Result<ClientOrder, BoxDynError>>>();

    orders
        .into_iter()
        .collect::<Result<Vec<ClientOrder>, BoxDynError>>()
}