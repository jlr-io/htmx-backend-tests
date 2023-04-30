use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
// consider using this crate
// https://github.com/sreedevk/pocketbase-sdk-rust

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(index))
        .route("/expenses", get(expenses));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index() -> impl IntoResponse {
    HtmlTemplate(IndexTemplate)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Expense {
    id: String,
    name: String,
    amount: u16,
}

#[derive(Debug, Template)]
#[template(path = "expenses.html")]
struct ExpensesTemplate {
    rows: Vec<Expense>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PaginatedResponse<T> {
    page: u32,
    per_page: u32,
    total_pages: u32,
    total_items: u32,
    items: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpenseRecord {
    id: String,
    name: String,
    amount: u16,
    collection_id: String,
    collection_name: String,
    updated: String,
    created: String,
}

async fn expenses() -> Result<impl IntoResponse, StatusCode> {
    let uri = "http://127.0.0.1:8090/api/collections/expenses/records";
    let pb_request = reqwest::get(uri).await.unwrap();
    let pb_response = pb_request.text().await.unwrap();
    match serde_json::from_str::<PaginatedResponse<ExpenseRecord>>(&pb_response) {
        Ok(response) => {
            let expenses = response
                .items
                .iter()
                .map(|item| Expense {
                    id: item.id.clone(),
                    name: item.name.clone(),
                    amount: item.amount,
                })
                .collect::<Vec<Expense>>();
            return Ok(HtmlTemplate(ExpensesTemplate { rows: expenses }));
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
}
