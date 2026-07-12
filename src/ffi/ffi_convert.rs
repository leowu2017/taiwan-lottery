use std::ffi::CString;
use std::os::raw::c_char;

use super::{
    DrawNumbersC, DrawResultC, HistoryDrawItemC, HistoryDrawPageC,
    LotteryGameMetadataC, LotteryGameNumberRuleC, LotteryGameQueryRangeC, SortedDrawNumbersC,
};

pub(crate) fn history_page_to_c(page: crate::HistoryDrawPage) -> Box<HistoryDrawPageC> {
    let mut items = Vec::with_capacity(page.items.len());
    for item in page.items {
        items.push(history_item_to_c(item));
    }

    let item_count = items.len();
    let boxed_items = items.into_boxed_slice();
    let items_ptr = Box::into_raw(boxed_items) as *mut HistoryDrawItemC;

    Box::new(HistoryDrawPageC {
        total_size: page.total_size,
        item_count,
        items: items_ptr,
    })
}

pub(crate) fn lottery_game_query_range_to_c(
    range: crate::LotteryGameQueryRange,
) -> Box<LotteryGameQueryRangeC> {
    Box::new(LotteryGameQueryRangeC {
        min_month: string_to_c_ptr(range.min_month),
        max_month: string_to_c_ptr(range.max_month),
    })
}

fn lottery_game_number_rule_to_c(rule: crate::LotteryGameNumberRule) -> LotteryGameNumberRuleC {
    LotteryGameNumberRuleC {
        name: string_to_c_ptr(rule.name.to_string()),
        picks: rule.picks,
        min: rule.min,
        max: rule.max,
        allow_repeat: i32::from(rule.allow_repeat),
    }
}

pub(crate) fn lottery_game_metadata_to_c(metadata: crate::LotteryGameMetadata) -> Box<LotteryGameMetadataC> {
    let mut rules = Vec::with_capacity(metadata.number_ranges.len());
    for rule in metadata.number_ranges {
        rules.push(lottery_game_number_rule_to_c(*rule));
    }

    let number_range_count = rules.len();
    let boxed_rules = rules.into_boxed_slice();
    let number_ranges = if number_range_count == 0 {
        std::ptr::null_mut()
    } else {
        Box::into_raw(boxed_rules) as *mut LotteryGameNumberRuleC
    };

    Box::new(LotteryGameMetadataC {
        display_name: string_to_c_ptr(metadata.display_name.to_string()),
        display_name_english: string_to_c_ptr(metadata.display_name_english.to_string()),
        display_name_chinese: string_to_c_ptr(metadata.display_name_chinese.to_string()),
        number_rule: string_to_c_ptr(metadata.number_rule.to_string()),
        number_range_count,
        number_ranges,
    })
}

pub(crate) fn draw_result_to_c(result: crate::DrawResult) -> Box<DrawResultC> {
    Box::new(DrawResultC {
        base: draw_numbers_to_c(result.base),
        has_bonus: i32::from(result.bonus.is_some()),
        bonus: result.bonus.unwrap_or_default(),
    })
}

fn history_item_to_c(item: crate::HistoryDrawItem) -> HistoryDrawItemC {
    HistoryDrawItemC {
        period: string_to_c_ptr(item.period),
        date: optional_string_to_c_ptr(item.date),
        redeemable_date: optional_string_to_c_ptr(item.redeemable_date),
        numbers: sorted_draw_numbers_to_c(item.numbers),
    }
}

fn draw_numbers_to_c(numbers: crate::DrawNumbers) -> DrawNumbersC {
    let numbers_len = numbers.numbers.len();
    let numbers_ptr = if numbers_len == 0 {
        std::ptr::null_mut()
    } else {
        Box::into_raw(numbers.numbers.into_boxed_slice()) as *mut i32
    };

    DrawNumbersC {
        numbers: numbers_ptr,
        numbers_len,
    }
}

fn sorted_draw_numbers_to_c(numbers: crate::SortedDrawNumbers) -> SortedDrawNumbersC {
    let (sorted_numbers, sorted_numbers_len) = match numbers.sorted {
        Some(sorted_numbers) if !sorted_numbers.is_empty() => {
            let len = sorted_numbers.len();
            let ptr = Box::into_raw(sorted_numbers.into_boxed_slice()) as *mut i32;
            (ptr, len)
        }
        _ => (std::ptr::null_mut(), 0),
    };

    SortedDrawNumbersC {
        base: draw_numbers_to_c(numbers.base),
        sorted_numbers,
        sorted_numbers_len,
    }
}

fn string_to_c_ptr(value: String) -> *mut c_char {
    let sanitized = value.replace('\0', "");
    CString::new(sanitized)
        .expect("string sanitized to avoid interior nul")
        .into_raw()
}

fn optional_string_to_c_ptr(value: Option<String>) -> *mut c_char {
    match value {
        Some(text) => string_to_c_ptr(text),
        None => std::ptr::null_mut(),
    }
}
