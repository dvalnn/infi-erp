use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::scheduler::CURRENT_DATE;

#[get("/CheckHealth")]
pub async fn check_health() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct NewDayForm {
    date: i32,
}

#[get("/Date")]
pub async fn get_date() -> impl Responder {
    let current_date = match CURRENT_DATE.read() {
        Ok(date) => date,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("{e}"))
        }
    };

    HttpResponse::Ok().body(format!("{}", *current_date))
}

#[post("/Date")]
pub async fn post_date(form: web::Form<NewDayForm>) -> impl Responder {
    let mut current_date = match CURRENT_DATE.write() {
        Ok(date) => date,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("{e}"))
        }
    };
    *current_date = form.date;
    HttpResponse::Created().finish()
}

#[get("/Orders/DeliveryDay/{day}")]
pub async fn get_orders_by_day(
    _day: web::Path<i32>,
    _pool: web::Data<PgPool>,
) -> impl Responder {
    HttpResponse::InternalServerError().body("Not implemented")
}

#[cfg(test)]
mod tests {
    use super::{check_health, NewDayForm};
    use crate::routes::{get_date, post_date};
    use actix_web::{test, App};

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
            .set_form(NewDayForm { date: 1 })
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success())
    }
}
