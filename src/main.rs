use chrono::NaiveDate;

mod websites;

fn main() {
    websites::marketwatch::get_marketwatch_data(NaiveDate::from_ymd_opt(2023, 4, 19).unwrap()).unwrap();
    websites::zacks::get_zacks_data().unwrap();
}