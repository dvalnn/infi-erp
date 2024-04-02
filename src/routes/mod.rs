use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use actix_web::{
    get, post,
    web::{self, Data, Form},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::{postgres::types::PgMoney, PgPool};
use uuid::Uuid;

use crate::{
    db_api::{Item, Order, OrderStatus, Transformation, TransformationDetails},
    scheduler::CURRENT_DATE,
};

fn internal_server_error(e: impl Debug + Display) -> HttpResponse {
    tracing::error!("{:?}", e);
    HttpResponse::InternalServerError().body(format!("{e}"))
}

fn bad_request(e: impl Debug + Display) -> HttpResponse {
    tracing::error!("{:?}", e);
    HttpResponse::BadRequest().body(format!("{e}"))
}

#[get("/check_health")]
pub async fn check_health() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct DayForm {
    day: u32,
}

#[get("/date")]
pub async fn get_date() -> impl Responder {
    match CURRENT_DATE.read() {
        Ok(date) => HttpResponse::Ok().body(format!("{}", *date)),
        Err(e) => internal_server_error(e),
    }
}

#[post("/date")]
pub async fn post_date(form: Form<DayForm>) -> impl Responder {
    match CURRENT_DATE.write() {
        Ok(mut date) => {
            *date = form.day;
            HttpResponse::Created().finish()
        }
        Err(e) => internal_server_error(e),
    }
}

#[get("/transformations")]
pub async fn get_daily_transformations(
    query: web::Query<DayForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_server_error(e),
    };

    let day = query.day as i32;
    let tranfs =
        match TransformationDetails::get_pending_by_day(day, &mut tx).await {
            Ok(details) => details,
            Err(e) => return internal_server_error(e),
        };

    tracing::info!(
        "Found {} pending transformations due on day {}",
        tranfs.len(),
        day
    );

    let mut order_ids = HashSet::new();
    for tf in &tranfs {
        let order = match Order::get_by_item_id(tf.product_id, &mut tx).await {
            Err(e) => return internal_server_error(e),
            Ok(Some(order)) => order,
            Ok(None) => {
                tracing::warn!(
                    "No order found for product id {}",
                    tf.product_id
                );
                continue;
            }
        };

        // Skip if this order was already seen on this run
        // Saves some uncessary work
        if !order_ids.insert(order.id()) {
            continue;
        }

        match order.status() {
            OrderStatus::Pending => {
                unreachable!("Pending orders do not have transformations")
            }
            OrderStatus::Scheduled => {
                if let Err(e) = order.production_start(&mut tx).await {
                    return internal_server_error(e);
                }
            }
            OrderStatus::Producing => continue,
            _ => todo!("Handle other order statuses"),
        }
    }

    tracing::info!("Started production for {} orders", order_ids.len());

    if let Err(e) = tx.commit().await {
        return internal_server_error(e);
    }

    HttpResponse::Ok().json(tranfs)
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct TransfCompletionFrom {
    transf_id: i64,
    material_id: Uuid,
    product_id: Uuid,
    line_id: String,
    time_taken: i64,
}

#[post("/transformations")]
pub async fn post_transformation_completion(
    form: Form<TransfCompletionFrom>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_server_error(e),
    };

    let transf = match Transformation::get_by_id(form.transf_id, &mut tx).await
    {
        Ok(tf) => tf,
        Err(e) => return internal_server_error(e),
    };

    if transf.material_id() != form.material_id {
        return bad_request("Material id does not match");
    }

    if transf.product_id() != form.product_id {
        return bad_request("Product id does not match");
    }

    let m_query_res = Item::get_by_id(form.material_id, &mut tx).await;
    let p_query_res = Item::get_by_id(form.product_id, &mut tx).await;
    let (material, product) = match (m_query_res, p_query_res) {
        (Ok(material), Ok(product)) => (material, product),
        (Err(e), _) | (_, Err(e)) => return internal_server_error(e),
    };

    let new_cost = material.get_cost() + PgMoney(form.time_taken * 100);
    let p_action_result = product.produce(new_cost, form.line_id.clone());
    let m_action_result = material.consume(form.line_id.clone());
    let (product, material) = match (p_action_result, m_action_result) {
        (Ok(p), Ok(m)) => (p, m),
        (Err(e), _) | (_, Err(e)) => return bad_request(e),
    };

    let current_date = match CURRENT_DATE.read() {
        Ok(date) => *date,
        Err(e) => return internal_server_error(e),
    };

    let tf_result = transf.complete(current_date, &mut tx).await;
    let m_result = material.update(&mut tx).await;
    let p_result = product.update(&mut tx).await;
    let tx_result = match (m_result, p_result, tf_result) {
        (Ok(_), Ok(_), Ok(_)) => tx.commit().await,
        (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
            return internal_server_error(e)
        }
    };

    if let Err(e) = tx_result {
        return internal_server_error(e);
    }

    HttpResponse::Created().finish()
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename_all = "lowercase")]
enum WarehouseAction {
    Entry(String),
    Exit(String),
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct WarehouseActionForm {
    item_id: Uuid,
    #[serde(flatten)]
    action_type: WarehouseAction,
}

#[post("/warehouse")]
pub async fn post_warehouse_action(
    form: Form<WarehouseActionForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut connection = match pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => return internal_server_error(e),
    };

