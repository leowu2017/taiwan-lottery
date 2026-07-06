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

int main(void) {
    test_lotto649_local_query();
    test_3d_local_query();
    test_invalid_game_code_returns_error();
    test_invalid_game_code_returns_error_for_remote_query();
    test_query_month_range_by_game();
    test_query_month_range_invalid_game_returns_error();
    return 0;
}