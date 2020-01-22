use time::Date;
use time::Weekday::{Saturday, Sunday};

const NYDAY: Date = date!(2019-01-01);
const MEMORIAL_DAY: Date = date!(2019-05-27);
const INDEPENDENCE_DAY: Date = date!(2019-07-04);
const DAY_AFTER_INDEP: Date = date!(2019-07-05);
const LABOR_DAY: Date = date!(2019-09-02);
const VETERANS_DAY: Date = date!(2019-11-11);
const THANKSGIVING: Date = date!(2019-11-28);
const DAY_AFTER_THANX: Date = date!(2019-11-28);
const CHRISTMAS_EVE: Date = date!(2019-12-24);
const CHRISTMAS_DAY: Date = date!(2019-12-25);
const NYE: Date = date!(2019-12-31);
const HOLIDAYS: [Date; 11] = [NYDAY, MEMORIAL_DAY, INDEPENDENCE_DAY, DAY_AFTER_INDEP, LABOR_DAY, VETERANS_DAY, THANKSGIVING, DAY_AFTER_THANX, CHRISTMAS_EVE, CHRISTMAS_DAY, NYE];

pub fn is_holiday(date: Date) -> bool {
    for holiday in HOLIDAYS.iter() {
        if date.month_day() == holiday.month_day() {
            return true
        }
    }

    false
}

pub fn should_skip_date(date: Date) -> bool {
    match date.weekday() {
        Saturday | Sunday => { return true; },
        _ => {},
    }

    if is_holiday(date) {
        return true;
    }

    false
}
