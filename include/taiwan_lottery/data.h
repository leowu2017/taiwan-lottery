#ifndef TAIWAN_LOTTERY_DATA_H
#define TAIWAN_LOTTERY_DATA_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Return codes for all download_* C APIs */
#define TAIWAN_LOTTERY_OK 0
#define TAIWAN_LOTTERY_NULL_PATH 1
#define TAIWAN_LOTTERY_INVALID_PATH_UTF8 2
#define TAIWAN_LOTTERY_IO_ERROR 3
#define TAIWAN_LOTTERY_NETWORK_ERROR 4
#define TAIWAN_LOTTERY_PARSE_ERROR 5
#define TAIWAN_LOTTERY_NULL_DATASET_CODE 6
#define TAIWAN_LOTTERY_INVALID_DATASET_CODE_UTF8 7
#define TAIWAN_LOTTERY_INVALID_GAME 8
#define TAIWAN_LOTTERY_INVALID_SESSION 9
#define TAIWAN_LOTTERY_INVALID_QUERY_UTF8 10
#define TAIWAN_LOTTERY_NULL_RESULT_POINTER 11

/* History game values for get_history_draw* C APIs */
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

/* History session values for get_history_draw* C APIs */
#define TAIWAN_LOTTERY_HISTORY_SESSION_THIRD 0
#define TAIWAN_LOTTERY_HISTORY_SESSION_FOURTH 1
#define TAIWAN_LOTTERY_HISTORY_SESSION_FIFTH 2

typedef struct taiwan_lottery_history_draw_item {
	char* period;
	char* lottery_date;
	char* redeemable_date;
	int32_t* draw_number_size;
	size_t draw_number_size_len;
	int32_t* draw_number_appear;
	size_t draw_number_appear_len;
	uint8_t has_draw_number_appear;
} taiwan_lottery_history_draw_item;

typedef struct taiwan_lottery_history_draw_page {
	size_t total_size;
	size_t item_count;
	taiwan_lottery_history_draw_item* items;
} taiwan_lottery_history_draw_page;

int download_api_doc(const char* output_dir);
int download_dataset(const char* output_dir, const char* dataset_code);
int download_history_draw(const char* output_dir);
int download_history_draw_from_gov_data(const char* output_dir);
int download_history_draw_from_taiwan_lottery(const char* output_dir);
int download_all(const char* output_dir);

/*
 * Query draw results from downloaded local history files (output_dir/D423F).
 *
 * period/month/end_month are optional UTF-8 strings (pass NULL or "" when unused).
 * out_page receives ownership of a heap-allocated result page on success.
 * The caller must free it with free_history_draw_page().
 */
int get_history_draw(
	const char* output_dir,
	int game,
	const char* period,
	const char* month,
	const char* end_month,
	int session,
	taiwan_lottery_history_draw_page** out_page);

/*
 * Query draw results directly from Taiwan Lottery web API.
 */
int get_history_draw_from_taiwan_lottory(
	int game,
	const char* period,
	const char* month,
	const char* end_month,
	int session,
	taiwan_lottery_history_draw_page** out_page);

void free_history_draw_page(taiwan_lottery_history_draw_page* page);

#ifdef __cplusplus
}
#endif

#endif
