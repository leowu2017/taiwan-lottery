#ifndef TAIWAN_LOTTERY_NUMBERS_H
#define TAIWAN_LOTTERY_NUMBERS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct draw_numbers {
	int32_t* numbers;
	size_t numbers_len;
} draw_numbers;

typedef struct bonus_draw_numbers {
	draw_numbers base;
	int32_t has_bonus;
	int32_t bonus;
} bonus_draw_numbers;

typedef struct sorted_draw_numbers {
	draw_numbers base;
	int32_t* sorted_numbers;
	size_t sorted_numbers_len;
} sorted_draw_numbers;

#ifdef __cplusplus
}
#endif

#endif