    let item = match Item::get_by_id(form.item_id, &mut connection).await {
        Ok(item) => item,
        Err(e) => return internal_server_error(e),
    };

    let item_action_result = match &form.action_type {
        WarehouseAction::Entry(warehouse_code) => {
            item.enter_warehouse(warehouse_code)
        }
        WarehouseAction::Exit(production_line_code) => {
            item.exit_warehouse(production_line_code)
        }
    };

    let item = match item_action_result {
        Ok(item) => item,
        Err(e) => return bad_request(e),
    };

    match item.update(&mut connection).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => internal_server_error(e),
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct MaterialArrivalFrom {
    _shipment_id: Uuid,
    _day: u32,
}

#[post("/materials/arrivals")]
pub async fn post_material_arrival(
    _form: Form<MaterialArrivalFrom>,
    _pool: Data<PgPool>,
) -> impl Responder {
    // let raw_materials = todo!("query raw_material_arrivals");
    HttpResponse::NotImplemented()
}

// TODO: material arrivals to warehouse
// TODO: delivery confirmations
#[cfg(test)]
mod tests {
    use super::{check_health, DayForm};
    use crate::{
        configuration::get_configuration,
        routes::{get_daily_transformations, get_date, post_date},
    };
    use actix_web::{test, web::Data, App};

    #[actix_web::test]
    async fn test_check_health() {
        let app = test::init_service(App::new().service(check_health)).await;
        let req = test::TestRequest::get().uri("/CheckHealth").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_date() {
        let app = test::init_service(App::new().service(get_date)).await;
        let req = test::TestRequest::get().uri("/Date").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        String::from_utf8(body.to_vec())
            .expect("Invalid UTF-8")
            .parse::<i32>()
            .expect("Invalid i32");
    }

    #[actix_web::test]
    async fn test_post_date() {
        let app = test::init_service(App::new().service(post_date)).await;
        let req = test::TestRequest::post()
            .uri("/Date")
            .set_form(DayForm { day: 1 })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success())
    }

    #[actix_web::test]
    async fn test_get_daily_transformations() {
        let pool = get_configuration()
            .expect("Failed to read configuration")
            .database
            .create_test_db()
            .await;

        let app = test::init_service(
            App::new()
                .service(get_daily_transformations)
                .app_data(Data::new(pool)),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/Transformations")
            .set_form(DayForm { day: 1 })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        if !resp.status().is_success() {
            let body = test::read_body(resp).await;
            let body_str =
                String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
            panic!("{}: {}", status, body_str);
        }
    }
}
