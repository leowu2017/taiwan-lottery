#include <assert.h>
#include <stddef.h>

#include <taiwan_lottery/draw.h>

typedef struct draw_expectation {
    int game;
    size_t numbers_len;
    int has_bonus;
    int min_value;
    int max_value;
    int bonus_min;
    int bonus_max;
} draw_expectation;

static void assert_range(const int32_t *numbers, size_t len, int min_value, int max_value) {
    size_t index;

    for (index = 0; index < len; ++index) {
        assert(numbers[index] >= min_value);
        assert(numbers[index] <= max_value);
    }
}

int main(void) {
    const draw_expectation cases[] = {
        {TAIWAN_LOTTERY_HISTORY_GAME_SUPER_LOTTO_638, 6, 1, 1, 49, 1, 8},
        {TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649, 6, 1, 1, 49, 1, 49},
        {TAIWAN_LOTTERY_HISTORY_GAME_DAILY_539, 5, 1, 1, 39, 1, 39},
        {TAIWAN_LOTTERY_HISTORY_GAME_3D, 3, 0, 0, 9, 0, 0},
        {TAIWAN_LOTTERY_HISTORY_GAME_4D, 4, 0, 0, 9, 0, 0},
        {TAIWAN_LOTTERY_HISTORY_GAME_49M6, 6, 0, 1, 49, 0, 0},
        {TAIWAN_LOTTERY_HISTORY_GAME_39M5, 5, 0, 1, 39, 0, 0},
        {TAIWAN_LOTTERY_HISTORY_GAME_38M6, 6, 0, 1, 38, 0, 0},
        {TAIWAN_LOTTERY_HISTORY_GAME_1224, 6, 0, 1, 36, 0, 0},
        {TAIWAN_LOTTERY_HISTORY_GAME_740, 7, 1, 1, 38, 1, 8},
        {TAIWAN_LOTTERY_HISTORY_GAME_TIC_TAC_TOE, 20, 0, 1, 80, 0, 0},
        {TAIWAN_LOTTERY_HISTORY_GAME_638, 6, 1, 1, 49, 1, 10},
        {TAIWAN_LOTTERY_HISTORY_GAME_BINGO_BINGO, 20, 1, 1, 80, 1, 80},
    };
    size_t index;

    for (index = 0; index < sizeof(cases) / sizeof(cases[0]); ++index) {
        taiwan_lottery_draw_result *result = NULL;
        int status = draw_by_game(cases[index].game, &result);

        assert(status == TAIWAN_LOTTERY_OK);
        assert(result != NULL);
        assert(result->base.numbers != NULL);
        assert(result->base.numbers_len == cases[index].numbers_len);
        assert_range(
            result->base.numbers,
            result->base.numbers_len,
            cases[index].min_value,
            cases[index].max_value
        );

        if (cases[index].has_bonus) {
            assert(result->has_bonus == 1);
            assert(result->bonus >= cases[index].bonus_min);
            assert(result->bonus <= cases[index].bonus_max);
        } else {
            assert(result->has_bonus == 0);
        }

        free_draw_result(result);
    }

    return 0;
}