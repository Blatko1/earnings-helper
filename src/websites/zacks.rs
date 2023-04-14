use chrono::{DateTime, Local};
use headless_chrome::{Browser, protocol::cdp::Page};

use super::ZACKS;

pub async fn get_zacks_data(date: DateTime<Local>) -> anyhow::Result<()> {
    

    Ok(())
}

const PARENT: &str = "table[id=\"earnings_rel_data_all_table\"]>tbody";
const SYMBOL_SELECTOR: &str = "";