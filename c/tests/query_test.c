#include <assert.h>
#include <stddef.h>
#include <string.h>

#include <taiwan_lottery/query.h>

#ifndef TEST_REPO_ROOT
#error TEST_REPO_ROOT must be defined by the build system.
#endif

static void test_lotto649_local_query(void) {
    taiwan_lottery_history_draw_page *page = NULL;
    int status = query_history_draw(
        TEST_REPO_ROOT "/data",
        TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649,
        NULL,
        "2026-01",
        "2026-01",
        &page
    );

    assert(status == TAIWAN_LOTTERY_OK);
    assert(page != NULL);
    assert(page->item_count > 0);
    assert(page->items[0].numbers.base.numbers != NULL);
    assert(page->items[0].numbers.base.numbers_len == 7);
    assert(page->items[0].numbers.sorted_numbers != NULL);
    assert(page->items[0].numbers.sorted_numbers_len == 7);
    assert(strcmp(page->items[0].period, "") != 0);

    free_history_draw_page(page);
}

static void test_3d_local_query(void) {
    taiwan_lottery_history_draw_page *page = NULL;
    int status = query_history_draw(
        TEST_REPO_ROOT "/data",
        TAIWAN_LOTTERY_HISTORY_GAME_3D,
        NULL,
        "2026-01",
        "2026-01",
        &page
    );

    assert(status == TAIWAN_LOTTERY_OK);
    assert(page != NULL);
    assert(page->item_count > 0);
    assert(page->items[0].numbers.base.numbers != NULL);
    assert(page->items[0].numbers.base.numbers_len == 3);
    assert(page->items[0].numbers.sorted_numbers == NULL);
    assert(page->items[0].numbers.sorted_numbers_len == 0);

    free_history_draw_page(page);
}

static void test_invalid_game_code_returns_error(void) {
    taiwan_lottery_history_draw_page *page = NULL;
    int status = query_history_draw(
        TEST_REPO_ROOT "/data",
        999,
        NULL,
        "2026-01",
        "2026-01",
        &page
    );

    assert(status == TAIWAN_LOTTERY_INVALID_GAME);
    assert(page == NULL);
}

static void test_invalid_game_code_returns_error_for_remote_query(void) {
    taiwan_lottery_history_draw_page *page = NULL;
    int status = query_history_draw_from_taiwan_lottery(
        999,
        NULL,
        "2026-01",
        "2026-01",
        &page
    );

    assert(status == TAIWAN_LOTTERY_INVALID_GAME);
    assert(page == NULL);
}

static void test_query_month_range_by_game(void) {
    taiwan_lottery_query_month_range *range = NULL;
    int status = lottery_game_query_month_range(
        TAIWAN_LOTTERY_HISTORY_GAME_1224,
        &range
    );

    assert(status == TAIWAN_LOTTERY_OK);
    assert(range != NULL);
    assert(range->min_month != NULL);
    assert(range->max_month != NULL);
    assert(strcmp(range->min_month, "2014-01") == 0);
    assert(strcmp(range->max_month, "2023-12") == 0);

    free_lottery_game_query_month_range(range);
}

static void test_query_month_range_invalid_game_returns_error(void) {
    taiwan_lottery_query_month_range *range = NULL;
    int status = lottery_game_query_month_range(999, &range);

    assert(status == TAIWAN_LOTTERY_INVALID_GAME);
    assert(range == NULL);
}

static void test_game_metadata_exposes_number_rules(void) {
    taiwan_lottery_game_metadata *metadata = NULL;
    int status = lottery_game_metadata(TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649, &metadata);

    assert(status == TAIWAN_LOTTERY_OK);
    assert(metadata != NULL);
    assert(metadata->display_name != NULL);
    assert(metadata->display_name_english != NULL);
    assert(metadata->display_name_chinese != NULL);
    assert(metadata->number_rule != NULL);
    assert(strcmp(metadata->display_name, "Lotto 649") == 0);
    assert(strcmp(metadata->display_name_english, "Lotto 649") == 0);
    assert(strlen(metadata->display_name_chinese) > 0);
    assert(strcmp(metadata->display_name_chinese, metadata->display_name_english) != 0);
    assert(strcmp(metadata->number_rule, "6 numbers from 1-49, plus 1 bonus number from 1-49") == 0);
    assert(metadata->number_range_count == 2);
    assert(metadata->number_ranges != NULL);
    assert(strcmp(metadata->number_ranges[0].name, "main") == 0);
    assert(metadata->number_ranges[0].picks == 6);
    assert(metadata->number_ranges[0].min == 1);
    assert(metadata->number_ranges[0].max == 49);
    assert(metadata->number_ranges[0].allow_repeat == 0);
    assert(strcmp(metadata->number_ranges[1].name, "bonus") == 0);

    free_lottery_game_metadata(metadata);
}

