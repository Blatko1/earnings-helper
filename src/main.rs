use chrono::{Days, NaiveDate};
use thirtyfour::{DesiredCapabilities, WebDriver};
use websites::{marketwatch, zacks};

mod websites;

#[tokio::main]
async fn main() {
    let day = RelativeDay::Today;
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    let driver = WebDriver::new("http://localhost:9515", caps).await.unwrap();

    let companies =
        marketwatch::get_marketwatch_data_relative_day(client, day).await.unwrap();

    zacks::get_zacks_data(day).await.unwrap();
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
