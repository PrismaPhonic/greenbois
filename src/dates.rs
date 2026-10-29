use chrono::Weekday::{Mon, Sat, Sun, Thu};
use chrono::{DateTime, Datelike, Local, NaiveDate};

type MonthDay = (u8, u8);

// Holidays that fall on the same date every year. Holidays that move year to year, like
// Thanksgiving, are worked out in is_floating_holiday instead.
const NYDAY: MonthDay = (1, 1);
const INDEPENDENCE_DAY: MonthDay = (7, 4);
const DAY_AFTER_INDEP: MonthDay = (7, 5);
const VETERANS_DAY: MonthDay = (11, 11);
const CHRISTMAS_EVE: MonthDay = (12, 24);
const CHRISTMAS_DAY: MonthDay = (12, 25);
const NYE: MonthDay = (12, 31);
const HOLIDAYS: [MonthDay; 7] = [
    NYDAY,
    INDEPENDENCE_DAY,
    DAY_AFTER_INDEP,
    VETERANS_DAY,
    CHRISTMAS_EVE,
    CHRISTMAS_DAY,
    NYE,
];

pub fn is_holiday(date: &DateTime<Local>) -> bool {
    let month_day: MonthDay = (date.month() as u8, date.day() as u8);

    for holiday in HOLIDAYS.iter() {
        if &month_day == holiday {
            return true;
        }
    }

    false
}

/// Checks for holidays that move year to year, worked out for the year of the given date.
pub fn is_floating_holiday(date: &DateTime<Local>) -> bool {
    let day = date.date_naive();
    let year = day.year();

    // Memorial Day is the last Monday of May, which is the fifth Monday when May has one.
    let memorial_day = match NaiveDate::from_weekday_of_month_opt(year, 5, Mon, 5) {
        Some(fifth_monday) => fifth_monday,
        None => NaiveDate::from_weekday_of_month_opt(year, 5, Mon, 4).unwrap(),
    };

    // Labor Day is the first Monday of September.
    let labor_day = NaiveDate::from_weekday_of_month_opt(year, 9, Mon, 1).unwrap();

    // Thanksgiving is the fourth Thursday of November.
    let thanksgiving = NaiveDate::from_weekday_of_month_opt(year, 11, Thu, 4).unwrap();
    let day_after_thanksgiving = thanksgiving.succ_opt().unwrap();

    day == memorial_day
        || day == labor_day
        || day == thanksgiving
        || day == day_after_thanksgiving
}

pub fn should_skip_date(date: &DateTime<Local>) -> bool {
    match date.weekday() {
        Sat | Sun => {
            return true;
        }
        _ => {}
    }

    if is_holiday(date) {
        return true;
    }

    if is_floating_holiday(date) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn local(year: i32, month: u32, day: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(year, month, day, 12, 0, 0).unwrap()
    }

    #[test]
    fn test_floating_holidays() {
        // 2023 has five Mondays in May.
        assert!(is_floating_holiday(&local(2023, 5, 29)));
        assert!(is_floating_holiday(&local(2023, 9, 4)));
        assert!(is_floating_holiday(&local(2023, 11, 23)));
        assert!(is_floating_holiday(&local(2023, 11, 24)));

        assert!(is_floating_holiday(&local(2024, 5, 27)));
        assert!(is_floating_holiday(&local(2024, 9, 2)));
        assert!(is_floating_holiday(&local(2024, 11, 28)));
        assert!(is_floating_holiday(&local(2024, 11, 29)));

        assert!(is_floating_holiday(&local(2025, 5, 26)));
        assert!(is_floating_holiday(&local(2025, 9, 1)));
        assert!(is_floating_holiday(&local(2025, 11, 27)));
        assert!(is_floating_holiday(&local(2025, 11, 28)));
    }

    #[test]
    fn test_not_floating_holidays() {
        // The fourth Monday of May 2023 is not the last one.
        assert!(!is_floating_holiday(&local(2023, 5, 22)));
        // Last year's dates no longer match.
        assert!(!is_floating_holiday(&local(2025, 5, 27)));
        assert!(!is_floating_holiday(&local(2025, 9, 2)));
        // The Thursday before Thanksgiving.
        assert!(!is_floating_holiday(&local(2024, 11, 21)));
    }

    #[test]
    fn test_should_skip_day_after_thanksgiving() {
        assert!(should_skip_date(&local(2024, 11, 29)));
    }
}
