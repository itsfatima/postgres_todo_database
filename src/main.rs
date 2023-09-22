use actix_web::{web, App, HttpServer, HttpResponse};
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use crate::db::init_pool;
use sqlx::query;
mod db;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
struct Todo {
    id: i32,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct NewTodo {
    title: String,
}

#[derive(Debug, Deserialize)]
struct UpdatedTodo {
    title: String,
    completed: bool,
}

async fn get_todos(pool: web::Data<PgPool>) -> HttpResponse {
    // Use sqlx query macro to fetch todos
    let todos = query!("SELECT id, title, completed FROM todos")
        .fetch_all(pool.get_ref())
        .await;

    match todos {
        Ok(rows) => {
            // Convert the rows into a Vec of Todo
            let todos: Vec<Todo> = rows.iter().map(|row| Todo {
                id: row.id,
                title: row.title.clone(),
                completed: row.completed,
            }).collect();

            HttpResponse::Ok().json(todos)
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn create_todo(
    pool: web::Data<PgPool>,
    new_todo: web::Json<NewTodo>,
) -> HttpResponse {
    let new_todo = new_todo.into_inner();

    // Use sqlx query macro to insert a new todo
    let result = query!(
        "INSERT INTO todos (title, completed) VALUES ($1, $2)",
        new_todo.title,
        false // Initially, a new Todo is not completed
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_todo(
    pool: web::Data<PgPool>,
    path: web::Path<(i32,)>,
) -> HttpResponse {
    let id = path.0;
    let result = sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn update_todo(
    pool: web::Data<PgPool>,
    path: web::Path<(i32,)>,
    updated_todo: web::Json<UpdatedTodo>,
) -> HttpResponse {
    let id = path.0;
    let updated_todo = updated_todo.into_inner();
    let result = sqlx::query("UPDATE todos SET title = $1, completed = $2 WHERE id = $3")
        .bind(&updated_todo.title)
        .bind(updated_todo.completed)
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}


use env_logger;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    // Initialize the logger
    env_logger::init();

    // Initialize the database pool
    let pool = init_pool().await.expect("Failed to initialize pool");

    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone())) // Using .app_data to set the data for the specific scope
            .route("/todo", web::get().to(get_todos))
            .route("/todo", web::post().to(create_todo))
            .route("/todo/{id}", web::delete().to(delete_todo))
            .route("/todo/{id}", web::put().to(update_todo))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}



