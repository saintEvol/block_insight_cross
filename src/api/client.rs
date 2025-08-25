use crate::api::api_error::ApiError;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use reqwest::{Method, RequestBuilder};

pub(crate) struct Client {
    client: reqwest::Client,
    config: Box<dyn ClientConfig>,
}

pub(crate) static CLIENT: Lazy<RwLock<Option<Client>>> = Lazy::new(|| RwLock::new(None));
pub(crate) static BASE_URL: Lazy<RwLock<Option<String>>> = Lazy::new(|| RwLock::new(None));

pub trait ClientConfig: Send + Sync + 'static {
    fn get_base_url(&self) -> String;
    fn before_request(&self, request_builder: RequestBuilder) -> RequestBuilder;
}

pub fn init(client_config: impl ClientConfig) {
    let base_url = client_config.get_base_url();
    BASE_URL.write().replace(base_url);

    let config = Box::new(client_config) as Box<dyn ClientConfig>;
    let client = reqwest::Client::new();
    let client = Client { client, config };
    CLIENT.write().replace(client);
}

impl Client {
    /// path格式: path,前面不需要/符号
    pub(crate) async fn request<Request, Response>(
        method: Method,
        path: &str,
        params: Request,
    ) -> Result<Option<Response>, ApiError>
    where
        Request: serde::Serialize,
        Response: serde::de::DeserializeOwned,
    {
        let client_read = CLIENT.read();
        let base_url_read = BASE_URL.read();
        if let (Some(client), Some(base_url)) = (client_read.as_ref(), base_url_read.as_ref()) {
            let full_path = format!("{base_url}/{path}");
            client.inner_request(method, &full_path, params).await
        } else {
            Err(ApiError::ClientNotInitialized)
        }
    }

    async fn inner_request<Request, ResponseType>(
        &self,
        method: Method,
        full_path: &str,
        params: Request,
    ) -> Result<Option<ResponseType>, ApiError>
    where
        Request: serde::Serialize,
        ResponseType: serde::de::DeserializeOwned,
    {
        let builder = self.client.request(method, full_path);
        let builder = self.config.before_request(builder);
        let send_result = builder.json(&params).send().await?;
        let resp = send_result
            .json::<rust_utils::http_utils::response::Response<ResponseType>>()
            .await?;
        if resp.is_success() {
            Ok(resp.data)
        } else {
            if let Some(err) = resp.message {
                Err(ApiError::LogicError(err))
            } else {
                Err(ApiError::LogicError("未知错误".to_string()))
            }
        }
    }
}
