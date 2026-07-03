#include <stdio.h>
#include <string.h>

#include <taiwan_lottery/query.h>

static void print_usage(const char *program) {
    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s local <game> period <PERIOD> [output_dir]\n", program);
    fprintf(stderr, "  %s local <game> month <YYYY-MM> [output_dir]\n", program);
    fprintf(stderr, "  %s local <game> month-range <YYYY-MM> <YYYY-MM> [output_dir]\n", program);
    fprintf(stderr, "  %s remote <game> period <PERIOD>\n", program);
    fprintf(stderr, "  %s remote <game> month <YYYY-MM>\n", program);
    fprintf(stderr, "  %s remote <game> month-range <YYYY-MM> <YYYY-MM>\n", program);
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
    const char *program = argc > 0 ? argv[0] : "query";
    const char *default_output_dir = "data";

    if (argc < 5) {
        print_usage(program);
        return 2;
    }

    const char *source = argv[1];
    int game = parse_game(argv[2]);
    const char *query_mode = argv[3];
    int is_month_range = strcmp(query_mode, "month-range") == 0;
    int value_count = is_month_range ? 2 : 1;
    int status;
    taiwan_lottery_history_draw_page *page = NULL;

    if (game < 0) {
        fprintf(stderr, "Invalid game: %s\n", argv[2]);
        print_usage(program);
        return 2;
    }

    if (strcmp(query_mode, "period") != 0 && strcmp(query_mode, "month") != 0 && !is_month_range) {
        fprintf(stderr, "Query mode must be period, month, or month-range\n");
        print_usage(program);
        return 2;
    }

    if (strcmp(source, "local") == 0) {
        if (argc < 4 + value_count + 1) {
            fprintf(stderr, "Missing query value(s) for mode: %s\n", query_mode);
            print_usage(program);
            return 2;
        }
        const char *output_dir = argc > 4 + value_count ? argv[4 + value_count] : default_output_dir;
        const char *period = strcmp(query_mode, "period") == 0 ? argv[4] : NULL;
        const char *month = !period ? argv[4] : NULL;
        const char *end_month = is_month_range ? argv[5] : month;

        status = query_history_draw(output_dir, game, period, month, end_month, &page);
    } else if (strcmp(source, "remote") == 0) {
        if (argc < 4 + value_count) {
            fprintf(stderr, "Missing query value(s) for mode: %s\n", query_mode);
            print_usage(program);
            return 2;
        }

        const char *period = strcmp(query_mode, "period") == 0 ? argv[4] : NULL;
        const char *month = !period ? argv[4] : NULL;
        const char *end_month = is_month_range ? argv[5] : month;

        status = query_history_draw_from_taiwan_lottery(game, period, month, end_month, &page);
    } else {
        print_usage(program);
        return 2;
    }

    if (status != TAIWAN_LOTTERY_OK) {
        fprintf(stderr, "Query failed (status=%d)\n", status);
        return 1;
    }

    printf("total_size=%zu\n", page->total_size);
    for (size_t i = 0; i < page->item_count; ++i) {
        const taiwan_lottery_history_draw_item *item = &page->items[i];
        printf("period=%s\n", item->period != NULL ? item->period : "");
        printf("date=%s\n", item->date != NULL ? item->date : "");

        if (item->numbers_draw != NULL && item->numbers_draw_len > 0) {
            printf("numbers_draw=");
            print_numbers(item->numbers_draw, item->numbers_draw_len);
            printf("\n");
        } else {
            printf("numbers_draw=<not available in local data>\n");
        }

        printf("numbers_sorted=");
        print_numbers(item->numbers_sorted, item->numbers_sorted_len);
        printf("\n\n");
    }

    free_history_draw_page(page);
    return 0;
}
