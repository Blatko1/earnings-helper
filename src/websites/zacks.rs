use thirtyfour::{By, DesiredCapabilities, WebDriver};

use crate::RelativeDay;

use super::ZACKS;

pub async fn get_zacks_data(day: RelativeDay) -> anyhow::Result<()> {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless()?;
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    let date = day.get_date();

    driver.goto(ZACKS).await?;
    driver.find(By::Id("date_select")).await?.click().await?;
    driver.find(By::Id("dt_12")).await?.click().await?;
    //std::fs::write("image.png", driver.screenshot_as_png().await?)?;

    driver.quit().await?;
    Ok(())
}

const PARENT: &str = "table[id=\"earnings_rel_data_all_table\"]>tbody";
const SYMBOL_SELECTOR: &str = "";
