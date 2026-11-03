use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Send the request (No 'mut' needed here anymore)
    let res = reqwest::get("http://httpbin.org/get").await?;

    // 2. EXTRACT data BEFORE consuming the response
    let status = res.status();
    let headers = res.headers().clone(); // Clone the headers to keep a local copy

    // 3. CONSUME the response (This destroys the 'res' variable)
    let body = res.text().await?; 

    // 4. PRINT using the extracted variables instead of 'res'
    println!("Status: {}", status);
    println!("Headers: \n{:#?}", headers);
    println!("Body: \n{}", body);
    
    Ok(())
}
