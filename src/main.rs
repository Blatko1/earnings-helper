use chrono::{Days, NaiveDate};
use thirtyfour::{DesiredCapabilities, WebDriver};
use websites::marketwatch;

mod websites;

#[tokio::main]
async fn main() {
    print!("Initializing WebDriver...");
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    let driver = WebDriver::new("http://localhost:9515", caps)
        .await
        .map_err(|e| println!("Is chromedriver started? Error: {e}"))
        .unwrap();
    println!("Success!");

    let day = RelativeDay::Tomorrow;

    print!("Reading MarketWatch data...");
    let mut max_reruns = 1;
    let companies = loop {
        match marketwatch::get_marketwatch_data(&driver, day).await {
            Ok(c) => {
                println!("Success!");
                break c;
            }
            Err(e) => {
                if max_reruns == 0 {
                    println!("\nCouldn't parse data: {e}");
                    break vec![];
                }
                print!("Failed to parse data! Trying again...");
                max_reruns -= 1;
                continue;
            }
        }
    };
    //zacks::get_zacks_data(day).await.unwrap();

    driver.quit().await.unwrap();
}

#[derive(Debug, Clone, Copy)]
pub enum RelativeDay {
    Yesterday,
    Today,
    Tomorrow,
}

impl RelativeDay {
    pub fn get_date(&self) -> NaiveDate {
        use chrono::offset::Local;
        let now = Local::now();
        match self {
            RelativeDay::Yesterday => {
                now.checked_sub_days(Days::new(1)).unwrap().date_naive()
            }
            RelativeDay::Today => now.date_naive(),
            RelativeDay::Tomorrow => {
                now.checked_add_days(Days::new(1)).unwrap().date_naive()
            }
        }
    }
}
