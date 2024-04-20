use serde::{Deserialize};
use serde_json::{json, Value};
use aws_sdk_dynamodb::{Client,types::AttributeValue};
use aws_sdk_xray as xray;
use lambda_runtime::{LambdaEvent, Error as LambdaError, service_fn};

#[derive(Deserialize)]
struct Request {
    id: Option<String>,
    student_name: Option<String>,    
}

#[::tokio::main]
async fn main() -> Result<(), xray::Error> {
    let config = aws_config::load_from_env().await;
    let _client = aws_sdk_xray::Client::new(&config);
    let _func = service_fn(handler);
    Ok(())
}


async fn handler(event: LambdaEvent<Value>) -> Result<Value, LambdaError> {
    let request: Request = match serde_json::from_value(event.payload) {
        Ok(request) => request,
        Err(_) => return Err(anyhow::anyhow!("Invalid payload").into()),
    };
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    check_student(&client, request.id.clone(), request.student_name.clone()).await?;
    if let Err(_) = check_student(&client, request.id.clone(), request.student_name.clone()).await {
        add_student(&client, request.id, request.student_name).await?;
        println!("Student added successfully");
    } else {
        println!("Student already exists");
    }

    Ok(json!({ "message": "Student checked and added successfully" }))
}

async fn add_student(client: &Client, id: Option<String>, student_name: Option<String>) -> Result<(), LambdaError> {
    let _table_name = "testtable";
    let id_v = AttributeValue::S(id.expect("id is required"));
    let student_name_v = student_name.expect("name is required").to_owned();
    
    client.put_item()
        .table_name("testtable")
        .item("id", id_v.clone())
        .item("student_name", AttributeValue::S(student_name_v))
        .send()
        .await?;

    Ok(())
}


async fn check_student(client: &Client, id: Option<String>, student_name: Option<String>) -> Result<(), LambdaError> {
    let table_name = "testtable";
    let id = AttributeValue::S(id.expect("id is required"));
    let _student_name = student_name.as_ref().expect("name is required");
    let response = client.get_item()
        .table_name(table_name)
        .key("id", id)
        .send()
        .await?;
    
    if response.item.is_some() {
        println!("Student exists");
    } else {
        println!("Student does not exist");
    }
    
    Ok(())
}