#ifndef TAIWAN_LOTTERY_QUERY_H
#define TAIWAN_LOTTERY_QUERY_H

#include <taiwan_lottery/numbers.h>

#ifdef __cplusplus
extern "C" {
#endif

#define TAIWAN_LOTTERY_OK 0
#define TAIWAN_LOTTERY_NULL_PATH 1
#define TAIWAN_LOTTERY_INVALID_PATH_UTF8 2
#define TAIWAN_LOTTERY_IO_ERROR 3
#define TAIWAN_LOTTERY_NETWORK_ERROR 4
#define TAIWAN_LOTTERY_PARSE_ERROR 5
#define TAIWAN_LOTTERY_NULL_DATASET_CODE 6
#define TAIWAN_LOTTERY_INVALID_DATASET_CODE_UTF8 7
#define TAIWAN_LOTTERY_INVALID_GAME 8
#define TAIWAN_LOTTERY_INVALID_QUERY_UTF8 9
#define TAIWAN_LOTTERY_NULL_RESULT_POINTER 10
#define TAIWAN_LOTTERY_INVALID_LANGUAGE 11

/* History game values for query_history_draw* C APIs */
#define TAIWAN_LOTTERY_HISTORY_GAME_SUPER_LOTTO_638 0
#define TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649 1
#define TAIWAN_LOTTERY_HISTORY_GAME_DAILY_539 2
#define TAIWAN_LOTTERY_HISTORY_GAME_3D 3
#define TAIWAN_LOTTERY_HISTORY_GAME_4D 4
#define TAIWAN_LOTTERY_HISTORY_GAME_49M6 5
#define TAIWAN_LOTTERY_HISTORY_GAME_39M5 6
#define TAIWAN_LOTTERY_HISTORY_GAME_38M6 7
#define TAIWAN_LOTTERY_HISTORY_GAME_1224 8
#define TAIWAN_LOTTERY_HISTORY_GAME_740 9
#define TAIWAN_LOTTERY_HISTORY_GAME_TIC_TAC_TOE 10
#define TAIWAN_LOTTERY_HISTORY_GAME_638 11
#define TAIWAN_LOTTERY_HISTORY_GAME_BINGO_BINGO 12

/* Display language values for lottery_game_metadata_with_language C API. */
#define TAIWAN_LOTTERY_DISPLAY_LANGUAGE_ENGLISH 0
#define TAIWAN_LOTTERY_DISPLAY_LANGUAGE_CHINESE 1

typedef struct taiwan_lottery_history_draw_item {
	/* Draw period identifier. */
	char* period;
	/* Draw date string when available. */
	char* date;
	/* Redeemable date string when available. */
	char* redeemable_date;
	/* Draw-order numbers and optional sorted view. */
	sorted_draw_numbers numbers;
} taiwan_lottery_history_draw_item;

typedef struct taiwan_lottery_history_draw_page {
	/* Total result size after filtering and deduplication. */
	size_t total_size;
	/* Number of returned items in items. */
	size_t item_count;
	/* Caller-owned array of history draw items. */
	taiwan_lottery_history_draw_item* items;
} taiwan_lottery_history_draw_page;

typedef struct taiwan_lottery_query_month_range {
	/* Earliest supported query month in YYYY-MM. */
	char* min_month;
	/* Latest supported query month in YYYY-MM. */
	char* max_month;
} taiwan_lottery_query_month_range;

typedef struct taiwan_lottery_game_number_rule {
	/* Segment name such as main, bonus, super, or zone_1. */
	char* name;
	/* How many numbers to pick from this segment. */
	size_t picks;
	/* Inclusive minimum value. */
	int32_t min;
	/* Inclusive maximum value. */
	int32_t max;
	/* Non-zero when values in this segment may repeat. */
	int32_t allow_repeat;
} taiwan_lottery_game_number_rule;

typedef struct taiwan_lottery_game_metadata {
	/* UI display name for the game (defaults to English). */
	char* display_name;
	/* English display name for the game. */
	char* display_name_english;
	/* Chinese display name for the game. */
	char* display_name_chinese;
	/* Human-readable rule summary. */
	char* number_rule;
	/* Number of entries in number_ranges. */
	size_t number_range_count;
	/* Caller-owned array describing each number-selection segment. */
	taiwan_lottery_game_number_rule* number_ranges;
} taiwan_lottery_game_metadata;

typedef struct taiwan_lottery_remote_query_param_support {
	/* Non-zero when month is supported. */
	int32_t month;
	/* Non-zero when end_month is supported. */
	int32_t end_month;
	/* Non-zero when openDate is supported. */
	int32_t open_date;
	/* Non-zero when period is supported. */
	int32_t period;
} taiwan_lottery_remote_query_param_support;

/* Query history draw data from downloaded local files under output_dir/D423F. */
int query_history_draw(
	const char* output_dir,
	int game,
	const char* period,
	const char* month,
	const char* end_month,
	taiwan_lottery_history_draw_page** out_page);

/* Query history draw data directly from Taiwan Lottery web APIs. */
int query_history_draw_from_taiwan_lottery(
	int game,
	const char* period,
	const char* month,
	const char* end_month,
	taiwan_lottery_history_draw_page** out_page);

/* Get supported query month bounds for one game. Caller must free out_range. */
int lottery_game_query_month_range(
	int game,
	taiwan_lottery_query_month_range** out_range);

/* Get display metadata and number-rule segments for one game. Caller must free out_metadata. */
int lottery_game_metadata(
	int game,
	taiwan_lottery_game_metadata** out_metadata);

/*
 * Get display metadata and number-rule segments for one game with explicit language.
 * Pass TAIWAN_LOTTERY_DISPLAY_LANGUAGE_ENGLISH or TAIWAN_LOTTERY_DISPLAY_LANGUAGE_CHINESE.
 * Caller must free out_metadata.
 */
int lottery_game_metadata_with_language(
	int game,
	int language,
	taiwan_lottery_game_metadata** out_metadata);

/* Get per-game remote query parameter support flags. */
int lottery_game_remote_query_param_support(
	int game,
	taiwan_lottery_remote_query_param_support* out_support);

/* Release memory returned by query_history_draw or query_history_draw_from_taiwan_lottery. */
void free_history_draw_page(taiwan_lottery_history_draw_page* page);
/* Release memory returned by lottery_game_query_month_range. */
void free_lottery_game_query_month_range(taiwan_lottery_query_month_range* range);
/* Release memory returned by lottery_game_metadata. */
void free_lottery_game_metadata(taiwan_lottery_game_metadata* metadata);

#ifdef __cplusplus
}
#endif

#endif
