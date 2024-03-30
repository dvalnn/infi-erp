use std::collections::HashSet;

use actix_web::{
    get, post,
    web::{Data, Form},
    HttpResponse, Responder,
};
use serde::Deserialize;
use sqlx::{postgres::types::PgMoney, PgPool};
use uuid::Uuid;

use crate::{
    db_api::{Item, Order, OrderStatus, TransformationDetails},
    scheduler::CURRENT_DATE,
};

pub fn internal_server_error(
    e: impl std::fmt::Debug + std::fmt::Display,
) -> HttpResponse {
    tracing::error!("{:?}", e);
    HttpResponse::InternalServerError().body(format!("{e}"))
}

#[get("/CheckHealth")]
pub async fn check_health() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
struct DayForm {
    day: u32,
}

#[get("/Date")]
pub async fn get_date() -> impl Responder {
    let current_date = match CURRENT_DATE.read() {
        Ok(date) => date,
        Err(e) => {
            tracing::error!("{:?}", e);
            return HttpResponse::InternalServerError().body(format!("{e}"));
        }
    };

    HttpResponse::Ok().body(format!("{}", *current_date))
}

#[post("/Date")]
pub async fn post_date(form: Form<DayForm>) -> impl Responder {
    let mut current_date = match CURRENT_DATE.write() {
        Ok(date) => date,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("{e}"))
        }
    };
    *current_date = form.day;
    HttpResponse::Created().finish()
}

#[get("/Transformations")]
pub async fn get_daily_transformations(
    form: Form<DayForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return internal_server_error(e),
    };

    let day = form.day as i32;
    let tranfs = match TransformationDetails::get_by_day(day, &mut tx).await {
        Ok(details) => details,
        Err(e) => {
            tracing::error!("{:?}", e);
            return HttpResponse::InternalServerError().body(format!("{e}"));
        }
    };

    tracing::info!("Found {} transformations due on day {}", tranfs.len(), day);

    let mut order_ids = HashSet::new();
    for tf in &tranfs {
        let id = match Order::get_id_by_product(tf.product_id, &mut tx).await {
            Ok(Some(order)) => order,
            Ok(None) => {
                return HttpResponse::InternalServerError()
                    .body("Product not associated with an order");
            }

            Err(e) => {
                return internal_server_error(e);
            }
        };

        order_ids.insert(id);

        let order = match Order::get_by_id(id, &mut tx).await {
            Ok(order) => order,
            Err(e) => {
                return internal_server_error(e);
            }
        };

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
            _ => todo!("Handle other statuses"),
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
    material_id: Uuid,
    product_id: Uuid,
    line_id: String,
    time_taken: i64,
}

#[post("/Transformations")]
pub async fn post_transformation_completion(
    form: Form<TransfCompletionFrom>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            return internal_server_error(e);
        }
    };

    let m_query = Item::get_by_id(form.material_id, &mut tx).await;
    let p_query = Item::get_by_id(form.product_id, &mut tx).await;

    let (material, product) = match (m_query, p_query) {
        (Ok(material), Ok(product)) => (material, product),
        (Err(e), _) | (_, Err(e)) => {
            return internal_server_error(e);
        }
    };

    let material = material.consume(form.line_id.clone());
    let product = match product.produce(
        material.get_cost() + PgMoney(form.time_taken * 100), //NOTE: 100 is the cost per unit time
        form.line_id.clone(),
    ) {
        Ok(product) => product,
        Err(e) => {
            tracing::error!("{:?}", e);
            return HttpResponse::BadRequest().body(format!("{e}"));
        }
    };

    let m_query = material.update(&mut tx).await;
    let p_query = product.update(&mut tx).await;

    let tx_result = match (m_query, p_query) {
        (Ok(_), Ok(_)) => tx.commit().await,
        (Err(e), _) | (_, Err(e)) => {
            return internal_server_error(e);
        }
    };

    if let Err(e) = tx_result {
        return internal_server_error(e);
    }

    HttpResponse::Created().finish()
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
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

#[post("/Warehouse")]
pub async fn post_warehouse_action(
    form: Form<WarehouseActionForm>,
    pool: Data<PgPool>,
) -> impl Responder {
    let mut connection = match pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            return internal_server_error(e);
        }
    };

    let item = match Item::get_by_id(form.item_id, &mut connection).await {
        Ok(item) => item,
        Err(e) => {
            return internal_server_error(e);
        }
    };

    let item = match &form.action_type {
        WarehouseAction::Entry(warehouse_code) => {
            item.enter_warehouse(warehouse_code)
        }
        WarehouseAction::Exit(production_line_code) => {
            item.exit_warehouse(production_line_code)
        }
    };

    match item.update(&mut connection).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => internal_server_error(e),
    }
}

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
