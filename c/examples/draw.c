#include <stdio.h>

#include <taiwan_lottery/draw.h>

#include "../src/game_args.h"

static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s <game>\n", program);
    fprintf(stderr, "  game: super-lotto638 | lotto649 | daily539 | 3d | 4d | 49m6 | 39m5 | 38m6 | 1224 | 740 | tic-tac-toe | 638 | bingo-bingo\n");
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

    game = taiwan_lottery_parse_game_arg(argv[1]);
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