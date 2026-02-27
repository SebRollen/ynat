use ynab_api::{Client, Request, YnabApiError};

#[tokio::main]
pub async fn main() -> Result<(), YnabApiError> {
    let client = Client::new("api_token");

    let req = Request::budgets().list().include_accounts(true);

    let _res = client.send(req).await?;
    Ok(())
}
