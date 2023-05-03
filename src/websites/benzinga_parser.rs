use thirtyfour::WebDriver;

use super::{
    Company, BENZINGA
};
use crate::RelativeDay;



pub async fn get_data(
    driver: &WebDriver,
    day: RelativeDay,
) -> anyhow::Result<Vec<Company>> {
    driver.goto(BENZINGA).await?;



    parse_data(driver).await
}

async fn parse_data(driver: &WebDriver) -> anyhow::Result<Vec<Company>> {
    Ok(vec![])
}
