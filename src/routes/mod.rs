use std::{
    collections::HashSet,
    fmt::{Debug, Display},
};

use actix_web::{
    get, post,
    web::{Data, Form, Query},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgMoney, PgPool};
use uuid::Uuid;

use crate::db_api::{
    self, Item, Order, OrderStatus, RawMaterial, Shipment, Transformation,
    TransformationDetails,
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

#[derive(Debug, Deserialize, Serialize)]
struct DayForm {
    day: u32,
}

#[get("/date")]
pub async fn get_date(pool: Data<PgPool>) -> impl Responder {
    let mut con = match pool.acquire().await {
        Ok(con) => con,
        Err(e) => return internal_server_error(e),
    };

    match db_api::get_date(&mut con).await {
        Ok(date) => HttpResponse::Ok().json(DayForm { day: date }),
        Err(e) => internal_server_error(e),
    }
}

#[post("/date")]
pub async fn post_date(
    form: Form<DayForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut con = match pool.acquire().await {
        Ok(con) => con,
        Err(e) => return internal_server_error(e),
    };

    match db_api::update_date(form.day, &mut con).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => internal_server_error(e),
    };

    HttpResponse::Created().finish()
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct ProductionForm {
    max_n_items: u32,
}

#[derive(Debug, Serialize)]
struct Recipe {
    steps: Vec<TransformationDetails>,
}

#[get("/production")]
pub async fn get_production(
    query: Query<ProductionForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_server_error(e),
    };

    let n_items = query.max_n_items as i64;
    let ids = match Transformation::get_n_next_raw_mat_transf(n_items, &mut tx)
        .await
    {
        Ok(ids) => ids,
        Err(e) => return internal_server_error(e),
    };

    let mut recipes = Vec::new();
    for material_id in ids {
        let mut steps = Vec::new();
        let mut material = material_id;
        while let Some(transf) =
            match TransformationDetails::get_by_id(material, &mut tx).await {
                Ok(t) => t,
                Err(e) => return internal_server_error(e),
            }
        {
            material = transf.product_id;
            steps.push(transf);
        }
        recipes.push(Recipe { steps })
    }

    if recipes.iter().any(|r| r.steps.is_empty()) {
        tracing::error!("Some transformations are missing");
        return HttpResponse::NotFound().finish();
    }

    for recipe in &recipes {
        //NOTE: all items in a recipe relate to the same order
        let transf = &recipe.steps[0];
        let order =
            match Order::get_by_item_id(transf.product_id, &mut tx).await {
                Ok(Some(order)) => order,
                Ok(None) => continue,
                Err(e) => return internal_server_error(e),
            };
        if let Err(e) = order.production_start(&mut tx).await {
            return internal_server_error(e);
        }
        tracing::info!("Started production for order {}", order.id());
    }

    if let Err(e) = tx.commit().await {
        return internal_server_error(e);
    }

    HttpResponse::Ok().json(recipes)
}

#[get("/transformations")]
pub async fn get_daily_transformations(
    query: Query<DayForm>,
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

    let current_date = match db_api::get_date(&mut tx).await {
        Ok(date) => date,
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

#[derive(Debug, Deserialize, Serialize)]
struct ExpectedShipmentForm {
    shipment_id: i64,
    material_type: RawMaterial,
    quantity: i32,
}

#[get("/materials/expected")]
pub async fn get_expected_shipments(
    query: Query<DayForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut con = match pool.acquire().await {
        Ok(con) => con,
        Err(e) => return internal_server_error(e),
    };

    let expected =
        Shipment::get_expected_for_arrival(query.day as i32, &mut con);
    let response_body = match expected.await {
        Ok(ship_vec) => ship_vec
            .iter()
            .map(|s| ExpectedShipmentForm {
                shipment_id: s.id,
                material_type: s.material_type,
                quantity: s.quantity,
            })
            .collect::<Vec<_>>(),
        Err(e) => return internal_server_error(e),
    };

    HttpResponse::Ok().json(response_body)
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct ShipmentArrivalForm {
    shipment_id: i64,
}

#[post("/materials/arrivals")]
pub async fn post_material_arrival(
    form: Form<ShipmentArrivalForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let date = {
        let mut con = match pool.acquire().await {
            Ok(con) => con,
            Err(e) => return internal_server_error(e),
        };
        match db_api::get_date(&mut con).await {
            Ok(date) => date as i32,
            Err(e) => return internal_server_error(e),
        }
    };

    match Shipment::arrived(form.shipment_id, date, &pool).await {
        Err(e) => internal_server_error(e),
        Ok(_) => {
            tracing::info!("Shipment {} arrived", form.shipment_id);
            HttpResponse::Created().finish()
        }
    }
}

#[get("/deliveries")]
pub async fn get_deliveries(pool: Data<PgPool>) -> impl Responder {
    let mut con = match pool.acquire().await {
        Ok(con) => con,
        Err(e) => return internal_server_error(e),
    };
    let deliveries = match Order::get_deliveries(&mut con).await {
        Ok(deliveries) => deliveries,
        Err(e) => return internal_server_error(e),
    };
    HttpResponse::Ok().json(deliveries)
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct DeliveryCompletionForm {
    id: Uuid,
}

#[post("/deliveries")]
pub async fn post_delivery_confirmation(
    form: Form<DeliveryCompletionForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut con = match pool.acquire().await {
        Ok(con) => con,
        Err(e) => return internal_server_error(e),
    };
    let date = match db_api::get_date(&mut con).await {
        Ok(date) => date,
        Err(e) => return internal_server_error(e),
    };
    match Order::confirm_delivery(&mut con, form.id, date).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => bad_request(e),
    }
}
