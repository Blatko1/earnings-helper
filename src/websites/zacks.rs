use chrono::{Datelike, NaiveDate, Weekday};
use std::{time::Duration, vec};
use thirtyfour::{
    prelude::{ElementQueryable, WebDriverError},
    By, WebDriver,
};

use crate::RelativeDay;

use super::{Company, ZACKS};

pub async fn get_zacks_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    driver.goto(ZACKS).await?;
    // Accept cookies in order to remove the cookies 'obstacle' dialog box.
    accept_cookies(driver).await?;

    let today = chrono::offset::Local::now().date_naive();
    let target = day.get_date();
    let target_weekday = target.weekday();
    // Weekdays here are counted from Sun to Sat.
    match today.weekday() {
        // Check if the target week is before the today week
        Weekday::Sun => {
            if target_weekday == Weekday::Sat {
                to_previous_week(driver).await?;
            }
        }
        // Check if the target week is after the today week
        Weekday::Sat => {
            if target_weekday == Weekday::Sun {
                to_next_week(driver).await?;
            }
        }
        _ => (),
    }

    get_data(driver, target).await
}

async fn to_previous_week(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(PREVIOUS_WEEK_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'Previous Week' button")
        .single()
        .await?;
    driver.execute(r#"arguments[0].scrollIntoView({behavior: "auto", block: "center"});"#, vec![button.to_json()?]).await?;
    button.click().await?;
    Ok(())
}

async fn to_next_week(driver: &WebDriver) -> anyhow::Result<()> {
    let button = driver
        .query(By::Css(NEXT_WEEK_SELECTOR))
        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
        .desc("Find 'Next Week' button")
        .single()
        .await?;
    driver.execute(r#"arguments[0].scrollIntoView({behavior: "auto", block: "center"});"#, vec![button.to_json()?]).await?;
    button.click().await?;
    Ok(())
}

async fn accept_cookies(driver: &WebDriver) -> anyhow::Result<()> {
    driver
        .query(By::Id("accept_cookie"))
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .desc("Find the 'Accept cookies' button")
        .single()
        .await?
        .click()
        .await?;
    Ok(())
}

async fn get_data(
    driver: &WebDriver,
    date: NaiveDate,
) -> anyhow::Result<Vec<Company>> {
    let weekday = date.weekday();
    let weekday_num = weekday.num_days_from_sunday();
    let header_id = format!("d_{}", weekday_num);
    let earnings_link_id = format!("cal_link_{}", weekday_num);
    // Retry until the header or the button is clicked 
    // (or maybe there is no button).
    loop {
        if let Ok(e) = driver.query(By::Id(&earnings_link_id)).wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL).desc("Wait for the 'earnings' button to load or find out if it doesn't exist").single().await {
            if e.click().await.is_err() {
                match driver
                    .query(By::Id(&header_id))
                    .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
                    .desc("Find and click date header to access data")
                    .single()
                    .await
                {
                    Ok(e) => match e.click().await {
                        Ok(_) => continue,
                        Err(WebDriverError::NoSuchElement(_)) => continue,
                        Err(e) => return Err(e.into()),
                    },
                    Err(e) => return Err(e.into()),
                }
            }
        } else {
            return Ok(vec![])
        }
        break;
    }

    Ok(vec![])
}

const PARENT: &str = "table[id=\"earnings_rel_data_all_table\"]>tbody";
const SYMBOL_SELECTOR: &str = "";
const PREVIOUS_WEEK_SELECTOR: &str = "div[class=\"prenext_txt align_left\"]>a";
const NEXT_WEEK_SELECTOR: &str = "div[class=\"prenext_txt align_right\"]>a";
const WAIT_INTERVAL: Duration = Duration::from_secs(1);
const TIMEOUT_FIVE_SEC: Duration = Duration::from_secs(5);
const TIMEOUT_TEN_SEC: Duration = Duration::from_secs(10);