static void test_game_metadata_for_bingo_bingo_exposes_super_rule(void) {
    taiwan_lottery_game_metadata *metadata = NULL;
    int status = lottery_game_metadata(TAIWAN_LOTTERY_HISTORY_GAME_BINGO_BINGO, &metadata);

    assert(status == TAIWAN_LOTTERY_OK);
    assert(metadata != NULL);
    assert(metadata->number_range_count == 2);
    assert(strcmp(metadata->number_ranges[1].name, "super") == 0);
    assert(metadata->number_ranges[1].picks == 1);
    assert(metadata->number_ranges[1].min == 1);
    assert(metadata->number_ranges[1].max == 80);

    free_lottery_game_metadata(metadata);
}

static void test_game_metadata_with_language_returns_chinese_display_name(void) {
    taiwan_lottery_game_metadata *metadata = NULL;
    int status = lottery_game_metadata_with_language(
        TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649,
        TAIWAN_LOTTERY_DISPLAY_LANGUAGE_CHINESE,
        &metadata
    );

    assert(status == TAIWAN_LOTTERY_OK);
    assert(metadata != NULL);
    assert(strlen(metadata->display_name) > 0);
    assert(strcmp(metadata->display_name, metadata->display_name_chinese) == 0);
    assert(strcmp(metadata->display_name, metadata->display_name_english) != 0);
    assert(strcmp(metadata->display_name_english, "Lotto 649") == 0);

    free_lottery_game_metadata(metadata);
}

static void test_game_metadata_with_language_rejects_invalid_language(void) {
    taiwan_lottery_game_metadata *metadata = NULL;
    int status = lottery_game_metadata_with_language(
        TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649,
        999,
        &metadata
    );

    assert(status == TAIWAN_LOTTERY_INVALID_LANGUAGE);
    assert(metadata == NULL);
}

static void test_game_metadata_invalid_game_returns_error(void) {
    taiwan_lottery_game_metadata *metadata = NULL;
    int status = lottery_game_metadata(999, &metadata);

    assert(status == TAIWAN_LOTTERY_INVALID_GAME);
    assert(metadata == NULL);
}

static void test_remote_query_param_support_non_bingo(void) {
    taiwan_lottery_remote_query_param_support support = {0};
    int status = lottery_game_remote_query_param_support(
        TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649,
        &support
    );

    assert(status == TAIWAN_LOTTERY_OK);
    assert(support.month != 0);
    assert(support.end_month != 0);
    assert(support.open_date == 0);
    assert(support.period != 0);
}

static void test_remote_query_param_support_bingo(void) {
    taiwan_lottery_remote_query_param_support support = {0};
    int status = lottery_game_remote_query_param_support(
        TAIWAN_LOTTERY_HISTORY_GAME_BINGO_BINGO,
        &support
    );

    assert(status == TAIWAN_LOTTERY_OK);
    assert(support.month == 0);
    assert(support.end_month == 0);
    assert(support.open_date != 0);
    assert(support.period == 0);
}

int main(void) {
    test_lotto649_local_query();
    test_3d_local_query();
    test_invalid_game_code_returns_error();
    test_invalid_game_code_returns_error_for_remote_query();
    test_query_month_range_by_game();
    test_query_month_range_invalid_game_returns_error();
    test_game_metadata_exposes_number_rules();
    test_game_metadata_for_bingo_bingo_exposes_super_rule();
    test_game_metadata_with_language_returns_chinese_display_name();
    test_game_metadata_with_language_rejects_invalid_language();
    test_game_metadata_invalid_game_returns_error();
    test_remote_query_param_support_non_bingo();
    test_remote_query_param_support_bingo();
    return 0;
}