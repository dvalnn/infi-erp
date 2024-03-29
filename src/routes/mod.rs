use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{db_api::TransformationDetails, scheduler::CURRENT_DATE};

#[get("/CheckHealth")]
pub async fn check_health() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Debug, Deserialize, Serialize)]
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
pub async fn post_date(form: web::Form<DayForm>) -> impl Responder {
    let mut current_date = match CURRENT_DATE.write() {
        Ok(date) => date,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("{e}"))
        }
    };
    *current_date = form.day;
    HttpResponse::Created().finish()
}

#[get("/DailyTransformations")]
pub async fn get_daily_transformations(
    form: web::Form<DayForm>,
    pool: web::Data<PgPool>,
) -> impl Responder {
    let mut connection = match pool.acquire().await {
        Ok(conn) => conn,
        Err(e) => {
            tracing::error!("{:?}", e);
            return HttpResponse::InternalServerError().body(format!("{e}"));
        }
    };

    let day = form.day as i32;
    let details = match TransformationDetails::get_by_date(day, &mut connection)
        .await
    {
        Ok(details) => details,
        Err(e) => {
            tracing::error!("{:?}", e);
            return HttpResponse::InternalServerError().body(format!("{e}"));
        }
    };

    HttpResponse::Ok().json(details)
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
            .uri("/DailyTransformations")
            .set_form(DayForm { day: 1 })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let status = resp.status();
        if !resp.status().is_success() {
            let body = test::read_body(resp).await;
            let body_str =
                String::from_utf8(body.to_vec()).expect("Invalid UTF-8");
            panic!("{} : {}", status, body_str);
        }
    }
}
