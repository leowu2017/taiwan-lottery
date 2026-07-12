use std::ffi::CString;

use super::{DrawNumbersC, HistoryDrawItemC, SortedDrawNumbersC};

pub(crate) fn free_history_draw_item(item: &HistoryDrawItemC) {
    if !item.period.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.period) };
    }
    if !item.date.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.date) };
    }
    if !item.redeemable_date.is_null() {
        // SAFETY: pointer was created by CString::into_raw in this crate.
        let _ = unsafe { CString::from_raw(item.redeemable_date) };
    }

    free_sorted_draw_numbers(&item.numbers);
}

pub(crate) fn free_draw_numbers(numbers: &DrawNumbersC) {
    if !numbers.numbers.is_null() {
        let _ = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                numbers.numbers,
                numbers.numbers_len,
            ))
        };
    }
}

pub(crate) fn free_sorted_draw_numbers(numbers: &SortedDrawNumbersC) {
    free_draw_numbers(&numbers.base);

    if !numbers.sorted_numbers.is_null() {
        let _ = unsafe {
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(
                numbers.sorted_numbers,
                numbers.sorted_numbers_len,
            ))
        };
    }
}
