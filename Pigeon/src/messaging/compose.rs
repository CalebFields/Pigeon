#[allow(dead_code)]
pub async fn compose_message(recipient: &str, body: &str) -> Result<Vec<u8>, crate::error::Error> {
    let _ = (recipient, body);
    Ok(Vec::new())
}


