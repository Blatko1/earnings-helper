use super::ZACKS;
use headless_chrome::Browser;

pub fn get_zacks_data() -> anyhow::Result<()> {
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    tab.navigate_to(ZACKS)?;
    tab.wait_for_element("#earnings_rel_data_all_table_wrapper")?;
    std::fs::write("site2", tab.get_content()?)?;
    Ok(())
}
