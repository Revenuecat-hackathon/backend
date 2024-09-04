use actix_web::{post, web};
use crate::utils::{api_response::ApiResponse, app_state};
use aws_sdk_bedrockruntime::primitives::Blob;
use serde_json::json;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct VibeRequest {
    input_text: String,
    max_tokens: Option<u32>,
}

const REQUEST_MAX_TOKENS: u32 = 200;

#[post("/map")]
pub async fn index(app_state: web::Data<app_state::AppState>,
                vibe:web::Json<VibeRequest>) -> Result<ApiResponse, ApiResponse> {

    println!("[+] /map - invoked {:#?}", vibe.0);

    let mut request_body = json!({
        "inputText": vibe.input_text,
        "textGenerationConfig": {}
    });
    request_body["textGenerationConfig"]["maxTokenCount"] = json!(vibe.max_tokens.unwrap_or(REQUEST_MAX_TOKENS));

    let result = app_state.bedrock_client
    .invoke_model()
    .model_id("amazon.titan-text-express-v1")
    .content_type("application/json")
    .body(Blob::new(serde_json::to_string(&request_body).unwrap()))
    .send()
    .await
    .map_err(|err| {
        println!("map_handlers /map response error: {:#?}", err);
        ApiResponse::new(500, "The service is unable to respond at this time".to_string())}
    )?;

    let output = std::str::from_utf8(result.body().as_ref()).unwrap();
    println!("[+] /map - invoked - got response {}", output);
    Ok(ApiResponse::new(200, output.to_string()))
}
