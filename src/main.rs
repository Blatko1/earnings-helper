mod websites;

use crate::websites::RelativeDay;
use std::io::Write;
use thirtyfour::{DesiredCapabilities, WebDriver};

// Site opens -> wait for the popup and close it 
// -> choose a date on calendar -> find data in html source

#[tokio::main]
async fn main() {
    print!("Initializing WebDriver...");
    std::io::stdout().flush().unwrap();
    let mut caps = DesiredCapabilities::chrome();
    //caps.set_headless().unwrap();
    let driver = WebDriver::new("http://localhost:9515", caps)
        .await
        .map_err(|e| println!("Is chromedriver started? Error: {e}"))
        .unwrap();
    println!("Success!");

    let day = RelativeDay::Tomorrow;

    //let companies1 = websites::marketwatch_data(&driver, day).await.unwrap();
    //println!("companise: {:?}", companies1);
    //let companies2 = websites::zacks_data(&driver, day).await.unwrap();
    //println!("companies2: {:?}", companies2);
    //let companies3 = websites::tradingview_data(&driver, day).await.unwrap();
    //println!("companies3: {:?}", companies3);
    //let companies4 = websites::investing_data(&driver, day).await.unwrap();
    //println!("companies4: {:?}", companies4);
    let companies5 = websites::benzinga_data(&driver, day).await.unwrap();
    println!("companies5: {:?}", companies5);

    //driver.quit().await.unwrap();
}
