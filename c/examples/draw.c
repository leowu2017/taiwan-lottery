#include <stdio.h>
#include <string.h>

#include <taiwan_lottery/draw.h>

static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s <game>\n", program);
    fprintf(stderr, "  game: super-lotto638 | lotto649 | daily539 | 3d | 4d | 49m6 | 39m5 | 38m6 | 1224 | 740 | tic-tac-toe | 638\n");
}

static int parse_game(const char *value) {
    if (strcmp(value, "super-lotto638") == 0 || strcmp(value, "5134") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_SUPER_LOTTO_638;
    }
    if (strcmp(value, "lotto649") == 0 || strcmp(value, "5118") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_LOTTO_649;
    }
    if (strcmp(value, "daily539") == 0 || strcmp(value, "5120") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_DAILY_539;
    }
    if (strcmp(value, "3d") == 0 || strcmp(value, "2108") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_3D;
    }
    if (strcmp(value, "4d") == 0 || strcmp(value, "2109") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_4D;
    }
    if (strcmp(value, "49m6") == 0 || strcmp(value, "1121") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_49M6;
    }
    if (strcmp(value, "39m5") == 0 || strcmp(value, "1197") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_39M5;
    }
    if (strcmp(value, "38m6") == 0 || strcmp(value, "5122") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_38M6;
    }
    if (strcmp(value, "1224") == 0 || strcmp(value, "5290") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_1224;
    }
    if (strcmp(value, "740") == 0 || strcmp(value, "2300") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_740;
    }
    if (strcmp(value, "tic-tac-toe") == 0 || strcmp(value, "2400") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_TIC_TAC_TOE;
    }
    if (strcmp(value, "638") == 0 || strcmp(value, "2500") == 0) {
        return TAIWAN_LOTTERY_HISTORY_GAME_638;
    }

    return -1;
}

static void print_numbers(const int32_t *values, size_t len) {
    size_t i;

    printf("[");
    for (i = 0; i < len; ++i) {
        if (i > 0) {
            printf(", ");
        }
        printf("%d", values[i]);
    }
    printf("]");
}

int main(int argc, char **argv) {
    const char *program = argc > 0 ? argv[0] : "draw";
    taiwan_lottery_draw_result *result = NULL;
    int game;
    int status;

    if (argc < 2) {
        print_usage(program);
        return 2;
    }

    game = parse_game(argv[1]);
    if (game < 0) {
        fprintf(stderr, "Invalid game: %s\n", argv[1]);
        print_usage(program);
        return 2;
    }

    status = draw_by_game(game, &result);
    if (status != TAIWAN_LOTTERY_OK) {
        fprintf(stderr, "Draw failed (status=%d)\n", status);
        return 1;
    }

    printf("numbers=");
    print_numbers(result->base.numbers, result->base.numbers_len);
    printf("\n");

    if (result->has_bonus) {
        printf("bonus=%d\n", result->bonus);
    }

    free_draw_result(result);
    return 0;
}