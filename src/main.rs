use websites::{marketwatch, zacks};

mod websites;

#[tokio::main]
async fn main() {
    let today = chrono::offset::Local::now();
    let companies = marketwatch::get_marketwatch_data(today).unwrap();
    zacks::get_zacks_data(today).unwrap();
}
