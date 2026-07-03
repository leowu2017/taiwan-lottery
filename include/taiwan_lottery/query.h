#ifndef TAIWAN_LOTTERY_QUERY_H
#define TAIWAN_LOTTERY_QUERY_H

#include <stddef.h>
#include <stdint.h>

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

typedef struct taiwan_lottery_history_draw_item {
	char* period;
	char* date;
	char* redeemable_date;
	int32_t* numbers_sorted;
	size_t numbers_sorted_len;
	int32_t* numbers_draw;
	size_t numbers_draw_len;
} taiwan_lottery_history_draw_item;

typedef struct taiwan_lottery_history_draw_page {
	size_t total_size;
	size_t item_count;
	taiwan_lottery_history_draw_item* items;
} taiwan_lottery_history_draw_page;

int query_history_draw(
	const char* output_dir,
	int game,
	const char* period,
	const char* month,
	const char* end_month,
	taiwan_lottery_history_draw_page** out_page);

int query_history_draw_from_taiwan_lottery(
	int game,
	const char* period,
	const char* month,
	const char* end_month,
	taiwan_lottery_history_draw_page** out_page);

void free_history_draw_page(taiwan_lottery_history_draw_page* page);

#ifdef __cplusplus
}
#endif

#endif
