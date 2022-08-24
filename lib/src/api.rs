use reqwest::{Client, Error, Response};


#[allow(unused, non_snake_case)]
pub async fn send_amf(target: String, response: String) -> Result<Response, Error> {
    let AMF_URL = "http://s45.youkia.pvz.youkia.com/pvz/amf/".to_owned();
    let client = Client::builder().build()?;
    let resp = client
        .post(AMF_URL)
        .header("Cookie", "value")
        .body("")
        .send()
        .await;
    resp
}
