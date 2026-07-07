#ifndef TAIWAN_LOTTERY_NUMBERS_H
#define TAIWAN_LOTTERY_NUMBERS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct draw_numbers {
	/* Primary numbers in draw order. */
	int32_t* numbers;
	/* Length of the numbers array. */
	size_t numbers_len;
} draw_numbers;

typedef struct bonus_draw_numbers {
	/* Primary numbers for the result. */
	draw_numbers base;
	/* Non-zero when bonus contains a valid value. */
	int32_t has_bonus;
	/* Bonus or super number when has_bonus != 0. */
	int32_t bonus;
} bonus_draw_numbers;

typedef struct sorted_draw_numbers {
	/* Primary numbers in draw order. */
	draw_numbers base;
	/* Sorted-number view when available, otherwise NULL. */
	int32_t* sorted_numbers;
	/* Length of sorted_numbers, or 0 when not available. */
	size_t sorted_numbers_len;
} sorted_draw_numbers;

#ifdef __cplusplus
}
#endif

#endif