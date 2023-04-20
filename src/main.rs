mod websites;

use std::io::Write;
use thirtyfour::{DesiredCapabilities, WebDriver};
use crate::websites::RelativeDay;

#[tokio::main]
async fn main() {
    print!("Initializing WebDriver...");
    std::io::stdout().flush().unwrap();
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    let driver = WebDriver::new("http://localhost:9515", caps)
        .await
        .map_err(|e| println!("Is chromedriver started? Error: {e}"))
        .unwrap();
    println!("Success!");

    let day = RelativeDay::Today;
    let companies1 = websites::marketwatch_data(&driver, day).await.unwrap();
    println!("companise: {:?}", companies1);

    let companies2 = websites::zacks_data(&driver, day).await.unwrap();

    driver.quit().await.unwrap();
}
