use chrono::{format::DelayedFormat, Datelike, Days, NaiveDate, Weekday};
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
    let weekday_num = weekday.num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    let header_text = format!(
        "Events For {} - {}",
        week_start.format("%-m/%-d/%Y"),
        week_end.format("%-m/%-d/%Y")
    );
    let header_id = format!("d_{}", weekday_num);
    let earnings_link_id = format!("cal_link_{}", weekday_num);
    println!("{earnings_link_id}");
    // Retry until the header or the button is clicked
    // (or maybe there is no button).
    driver
        .query(By::Id("WeeklyEventsTitle"))
        .with_text(header_text)
        .wait(TIMEOUT_TEN_SEC, WAIT_INTERVAL)
        .single()
        .await?;
    if let Ok(e) = driver.query(By::Id(&earnings_link_id))
    .with_text("Earnings").wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
    .desc("Wait for the 'earnings' button to load or find out if it doesn't exist").single().await {
        loop {
            if e.click().await.is_err() {
                driver
                        .query(By::Id(&header_id))
                        .wait(TIMEOUT_FIVE_SEC, WAIT_INTERVAL)
                        .desc("Find and click date header to access data")
                        .single()
                        .await?.click().await?;
                continue;
            }
            break;
        }
    }
    else {
        return Ok(vec![ ])
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

#[test]
fn week_range_dates() {
    let date = NaiveDate::from_ymd_opt(2023, 4, 17).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/16/2023 - 4/22/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 16).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/16/2023 - 4/22/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 15).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/9/2023 - 4/15/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 6).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/2/2023 - 4/8/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 24).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/23/2023 - 4/29/2023"
    );

    let date = NaiveDate::from_ymd_opt(2023, 4, 30).unwrap();
    let weekday_num = date.weekday().num_days_from_sunday() as u64;
    let week_start = date.checked_sub_days(Days::new(weekday_num)).unwrap();
    let week_end = date.checked_add_days(Days::new(6 - weekday_num)).unwrap();
    assert_eq!(
        format!(
            "Events For {} - {}",
            week_start.format("%-m/%-d/%Y"),
            week_end.format("%-m/%-d/%Y")
        ),
        "Events For 4/30/2023 - 5/6/2023"
    );
}
