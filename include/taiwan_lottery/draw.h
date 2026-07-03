#ifndef TAIWAN_LOTTERY_DRAW_H
#define TAIWAN_LOTTERY_DRAW_H

#include <taiwan_lottery/numbers.h>
#include <taiwan_lottery/query.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef bonus_draw_numbers taiwan_lottery_draw_result;

int draw_by_game(int game, taiwan_lottery_draw_result** out_result);
void free_draw_result(taiwan_lottery_draw_result* result);

#ifdef __cplusplus
}
#endif

#endif