use chrono::{Local, Datelike, Date};
use chrono::Weekday::{Sat,Sun};

type MonthDay = (u8, u8);

const NYDAY: MonthDay = (01, 01);
const MEMORIAL_DAY: MonthDay = (05, 27);
const INDEPENDENCE_DAY: MonthDay = (07, 04);
const DAY_AFTER_INDEP: MonthDay = (07, 05);
const LABOR_DAY: MonthDay = (09, 02);
const VETERANS_DAY: MonthDay = (11, 11);
const THANKSGIVING: MonthDay = (11, 28);
const DAY_AFTER_THANX: MonthDay = (11, 28);
const CHRISTMAS_EVE: MonthDay = (12, 24);
const CHRISTMAS_DAY: MonthDay = (12, 25);
const NYE: MonthDay = (12, 31);
const HOLIDAYS: [MonthDay; 11] = [
    NYDAY,
    MEMORIAL_DAY,
    INDEPENDENCE_DAY,
    DAY_AFTER_INDEP,
    LABOR_DAY,
    VETERANS_DAY,
    THANKSGIVING,
    DAY_AFTER_THANX,
    CHRISTMAS_EVE,
    CHRISTMAS_DAY,
    NYE,
];

pub fn is_holiday(date: Date<Local>) -> bool {
    let month_day: MonthDay = (date.month() as u8, date.day() as u8);

    for holiday in HOLIDAYS.iter() {
        if &month_day == holiday {
            return true;
        }
    }

    false
}

pub fn should_skip_date(date: Date<Local>) -> bool {
    match date.weekday() {
        Sat | Sun => {
            return true;
        }
        _ => {}
    }

    if is_holiday(date) {
        return true;
    }

    false
}
